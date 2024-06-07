#![allow(dead_code)]

use std::f32::consts::PI;
use std::marker::PhantomData;
use std::ops::Deref;

use chrono::{Datelike, DateTime, FixedOffset, Local, Timelike};
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::Drawable;
use embedded_graphics::geometry::{Dimensions, Point};
use embedded_graphics::mono_font::ascii::{FONT_5X7, FONT_8X13};
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::Bgr565;
use embedded_graphics::prelude::{Primitive, RgbColor, Transform};
use embedded_graphics::primitives::{
    Circle, Line, PrimitiveStyle, Rectangle,
};
use embedded_graphics::text::{Alignment, Baseline, Text, TextStyleBuilder};

use crate::fs::config::CONFIG;

#[repr(u8)]
pub enum Hand {
    Second,
    Minute,
    Hour,
}

impl Hand {
    pub fn usize(&self) -> usize {
        match self {
            Hand::Second => 2,
            Hand::Minute => 1,
            Hand::Hour => 0,
        }
    }
}
#[inline(always)]
pub fn polar(circle: &Circle, angle: f32, radius_delta: i32) -> Point {
    let radius = circle.diameter as f32 / 2.0 + radius_delta as f32;
    circle.center()
        + Point::new(
            (angle.sin() * radius) as i32,
            -(angle.cos() * radius) as i32,
        )
}

macro_rules! hour_to_angle {
    ($value: expr) => {
        (($value % 12) as f32 / 12.0) * 2.0 * PI
    };
}

macro_rules! min_to_angle {
    ($value: expr) => {
        ($value as f32 / 60.0) * 2.0 * PI
    };
}
pub struct Clock<D: DrawTarget<Color = Bgr565>> {
    width: u32,
    height: u32,
    face: Circle,
    text: [u32; 6],
    size: u32,
    bg_color: Bgr565,
    fg_color: Bgr565,
    date_fixed_offset: i32,
    _d: PhantomData<D>,

}
impl<D: DrawTarget<Color = Bgr565>> Clock<D>
{
    pub fn new(width: u32, height: u32, size: u32, bg: Bgr565, fg: Bgr565) -> Self {
        let face = Circle::with_center(
            Point::new((width / 2) as i32, (height / 2) as i32),
            width.min(height) - 2 * size,
        );
        let date_fixed_offset = match CONFIG.deref() {
            None => 0,
            Some(config) => config.date_fixed_offset,
        };
        Self {
            width,
            height,
            face,
            text: [0; 6],
            size,
            bg_color: bg,
            fg_color: fg,
            date_fixed_offset,
            _d: Default::default(),
        }
    }

    pub fn draw_face(&self, target: &mut D, color: Bgr565) -> anyhow::Result<(), D::Error>
    {
        self.face
            .into_styled(PrimitiveStyle::with_stroke(color, 2))
            .draw(target)?;
        Circle::with_center(self.face.center(), 4)
            .into_styled(PrimitiveStyle::with_fill(color))
            .draw(target)?;
        for i in 0..12 {
            let angle = hour_to_angle!(i);
            let start = polar(&self.face, angle, 0);
            let end = polar(&self.face, angle, -5);
            Line::new(start, end)
                .into_styled(PrimitiveStyle::with_stroke(color, 1))
                .draw(target)?;
            Text::with_text_style(
                format!(
                    "{}",
                    match i {
                        0 => 12,
                        _ => i,
                    }
                )
                .as_str(),
                polar(&self.face, angle, -8) + Point::new(0, 3),
                MonoTextStyle::new(&FONT_5X7, self.fg_color),
                TextStyleBuilder::new()
                    .alignment(Alignment::Center)
                    .baseline(Baseline::Alphabetic)
                    .build(),
            )
            .draw(target)?;
        }

        Ok(())
    }

