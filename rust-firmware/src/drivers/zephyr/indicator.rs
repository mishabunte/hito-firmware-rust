use super::super::indicator::{Indicator, LedColor, BlinkSpeed};
use super::ffi;
use core::any::Any;
use crate::{log_info};

pub struct IndicatorImpl {
    initialized: bool,
}

impl IndicatorImpl {
    pub fn new() -> Self {
        Self {
            initialized: false,
        }
    }

    fn led_color_to_c_enum(color: LedColor) -> u8 {
        match color {
            LedColor::Red => 0,   // HITO_LED_RED
            LedColor::Green => 1, // HITO_LED_GREEN 
            LedColor::Blue => 2,  // HITO_LED_BLUE
        }
    }
}

impl Indicator for IndicatorImpl {
    fn init(&mut self) {
        if !self.initialized {
            // Turn off all LEDs initially
            unsafe {
                ffi::hito_led_off();
            }
            self.initialized = true;
            log_info!("LED hardware initialized");
        }
    }

    fn turn_on(&mut self, color: LedColor) {
        let led_id = Self::led_color_to_c_enum(color);
        unsafe {
            // Turn off all LEDs first, then turn on the desired one
            ffi::hito_led_off();
            ffi::hito_led_on(led_id);
        }
        log_info!("LED turned on: {:?}", color);
    }

    fn turn_off(&mut self) {
        unsafe {
            ffi::hito_led_off();
        }
        log_info!("All LEDs turned off");
    }

    fn blink(&mut self, color: LedColor, speed: BlinkSpeed) {
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
