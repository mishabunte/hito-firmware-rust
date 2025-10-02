mod hito_firmware;
mod drivers;
mod crypto;
mod platform;

#[cfg(any(feature = "minifb", feature = "zephyr"))]
slint::include_modules!();

use hito_firmware_rust::rust_main;

#[cfg(feature = "minifb")]
use hito_firmware_rust::init_stack_baseline;

use hito_firmware::HitoFirmware;
use crate::drivers::{Display, Indicator, LedColor};
use crate::platform::{MyPlatform, Timer, DisplayWrapper};

#[cfg(feature = "minifb")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_stack_baseline();
    rust_main();
}