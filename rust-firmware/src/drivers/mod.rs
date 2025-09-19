//pub mod display;
//pub mod touch;
//pub mod minifb;
//pub mod indicator;
//

mod display;
pub use display::Display;

mod touch;
pub use touch::Touch;

mod indicator;
pub use indicator::{Indicator, LedColor, BlinkSpeed};

pub mod logging;

#[cfg(feature = "minifb")]
pub use minifb::led_desktop::DesktopLedDriver;

#[cfg(feature = "minifb")]
pub mod minifb;

#[cfg(feature = "minifb")]
pub use minifb::{DisplayImpl, TouchImpl, IndicatorImpl};

#[cfg(feature = "zephyr")]
pub mod zephyr;

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

