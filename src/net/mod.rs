use std::ffi::c_long;
use std::ops::Deref;
use std::thread;
use std::time::Duration;

use anyhow;
use embedded_svc::ipv4::IpInfo;
use esp_idf_hal::modem::Modem;
use esp_idf_svc::eventloop::EspEventLoop;
use esp_idf_svc::eventloop::System;
use esp_idf_svc::nvs::{EspNvsPartition, NvsDefault};
use esp_idf_svc::sys::{clock_settime, clockid_t, time_t, timespec};
use esp_idf_svc::wifi::{AuthMethod, BlockingWifi, ClientConfiguration, Configuration, EspWifi};
use log::{error, info};

use crate::fs::config::CONFIG;
use crate::net::ntp::NtpClient;

pub mod ntp;
static mut NET_INFO: Option<IpInfo> = None;
pub fn net_info() -> Option<IpInfo> {
    unsafe {NET_INFO}
}
fn setup_wifi(
    modem: Modem,
    sys_loop: EspEventLoop<System>,
    nvs: EspNvsPartition<NvsDefault>,
) -> anyhow::Result<BlockingWifi<EspWifi<'static>>> {

    let mut wifi = BlockingWifi::wrap(EspWifi::new(modem, sys_loop.clone(), Some(nvs))?, sys_loop)?;
    let accounts = match CONFIG.deref() {
        None => {
            panic!("fetch wifi accounts from partition failed");
        }
        Some(config) => {
            &config.wifi

        }
    };
    'WIFI_OK: for account in accounts {
        let wifi_configuration: Configuration = Configuration::Client(ClientConfiguration {
            ssid: account.ssid.parse()
                .map_err(|_|anyhow::Error::msg("account parse failed"))?,
            bssid: None,
            auth_method: AuthMethod::WPAWPA2Personal,
            password: account.password.parse()
                .map_err(|_|anyhow::Error::msg("password parse failed"))?,
            channel: None,
        });

        wifi.set_configuration(&wifi_configuration)?;
        wifi.start()?;
        info!("try connect {}", account.ssid);
        match wifi.connect() {
            Ok(_) => match wifi.wait_netif_up() {
                Ok(_) => break 'WIFI_OK,
                Err(_) => continue,
            },
            Err(_) => continue,
        }
    }
    unsafe {NET_INFO.replace(wifi.wifi().sta_netif().get_ip_info()?)};
    Ok(wifi)
}

static mut WIFI: Option<BlockingWifi<EspWifi>> = None;
pub fn setup_network(
    modem: Modem,
    sys_loop: EspEventLoop<System>,
    nvs: EspNvsPartition<NvsDefault>,
) -> anyhow::Result<()> {
    let wifi = setup_wifi(modem, sys_loop.clone(), nvs.clone()).expect("setup_wifi failed");
    info!("{:?}", wifi.wifi().sta_netif().get_ip_info().unwrap());
    unsafe {WIFI.replace(wifi)};
    thread::Builder::new()
        .stack_size(1024 * 16)
        .name("ntp-update".into())
        .spawn(move || {
            match sync_time() {
                Ok(_) => info!("ntp sync time succeed!"),
                Err(e) => error!("ntp sync time failed! {}", e),
            }
            let mut interval = 0u32;
            let sync_time_interval = match CONFIG.deref() {
                None => 3600,
                Some(config) => config.sync_time_interval,
            };

            loop {
                if interval >= sync_time_interval {
                    match sync_time() {
                        Ok(_) => info!("ntp sync time succeed!"),
                        Err(e) => error!("ntp sync time failed! {}", e),
                    }
                    interval = 0
                }
                thread::sleep(Duration::from_secs(1));
                interval += 1;
            }
        })
        .unwrap();
    Ok(())
}


const CLOCK_MONOTONIC: clockid_t = 1;
pub fn sync_time() -> anyhow::Result<()> {
    let ntp_server = match CONFIG.deref() {
        None => "ntp0.ntp-servers.net",
        Some(config) => &*config.ntp_server,
    };
    let client = NtpClient::new();
    let res = client.request(ntp_server)?;
    let seconds = res.unix_time.clone();
    let t = timespec {
        tv_sec: seconds as time_t,
        tv_nsec: (seconds / 1000000000) as c_long,
    };
    unsafe {
        clock_settime(CLOCK_MONOTONIC, &t as *const timespec);
    }
    info!("sync time: {}", res.format_time("%Y-%m-%d %H:%M:%S")?);
    Ok(())
}

