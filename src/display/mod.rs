use embedded_graphics::image::{Image, ImageRaw};
use embedded_graphics::mono_font::ascii::FONT_7X13_BOLD;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::prelude::Transform;
use embedded_graphics::text::{Alignment, Baseline, Text, TextStyleBuilder};
use embedded_graphics_core::Drawable;
use embedded_graphics_core::geometry::Dimensions;
use embedded_graphics_core::pixelcolor::Bgr565;
use embedded_graphics_core::prelude::{DrawTarget, Point, RgbColor};
use embedded_hal::digital::OutputPin;
use embedded_hal::spi::{MODE_0, SpiDevice};
use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::gpio::{Gpio0, IOPin, PinDriver};
use esp_idf_hal::prelude::FromValueType;
use esp_idf_hal::spi::{Dma, SPI2, SpiConfig, SpiDeviceDriver};
use esp_idf_hal::spi::config::DriverConfig;

use st7735r::{Orientation, ST7735};

use crate::{DISPLAY_HEIGHT, DISPLAY_WIDTH};

pub mod clock;
pub mod st7735r;
mod backend;
pub mod ui;
pub fn setup_display(
    spi2: SPI2,
    clk: impl IOPin,
    cs: impl IOPin,
    sdo: impl IOPin,
    rst: impl IOPin,
    dc: impl IOPin,
) -> anyhow::Result<ST7735<impl SpiDevice, impl OutputPin, impl OutputPin>> {
    let sdi = Option::<Gpio0>::None;
    let driver_config = DriverConfig::new()
        .dma(Dma::Auto((DISPLAY_WIDTH * DISPLAY_HEIGHT) as usize));
    let spi_config = SpiConfig::new()
        .baudrate(40.MHz().into())
        .write_only(true)
        .data_mode(MODE_0);
    let spi =
        SpiDeviceDriver::new_single(spi2, clk, sdo, sdi, Some(cs), &driver_config, &spi_config)?;

    let mut display = ST7735::new(
        spi,
        PinDriver::output(dc)?,
        Some(PinDriver::output(rst)?),
        true,
        false,
        DISPLAY_WIDTH,
        DISPLAY_HEIGHT,
    );
    display.hard_reset(&mut FreeRtos).unwrap();
    display.init(&mut FreeRtos).unwrap();
    display.clear(Bgr565::BLACK).unwrap();
    display.set_orientation(&Orientation::Portrait).unwrap();

    display.set_offset(0, 0);

    Ok(display)
}
const IMG_LOGO: &[u8] = include_bytes!("../../ui/image/icon.raw");
pub fn show_logo<D>(display: &mut D, width: u32, height: u32) -> anyhow::Result<(), D::Error>
where
    D: DrawTarget<Color = Bgr565>,
{
    let raw: ImageRaw<Bgr565> = ImageRaw::new(IMG_LOGO, width);
    let im = Image::new(&raw, Point::new(0, 0));
    im.draw(display)?;
    let mut text = Text::with_text_style(
        "Wait for Wifi...",
        Point::zero(),
        MonoTextStyle::new(&FONT_7X13_BOLD, Bgr565::BLUE),
        TextStyleBuilder::new()
            .alignment(Alignment::Left)
            .baseline(Baseline::Alphabetic)
            .build(),
    );
    let text_size = text.bounding_box();
    text.translate_mut(
        Point::new(
            ((width - text_size.size.width) / 2) as i32,
            (height - text_size.size.height / 2) as i32,
        )
    );
    text.draw(display)?;
    Ok(())
}
