use crate::log_info;
use crate::common::{QR_CODE, ui_get_progress_bar_properties};
use crate::drivers::qr_code::IMAGE_MAX_WIDTH;

pub trait Display {
    fn init(&mut self);
    fn draw_rect(&self, x: u16, y: u16, w: u16, h: u16, color: u16);
    fn fill_rect(&mut self, x: u16, y: u16, w: u16, h: u16, color: u16);
    fn draw_line(&mut self, y: u16, x_start: u16, x_end: u16, pixels: &[u16]);
    fn draw_qr(&mut self, x: u16, y: u16, data: &str) {
      unsafe {
        let code = QR_CODE.as_ref().unwrap();
        let qr_data = code.get_data();
        let qr_width = code.get_width() as u8;

        self.draw_qr_from_buffer(x, y, qr_data, qr_width as usize);
      }
    }

    fn draw_qr_from_buffer(&mut self, x: u16, y: u16, buffer: &[u8], qr_width: usize) {

      log_info!("Drawing QR code at ({}, {}), QR width: {}, IMAGE_MAX_WIDTH: {}", x, y, qr_width, IMAGE_MAX_WIDTH);

        let qr_density = IMAGE_MAX_WIDTH / qr_width as usize;
        // Convert u8 QR data to u16 RGB565 colors
        let image_width = qr_width as usize * qr_density;
        let mut line_buffer = [0u16; IMAGE_MAX_WIDTH];
        const WHITE: u16 = 0xFFFF; // RGB565 white
        const BLACK: u16 = 0x0000; // RGB565 black\
        for k in 0..qr_width {
            let start = k as usize * qr_width as usize;
            let end = (k as usize + 1) * qr_width as usize;
            // Convert u8 values (0 or 1) to u16 RGB565 colors
            for (i, &pixel) in buffer[start..end].iter().enumerate() {
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
    fn update(&mut self);
    fn draw_progress_bar(&mut self,progress: u8) {
        let (x, y, w, h, border_width) = match ui_get_progress_bar_properties() {
            Some(props) => (props.x, props.y, props.w, props.h, props.border_width),
            None => {
                log_info!("Progress bar properties not set");
                return;
            }
        };
        let filled_width = (w as u32 * progress as u32) / 100;
        log_info!("Filled width: {}", filled_width);
        self.draw_rect(x-border_width, y-border_width, w+2*border_width, h+2*border_width, 0x0000); // Black border
        self.fill_rect(x, y, w, h, 0xFFFF); // White background
        self.fill_rect(x, y, filled_width as u16, h, 0xAD55); // Filled part part
    }
    fn set_brightness(&self, brightness: u8);
}