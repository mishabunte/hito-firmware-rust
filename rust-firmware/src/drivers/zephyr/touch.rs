use super::super::Touch;
use super::ffi;

// Import the logging macros
use crate::{log_info};

pub struct TouchImpl {
    is_released: bool,
}

impl TouchImpl {
    pub fn new() -> Self {
        Self {
            is_released: true,
        }
    }
}

impl Touch for TouchImpl {
    fn init(&mut self) {
        unsafe { ffi::ft6336_ctp_power_on() };
        unsafe { ffi::ft6336_ctp_init() };
        log_info!("Touch controller initialized");
    }

    fn is_pressed(&mut self) -> Option<bool> {
        if self.is_released && unsafe { ffi::ft6336_ctp_is_pressed() } {
            self.is_released = false;
            return Some(true);
        } if !self.is_released && unsafe { ffi::ft6336_ctp_is_touch_released() } {
            self.is_released = true;
            return Some(false);
        } else {
          return None;
        }
    }

    fn get_position(&self) -> (u16, u16) {
        unsafe { ffi::ft6336_ctp_read_touch(); }
        (unsafe { ffi::ft6336_ctp_touch_x() }, unsafe { ffi::ft6336_ctp_touch_y() })
    }

    fn has_touch(&self) -> bool {
        unsafe { ffi::ft6336_ctp_has_touch() }
    }
}
