use super::super::Battery;
use super::ffi;
use crate::log_info;

#[derive(Clone)]
pub struct BatteryImpl {
    initialized: bool,
}

impl BatteryImpl {
    pub fn new() -> Self {
        Self {
            initialized: false,
        }
    }
}

impl Battery for BatteryImpl {
    fn get_level(&self) -> i32 {
        unsafe { ffi::hito_battery_level() }
    }
    fn reboot(&self) {
        unsafe { if !ffi::hito_power_reboot() {
            // If reboot fails, log and halt
            log_info!("Battery reboot failed");
            loop {}
        }
      }
    }
}