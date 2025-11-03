extern crate alloc;
use alloc::string::String;

include!("firmware_version_bindings.rs");
use core::str;

pub struct HitoFirmwareVersion {
  version: String,
}

impl HitoFirmwareVersion {
  pub fn new() -> Self {
      let version = unsafe {
        str::from_utf8(HITO_FIRMWARE_VERSION).unwrap()
      };
      Self { version: String::from(version) }
  }

  pub fn get_version(&self) -> &str {
      &self.version
  }
}