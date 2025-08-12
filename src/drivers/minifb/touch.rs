use super::super::Touch;

use crate::drivers::minifb::simulator_window::*;

pub struct TouchImpl {
}

impl TouchImpl {

    pub fn new() -> Self {
        Self {
        }
    }
}

impl Touch for TouchImpl {

    fn init(&mut self) {
    }

    fn is_pressed(&self) -> bool {
        simulator_window_has_touch()
    }

    fn get_position(&self) -> (u16, u16) {
        simulator_window_touch_position()
    }

}
