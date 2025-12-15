mod hito_firmware;
mod drivers;
mod crypto;
mod platform;
mod vault;
mod firmware_state;
mod common;

// #[cfg(any(feature = "minifb", feature = "zephyr"))]
// slint::include_modules!();

use hito_firmware_rust::rust_main;

pub use hito_firmware_rust::now_us;

#[cfg(feature = "minifb")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    rust_main();
}