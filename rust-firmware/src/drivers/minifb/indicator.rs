use super::super::Indicator;
use crate::drivers::minifb::led_desktop::DesktopLedDriver;
use crate::drivers::indicator::{LedColor, BlinkSpeed};

//use crate::drivers::minifb::simulator_window::simulator_window_init;

pub struct IndicatorImpl {
    led_driver: DesktopLedDriver,
}

impl IndicatorImpl {
    pub fn new() -> Self {
        let led_driver = DesktopLedDriver::new();
        Self { led_driver }
    }

}

impl Indicator for IndicatorImpl {
    fn init(&mut self) {
        self.led_driver.init(); 
    }
    
    fn turn_on(&mut self, color: LedColor) {
        match color {
            LedColor::Red => self.led_driver.turn_on(LedColor::Red),
            LedColor::Green => self.led_driver.turn_on(LedColor::Green),
            LedColor::Blue => self.led_driver.turn_on(LedColor::Blue),
        }
    }
    
    fn turn_off(&mut self) {
        self.led_driver.turn_off();
    }

    fn blink(&mut self, color: LedColor, speed: BlinkSpeed) {
        self.led_driver.blink(color, speed);
    }
    
    fn as_any(&self) -> &dyn core::any::Any {
        self.led_driver.as_any()
    }
    
    fn as_any_mut(&mut self) -> &mut dyn core::any::Any {
        self.led_driver.as_any_mut()
    }

  
}
