use std::thread;

use anyhow::Result;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use log::{info, LevelFilter};

use display::setup_display;
use net::setup_network;

use crate::display::show_logo;
use crate::display::ui::show_ui;
use crate::utils::state::State;

mod display;
mod net;
mod fs;
pub mod button;
mod utils;

pub const DISPLAY_WIDTH: u32 = 128;
pub const DISPLAY_HEIGHT: u32 = 128;

fn main() -> Result<()> {
    info!("setup system!");
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();
    esp_idf_svc::log::set_target_level("*", LevelFilter::Error)?;
    let per = Peripherals::take()?;
    info!("setup display!");

    let (state_sender, state_receiver) = std::sync::mpsc::channel::<State>();
    let button_sender = state_sender.clone();
    thread::Builder::new()
        .stack_size(4096)
        .name(String::from("BUTTON"))
        .spawn(||{
            button::button_state(per.pins.gpio8, per.pins.gpio10, button_sender)
        }).unwrap();
    let mut display = setup_display(
        per.spi2,
        per.pins.gpio3,
        per.pins.gpio2,
        per.pins.gpio4,
        per.pins.gpio5,
        per.pins.gpio0,
    )?;
    show_logo(&mut display, DISPLAY_WIDTH, DISPLAY_HEIGHT).expect("show_logo failed!");
    let modem = per.modem;
    let sys_loop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;
    info!("setup network!");
    setup_network(modem, sys_loop, nvs)?;
    show_ui(display, state_receiver).unwrap();
    Ok(())
}
