use std::fmt::{Display, Formatter};

use esp_idf_hal::sys::heap_caps_get_info;
use esp_idf_svc::sys::{esp, esp_efuse_mac_get_default, MALLOC_CAP_DEFAULT, multi_heap_info_t};

pub mod state;

#[repr(transparent)]
pub struct DeviceID([u8; 6]);
impl DeviceID{
    pub fn get() -> Self{
        let mut mac = [0u8; 6];
        match esp!(unsafe { esp_efuse_mac_get_default(mac.as_mut_ptr() as *mut _) }){
            Ok(_) => DeviceID(mac),
            Err(_) => DeviceID([0;6])
        }
    }
}
impl Display for DeviceID{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}", self.0[0], self.0[1], self.0[2], self.0[3], self.0[4], self.0[5])
    }
}


pub struct MemInfo{
    pub info: multi_heap_info_t
}

impl MemInfo{
    pub fn new() -> MemInfo {
        Self{ info: Default::default() }
    }
    pub fn fetch(&mut self){
         unsafe {heap_caps_get_info(&mut self.info, MALLOC_CAP_DEFAULT)}
    }
    #[allow(dead_code)]
    pub fn percent(&self) -> f32{
        let free = self.info.total_free_bytes as f32;
        let total = (self.info.total_free_bytes + self.info.total_allocated_bytes) as f32;
        free / total
    }
    pub fn kb(&self) -> (f32, f32) {
        let free = self.info.total_free_bytes as f32;
        let total = (self.info.total_free_bytes + self.info.total_allocated_bytes) as f32;
        (free / 1024.0, total / 1024.0)
    }

}

pub fn inert_lf(buffer: &str, p: usize) -> String{
    let mut msg = String::new();
    let buffer_len = buffer.len();
    let len = buffer_len / p;
    if len > 0 {
        for i in (0..buffer_len).step_by(p){
            let o = if i + p > buffer_len {
                buffer_len
            }else {
                i + p
            };
            msg += &buffer[i..o];
            msg += "\n"
        }
    }
    else {
        msg += buffer
    }
    msg
}