    pub fn draw_hand(
        &mut self,
        target: &mut D,
        value: u32,
        length_delta: i32,
        hand: Hand,
        color: Bgr565,
    ) -> anyhow::Result<(), D::Error>
    {
        let old_value = self.text[3 + hand.usize()];
        let stroke = match hand {
            Hand::Second => 1,
            Hand::Minute => 2,
            Hand::Hour => 2,
        };
        let angle = match hand {
            Hand::Hour => hour_to_angle!(value),
            _=> min_to_angle!(value)
        };
        let old_angle = match hand {
            Hand::Hour => hour_to_angle!(old_value),
            _=> min_to_angle!(old_value)
        };
        match old_value == value {
            false => {
                let end = polar(&self.face, old_angle, length_delta);
                Line::new(self.face.center(), end)
                    .into_styled(PrimitiveStyle::with_stroke(self.bg_color, stroke))
                    .draw(target)?;
                let end = polar(&self.face, angle, length_delta);
                Line::new(self.face.center(), end)
                    .into_styled(PrimitiveStyle::with_stroke(color, stroke))
                    .draw(target)?;
            }
            true => {
                let end = polar(&self.face, angle, length_delta);
                Line::new(self.face.center(), end)
                    .into_styled(PrimitiveStyle::with_stroke(color, stroke))
                    .draw(target)?;
            }
        }
        Ok(())
    }

    pub fn draw_text(
        &mut self,
        target: &mut D,
        date: &DateTime<FixedOffset>,
    ) -> anyhow::Result<(), D::Error>
    {
        let text_font = MonoTextStyle::new(&FONT_8X13, self.fg_color);
        let date = date.naive_local();
        let (ymd, hms) = self.text.split_at(3);
        if hms != [date.hour(), date.minute(), date.second()] {
            let time_str = format!("{}", date.format("%H:%M:%S"));
            let mut time_text = Text::with_text_style(
                &time_str,
                Point::zero(),
                text_font,
                TextStyleBuilder::new()
                    .alignment(Alignment::Left)
                    .baseline(Baseline::Alphabetic)
                    .build(),
            );
            time_text.translate_mut(Point::new(
                ((self.width - time_text.bounding_box().size.width) / 2) as i32,
                time_text.bounding_box().size.height as i32,
            ));
            let time_text_dimensions = time_text.bounding_box();
            Rectangle::new(time_text_dimensions.top_left, time_text_dimensions.size)
                .into_styled(PrimitiveStyle::with_fill(self.bg_color))
                .draw(target)?;
            time_text.draw(target)?;
        }
        let date_str = format!("{}", date.format("%Y-%m-%d"));
        let mut date_text = Text::with_text_style(
            &date_str,
            Point::zero(),
            text_font,
            TextStyleBuilder::new()
                .alignment(Alignment::Left)
                .baseline(Baseline::Alphabetic)
                .build(),
        );
        date_text.translate_mut(Point::new(
            ((self.width - date_text.bounding_box().size.width) / 2) as i32,
            (self.width - date_text.bounding_box().size.height / 2) as i32,
        ));
        if ymd != [date.year() as u32, date.month(), date.day()] {
            let date_text_dimensions = date_text.bounding_box();
            Rectangle::new(date_text_dimensions.top_left, date_text_dimensions.size)
                .into_styled(PrimitiveStyle::with_fill(self.bg_color))
                .draw(target)?;
        }
        else {
            date_text.draw(target)?;
        }
        Ok(())
    }
    pub fn update(&mut self, display: &mut D) -> anyhow::Result<(), D::Error>
    {

        let date = Local::now().with_timezone(&FixedOffset::east_opt(self.date_fixed_offset).unwrap());
        self.draw_face(display, self.fg_color)?;
        self.draw_hand(display, date.hour(), -20, Hand::Hour, Bgr565::RED)?;
        self.draw_hand(display, date.minute(), -15, Hand::Minute, Bgr565::GREEN)?;
        self.draw_hand(display, date.second(), -10, Hand::Second, Bgr565::BLUE)?;
        self.draw_text(display, &date)?;
        self.text = [date.year() as u32 ,date.month(), date.day(), date.hour(), date.minute(), date.second()];
        Ok(())
    }
}
