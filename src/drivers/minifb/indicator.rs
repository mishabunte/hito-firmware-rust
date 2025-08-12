use super::super::*;

use crate::drivers::minifb::simulator_window::simulator_window_init;

pub struct IndicatorImpl {
}

impl IndicatorImpl {
    pub fn new() -> Self {
        Self {
        }
    }

}

impl Indicator for IndicatorImpl {

    fn init(&mut self) {
        simulator_window_init();
    }
    fn turn_on(&mut self, rgb: IndicatorRGB) {
    }
    fn turn_off(&mut self) {
    }
}
