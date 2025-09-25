use super::super::Display;
use super::ffi;

#[derive(Clone)]
pub struct DisplayImpl {
    initialized: bool,
    // Pre-allocated buffer for big-endian conversion to eliminate Vec allocations
    conversion_buffer: [u8; 640], // 320 pixels * 2 bytes = max line size
}

impl DisplayImpl {
    pub fn new() -> Self {
        Self {
            initialized: false,
            conversion_buffer: [0u8; 640],
        }
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
        if !self.initialized {
            unsafe {
                ffi::hito_pin_config();
                ffi::ili9342_lcd_init();
            }
            
            self.set_brightness(100);

            self.fill_rect(0, 0, 320, 240, 0xFFFF);
            self.ili9342_lcd_draw_screen_corners();
            self.initialized = true;
        }
    }

    fn update(&mut self) {
        // No update needed for hardware LCD
    }
    
    fn draw_rect(&self, x: u16, y: u16, w: u16, h: u16, rgb565: u16) {
        // For drawing a rectangle outline, we can use fill_rect for now
        // A proper implementation would draw just the outline
        self.fill_rect_internal(x, y, w, 1, rgb565); // top
        self.fill_rect_internal(x, y + h - 1, w, 1, rgb565); // bottom
        self.fill_rect_internal(x, y, 1, h, rgb565); // left
        self.fill_rect_internal(x + w - 1, y, 1, h, rgb565); // right
    }

    fn fill_rect(&mut self, x: u16, y: u16, w: u16, h: u16, rgb565: u16) {
        self.fill_rect_internal(x, y, w, h, rgb565);
    }

    fn set_brightness(&self, brightness: u8) {
        self.set_brightness(brightness);
    }

    fn draw_line(&mut self, y: u16, x_start: u16, x_end: u16, pixels: &[u16]) {
        let width = x_end - x_start;
        let byte_count = (width * 2) as usize;

        // Use pre-allocated buffer instead of Vec allocation
        // Convert RGB565 pixels to big-endian format in-place
        for (i, &px) in pixels.iter().enumerate() {
            let byte_idx = i * 2;
            if byte_idx + 1 < self.conversion_buffer.len() {
                self.conversion_buffer[byte_idx] = (px >> 8) as u8;     // MSB first
                self.conversion_buffer[byte_idx + 1] = (px & 0xFF) as u8; // LSB second
            }
        }

        unsafe {
            ffi::ili9342_lcd_draw_bitmap(
                x_start,
                y,
                width,
                1,
                self.conversion_buffer.as_ptr(),
                (width * 2) as u16,
            );
        }
    }
}

impl DisplayImpl {
    pub fn power_on(&self) {
        unsafe {
            ffi::ili9342_lcd_init();
        }
    }

    fn fill_rect_internal(&self, x: u16, y: u16, w: u16, h: u16, rgb565: u16) {
        unsafe {
            ffi::ili9342_lcd_fill_rect(x, y, w, h, rgb565);
        }
    }

    pub fn set_brightness(&self, brightness: u8) {
        unsafe {
            ffi::ili9342_lcd_set_brightness(brightness);
        }
    }

    pub fn draw_qr(&self, qr_data: &[u8], x: u16, y: u16, qr_width: u8, image_max_width: u16, center: bool) -> bool {
        unsafe {
            ffi::ili9342_lcd_draw_qr(qr_data.as_ptr(), x, y, qr_width, image_max_width, center)
        }
    }

    pub fn draw_bitmap(&self, x: u16, y: u16, width: u16, height: u16, bitmap: *const u8, bitmap_row_pitch: u16) {
        unsafe {
            ffi::ili9342_lcd_draw_bitmap(x, y, width, height, bitmap, bitmap_row_pitch);
        }
    }

    pub fn power_off(&self) {
        unsafe {
            ffi::ili9342_lcd_led_off();
            ffi::ili9342_lcd_display_off();
            ffi::ili9342_lcd_sleep_in();
            ffi::ili9342_lcd_power_off();
        }
    }
}
