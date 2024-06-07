use std::net::Ipv4Addr;
use std::sync::mpsc::Receiver;
use std::time::Duration;

use embedded_graphics_core::draw_target::DrawTarget;
use embedded_graphics_core::geometry::OriginDimensions;
use embedded_graphics_core::pixelcolor::Bgr565;
use esp_idf_hal::sys::{heap_caps_print_heap_info, MALLOC_CAP_DEFAULT};
use slint::Color;

use crate::{DISPLAY_HEIGHT, DISPLAY_WIDTH};
use crate::display::backend::EspBackend;
use crate::display::clock::Clock;
use crate::net::net_info;
use crate::utils::{DeviceID, MemInfo};
use crate::utils::state::{Btn, State};

slint::include_modules!();

#[inline(always)]
pub fn show_info(strong: MainWindow) {
    let device_id = DeviceID::get();
    let mut meminfo = MemInfo::new();
    meminfo.fetch();
    let (free, total) = meminfo.kb();
    let mem_info = format!(": {:.2}/{:.2}", free, total);
    let info = match net_info() {
        None => format!("Mac: {}", device_id),
        Some(info) => format!("Mac: {}\nIp: {}\nNet: {}/{}\nDns: {}\nMem: {}", device_id, info.ip,
                              info.subnet.gateway, info.subnet.mask,
                              info.dns.unwrap_or(Ipv4Addr::new(0, 0, 0, 0)), mem_info)
    };
    strong.invoke_set_info_text(info.into(), Color::from_rgb_u8(255, 255, 255), 22, 300);
    strong.invoke_set_visible("info".into(), true);
}


pub fn show_ui<D>(mut display: D, receiver: Receiver<State>) -> anyhow::Result<(), D::Error>
    where D: DrawTarget<Color=Bgr565> + OriginDimensions + 'static  + Clone, D::Error: std::fmt::Debug
{
    slint::platform::set_platform(Box::new(EspBackend::new(display.clone())))
        .expect("backend already initialized");
    let root = MainWindow::new().unwrap();
    let strong = root.clone_strong();
    let timer = slint::Timer::default();
    let mut clock = Clock::new(DISPLAY_WIDTH,
                               DISPLAY_HEIGHT, 17,
                               Bgr565::new(0, 0, 0),
                               Bgr565::new(245, 152, 66));
    let mut clock_update_interval = 0u32;
    let mut show_clock = false;
    timer.start(
        slint::TimerMode::Repeated,
        Duration::from_millis(10),
        move || {
            match receiver.try_recv() {
                Ok(state) => {
                    match state {
                        State::Btn(ref btn) => {
                            match btn {
                                Btn::Right => {
                                    strong.invoke_select_next();
                                }
                                Btn::Left => {
                                    strong.invoke_select_prev();
                                }
                                Btn::Exit => {
                                    show_clock = false;
                                    strong.invoke_set_visible("info".into(), false);
                                    strong.invoke_set_visible("about".into(), false);
                                    strong.invoke_set_visible("debug".into(), false);
                                    strong.invoke_set_visible("carousel".into(), true);
                                }
                                Btn::Ok => {
                                    strong.invoke_set_visible("carousel".into(), false);
                                    match strong.invoke_selected() {
                                        //profile
                                        0 => {
                                            show_info(strong.clone_strong());
                                        }
                                        //home
                                        1 => {
                                            show_clock = true;
                                            clock_update_interval = 0;
                                        }
                                        //debug
                                        2 => {
                                            strong.invoke_set_visible("debug".into(), true);
                                        }
                                        //about
                                        3 => {
                                            strong.invoke_set_visible("about".into(), true);
                                        }
                                        _ => unreachable!()
                                    }
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                    if strong.invoke_get_visible("debug".into()) {
                        strong.invoke_set_debug_text(format!("{}", state).into(), Color::from_rgb_u8(255, 255, 255), 24, 400);
                        unsafe {heap_caps_print_heap_info(MALLOC_CAP_DEFAULT)}
                    }

                }
                Err(_) => {}
            }
            if show_clock && clock_update_interval >= 800 {
                clock.update(&mut display).expect("show clock failed!");
                clock_update_interval = 0;
            }
            clock_update_interval += 10;
        },
    );
    root.run().unwrap();
    Ok(())
}