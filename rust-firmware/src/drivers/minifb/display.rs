use super::super::Display;

use crate::drivers::minifb::simulator_window::*;
use crate::log_info;

#[derive(Clone)]
pub struct DisplayImpl {
    brightness: u8,
}

impl DisplayImpl {
    pub fn new() -> Self {
        Self {
            brightness: 100, // Default brightness
        }
    }

    fn rgb565_to_u32(color: u16) -> u32 {
        let r8 = ((color >> 11) & 0x1F) << 3 as u8;
        let g8 = ((color >> 5) & 0x3F)  << 2 as u8;
        let b8 = (color & 0x1F) << 3 as u8;

        ((r8 as u32) << 16) | ((g8 as u32) << 8) | ((b8 as u32) << 0)
    }
}

impl Display for DisplayImpl {

    fn init(&mut self) {
        simulator_window_init();
        log_info!("Minifb Display initialized");
    }

    fn update(&mut self) {
        simulator_window_update();
    }
    
    fn draw_rect(&self, x: u16, y: u16, w: u16, h: u16, rgb565: u16) {
        let color32 = Self::rgb565_to_u32(rgb565);
        simulator_window_draw_rect(x, y, w, h, color32);
    }

    fn fill_rect(&mut self, x: u16, y: u16, w: u16, h: u16, rgb565: u16) {
        let color32 = Self::rgb565_to_u32(rgb565);
        simulator_window_fill_rect(x, y, w, h, color32);
    }

    fn set_brightness(&self, brightness: u8) {
    }

    fn draw_line(&mut self, y: u16, x_start: u16, x_end: u16, pixels: &[u16]) {
        simulator_window_draw_line(y, x_start, x_end, pixels);
    }
}
