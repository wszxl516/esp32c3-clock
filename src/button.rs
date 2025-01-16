use std::sync::mpsc::Sender;
use std::thread::sleep;
use std::time::Duration;
use button_driver::{Button, ButtonConfig, Mode};
use esp_idf_hal::gpio::PinDriver;
use esp_idf_hal::sys::{esp, gpio_config, gpio_config_t};
use esp_idf_svc::hal::gpio::IOPin;
use log::debug;
use crate::utils::state::{Btn, State};

pub fn button_state(btn0_pin: impl IOPin, btn1_pin: impl IOPin, sender: Sender<State>) -> anyhow::Result<()>{

    let config = ButtonConfig{
        mode: Mode::PullUp,
        ..Default::default()
    };

    let mut select_btn = Button::new(PinDriver::input(btn0_pin)?, config);
    let mut ok_btn = Button::new(PinDriver::input(btn1_pin)?, config);


    {
        // this config will work
        let g_config = gpio_config_t{
            pin_bit_mask: (1<<8) | (1<<10),
            mode: 1,
            pull_up_en: 1,
            pull_down_en: 0,
            intr_type: 1,
        };
        unsafe {esp!(gpio_config(&g_config))?;}
    }
    loop {
        select_btn.tick();
        ok_btn.tick();

        if select_btn.is_clicked() {
            sender.send(State::Btn(Btn::Right))?;
            debug!("Left Click");
        } else if select_btn.is_double_clicked() {
            sender.send(State::Btn(Btn::Left))?;
            debug!("Right Click");
        } else if ok_btn.is_clicked() {
            sender.send(State::Btn(Btn::Ok))?;
            debug!("Ok Click");
        } else if ok_btn.is_double_clicked() {
            sender.send(State::Btn(Btn::Exit))?;
            debug!("Exit Click");
        }
        select_btn.reset();
        ok_btn.reset();
        sleep(Duration::from_millis(10))
    }
}
