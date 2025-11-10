use super::super::Display;

use crate::drivers::minifb::simulator_window::*;
use crate::log_info;
extern crate alloc;
use crate::drivers::qr_code::QrCodeWrapper;
use alloc::vec::Vec;

use crate::drivers::qr_code::IMAGE_MAX_WIDTH;

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
    pub fn ili9342_lcd_draw_screen_corners(&mut self) {
      let color = 0x0000;

      self.fill_rect(0, 0, 2, 2, color);
      self.fill_rect(2, 0, 2, 1, color);
      self.fill_rect(0, 2, 1, 2, color);

      self.fill_rect(318, 0, 2, 2, color);
      self.fill_rect(316, 0, 2, 1, color);
      self.fill_rect(319, 2, 1, 2, color);

      self.fill_rect(318, 238, 2, 2, color);
      self.fill_rect(316, 239, 2, 1, color);
      self.fill_rect(319, 236, 1, 2, color);

      self.fill_rect(0, 238, 2, 2, color);
      self.fill_rect(2, 239, 2, 1, color);
      self.fill_rect(0, 236, 1, 2, color);
    }
}

impl Display for DisplayImpl {

    fn init(&mut self) {
        simulator_window_init();
    }

    fn update(&mut self) {
        simulator_window_update();
        self.ili9342_lcd_draw_screen_corners();
    }

    fn draw_qr(&mut self, x: u16, y: u16, data: &str) {
        let code = QrCodeWrapper::new(data);
        let qr_data = code.get_data();
        let qr_width = code.get_width() as u8;

        let qr_density = IMAGE_MAX_WIDTH / qr_width as usize;
        // Convert u8 QR data to u16 RGB565 colors
        let image_width = qr_width as usize * qr_density;
        let mut line_buffer = [0u16; IMAGE_MAX_WIDTH];
        const WHITE: u16 = 0xFFFF; // RGB565 white
        const BLACK: u16 = 0x0000; // RGB565 black

        for k in 0..qr_width {
            let start = k as usize * qr_width as usize;
            let end = (k as usize + 1) * qr_width as usize;

            // Convert u8 values (0 or 1) to u16 RGB565 colors
            for (i, &pixel) in qr_data[start..end].iter().enumerate() {
                for j in 0..qr_density {
                    line_buffer[i * qr_density + j] = if pixel == 0 { WHITE } else { BLACK };
                }
            }
            for y_times in 0..qr_density {
                self.draw_line(
                    k as u16 * qr_density as u16 + y_times as u16 + y as u16,
                    x,
                    x + image_width as u16,
                    &line_buffer[0..image_width as usize]
                );
            }
        }
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
