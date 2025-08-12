use super::super::Display;

use crate::drivers::minifb::simulator_window::*;

pub struct DisplayImpl {
}

impl DisplayImpl {
    pub fn new() -> Self {
        Self {
        }
    }

    /*
    fn rgb565_to_color32(color: u16) -> Color32 {
        let r = ((color >> 11) & 0x1F) as u8;
        let g = ((color >> 5) & 0x3F) as u8;
        let b = (color & 0x1F) as u8;
        Color32::from_rgb(r << 3, g << 2, b << 3)
    }
    */

    fn rgb565_to_u32(color: u16) -> u32 {
        let r8 = ((color >> 11) & 0x1F) << 3 as u8;
        let g8 = ((color >> 5) & 0x3F)  << 2 as u8;
        let b8 = (color & 0x1F) << 3 as u8;

        ((r8 as u32) << 16) | ((g8 as u32) << 8) | ((b8 as u32) << 0)
    }

}

impl Display for DisplayImpl {

    fn init(&mut self) {
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

    //fn draw_bitmap(&mut self, x: u16, y: u16, w: u16, h: u16, : u16) {
        //let color32 = Self::rgb565_to_u32(rgb565);
        //simulator_window_draw_bitmap(x, y, w, h, color32);
    //}
}
