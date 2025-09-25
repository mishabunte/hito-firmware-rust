use super::super::Battery;
use super::ffi;

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
    fn get_level() -> i32 {
        unsafe { ffi::hito_battery_level() }
    }
}