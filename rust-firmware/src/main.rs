mod hito_firmware;
mod drivers;
mod crypto;
mod platform;

#[cfg(any(feature = "minifb", feature = "zephyr"))]
slint::include_modules!();

use hito_firmware_rust::rust_main;

use hito_firmware::HitoFirmware;
use crate::drivers::{Display, Indicator, LedColor};
use crate::platform::{MyPlatform, Timer, DisplayWrapper};

#[cfg(feature = "minifb")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    rust_main();
}