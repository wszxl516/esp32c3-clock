#![allow(dead_code)]

use std::ffi::{CStr, CString};

use esp_idf_hal::sys::{esp, esp_partition_erase_range, esp_partition_find_first,
                       esp_partition_read, esp_partition_subtype_t_ESP_PARTITION_SUBTYPE_ANY,
                       esp_partition_t, esp_partition_type_t_ESP_PARTITION_TYPE_ANY,
                       esp_partition_write, EspError};
use lazy_static::lazy_static;
use log::{error, info};

pub mod config;

#[repr(transparent)]
#[derive(Copy, Clone, Debug)]
pub struct Partition{
    pub raw: esp_partition_t
}
impl Partition{
    pub fn new(name: &str) -> Option<Self>{
        let partition_name = CString::new(name).ok()?;
        let part = unsafe { esp_partition_find_first(
            esp_partition_type_t_ESP_PARTITION_TYPE_ANY,
            esp_partition_subtype_t_ESP_PARTITION_SUBTYPE_ANY,
            partition_name.as_ptr(),
        )};
        match !part.is_null() {
            true => Some(Self{raw: unsafe{*part}}),
            false => None
        }

    }

    pub const fn address(&self) -> u32 {
        self.raw.address
    }
    pub const fn size(&self) -> u32 {
        self.raw.size
    }
    pub const fn raw(&self) ->  esp_partition_t {
        self.raw
    }
    pub fn label(&self) -> anyhow::Result<&str, anyhow::Error> {
        unsafe { CStr::from_ptr(self.raw.label.as_ptr()).to_str().map_err(|_|anyhow::Error::msg("label decode failed!")) }
    }

    pub fn read(&self, offset: usize, buffer: &mut [u8]) -> anyhow::Result<(), EspError>{
        esp!(unsafe {esp_partition_read(&self.raw, offset, buffer.as_mut_ptr() as _ , buffer.len()) })
    }

    pub fn write(&self, offset: usize, buffer: &[u8]) -> anyhow::Result<(), EspError>{
        esp!(unsafe {esp_partition_write(&self.raw, offset, buffer.as_ptr() as _ , buffer.len()) })
    }
    pub fn erase(&self, offset: usize, size: usize) -> anyhow::Result<(), EspError>{
        esp!(unsafe {esp_partition_erase_range(&self.raw, offset, size) })
    }
}

unsafe impl Send for Partition {}
unsafe impl Sync for Partition {}

lazy_static!{
    pub static ref DATA_PART: Option<Partition> = {
        match Partition::new("data"){
            None => {
                error!("can not find partition: data");
                None
            },
            Some(part) =>{
                info!("partition name: {}, offset: {:#x}, part size: {:#x}",
                                          part.label().unwrap_or("Unknown"),
                                          part.address(), part.size());
                Some(part)
            }
        }
    };
}
