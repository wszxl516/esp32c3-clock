use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

use crate::fs::DATA_PART;

#[derive(Serialize, Deserialize, Debug)]
pub struct Wifi {
    pub ssid: String,
    pub password: String,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub wifi: Vec<Wifi>,
    pub sync_time_interval: u32,
    pub date_fixed_offset: i32
}
impl Config{
    pub fn from_partition() -> Option<Config>{
        let partition = *DATA_PART;
        match partition {
            None => {
                None
            },
            Some(part) => {
                let mut buffer = vec![0u8; 512];
                part.read(0, &mut buffer).ok()?;
                let (end_idx, _) = buffer.iter().enumerate().find(|(_i, v)|**v == 0xff)?;
                let json_str = std::str::from_utf8(buffer.split_at(end_idx).0).ok()?;
                match serde_json::from_str::<Config>(json_str) {
                    Ok(accounts) =>Some(accounts),
                    Err(_) => {
                        None
                    }
                }
            }
        }
    }
}

lazy_static!{
    pub static ref CONFIG: Option<Config> = Config::from_partition();
}