//pub mod display;
//pub mod touch;
//pub mod minifb;
//pub mod indicator;
//

pub trait Display {
    fn init(&mut self);
    fn draw_rect(&self, x: u16, y: u16, w: u16, h: u16, color: u16);
    fn fill_rect(&mut self, x: u16, y: u16, w: u16, h: u16, color: u16);
    //fn draw_bitmap(&self, x: u16, y: u16, bmp: &Bitmap565);
    fn update(&mut self);
}

pub trait Touch {
    fn init(&mut self);
    fn is_pressed(&self) -> bool;
    fn get_position(&self) -> (u16, u16);
}

pub enum IndicatorRGB { Red, Green, Blue, }

pub trait Indicator {
    fn init(&mut self);
    fn turn_on(&mut self, rgb: IndicatorRGB);
    fn turn_off(&mut self);
}

//mod display;
//pub use display::Display;

//mod touch;
//pub use Touch;

#[cfg(feature = "minifb")]
mod minifb;

#[cfg(feature = "minifb")]
pub use minifb::{DisplayImpl, TouchImpl, IndicatorImpl};

#[cfg(feature = "zephyr")]
mod zephyr;

#[cfg(feature = "zephyr")]
pub use zephyr::{DisplayImpl, TouchImpl, IndicatorImpl};

/*
pub struct Bitmap565<'a> {
    pub width:  u16,
    pub height: u16,
    pub row_pitch: u16,    // bytes per row
    pub data:   &'a [u8],  // length == row_pitch * height
}
*/

