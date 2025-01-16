use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, UNIX_EPOCH};
use embedded_graphics_core::geometry::{OriginDimensions, Point, Size};
use embedded_graphics_core::pixelcolor::{Bgr565, Rgb565};
use embedded_graphics_core::pixelcolor::raw::RawU16;
use embedded_graphics_core::prelude::DrawTarget;
use embedded_graphics_core::primitives::Rectangle;
use slint::platform::software_renderer::{LineBufferProvider, Rgb565Pixel};

use crate::display::DISPLAY_WIDTH;

#[derive(Clone)]
pub struct EspBackend<D: DrawTarget<Color = Bgr565> + OriginDimensions> {
    window: RefCell<Option<Rc<slint::platform::software_renderer::MinimalSoftwareWindow>>>,
    display: RefCell<D>,
    buffer: RefCell<[Rgb565Pixel; DISPLAY_WIDTH as usize]>,
    size: Size
}


impl<D: DrawTarget<Color = Bgr565> + OriginDimensions> EspBackend<D>
{
    pub fn new(display: D) -> Self{
        let size = display.size();
        Self{ window: RefCell::new(None), display: RefCell::new(display), buffer: RefCell::new([Rgb565Pixel(0); DISPLAY_WIDTH as usize]), size }
    }


}

impl <D: DrawTarget<Color = Bgr565>  + OriginDimensions>slint::platform::Platform for EspBackend<D>

{
    fn create_window_adapter(
        &self,
    ) -> Result<Rc<dyn slint::platform::WindowAdapter>, slint::PlatformError> {
        let window = slint::platform::software_renderer::MinimalSoftwareWindow::new(
            slint::platform::software_renderer::RepaintBufferType::ReusedBuffer,
        );
        self.window.replace(Some(window.clone()));
        Ok(window)
    }

    fn run_event_loop(&self) -> anyhow::Result<(), slint::PlatformError> {
        let size = slint::PhysicalSize::new(self.size.width, self.size.height);
        self.window.borrow().as_ref().unwrap().set_size(size);

        loop{
            slint::platform::update_timers_and_animations();
            if let Some(window) = self.window.borrow().clone() {
                if window.has_active_animations() {
                    continue;
                }
                window.draw_if_needed(|renderer| {
                    renderer.render_by_line(self);
                });

            }
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    fn duration_since_start(&self) -> Duration {
        std::time::SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
    }

    fn debug_log(&self, arguments: core::fmt::Arguments) {
        println!("{}", arguments);
    }
}

impl<D: DrawTarget<Color = Bgr565> + OriginDimensions>
LineBufferProvider
for &EspBackend<D>

{
    type TargetPixel = Rgb565Pixel;

    fn process_line(
        &mut self,
        line: usize,
        range: core::ops::Range<usize>,
        render_fn: impl FnOnce(&mut [Rgb565Pixel]),
    ) {
        let mut buffer = self.buffer.borrow_mut();
        let  buffer =  &mut buffer[range.clone()];
        render_fn(buffer);
        self.display.borrow_mut().fill_contiguous(
            &Rectangle::new(Point::new(range.start as _, line as _), Size::new(range.len() as _, 1)),
            buffer.iter().map(|p| Bgr565::from(Rgb565::from(RawU16::new(p.0))).into())
        ).map_err(drop).unwrap();
    }
}