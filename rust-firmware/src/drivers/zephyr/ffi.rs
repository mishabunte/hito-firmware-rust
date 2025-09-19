
#[cfg(feature = "zephyr")]
extern "C" {
    // Initialization functions
    pub fn ili9342_lcd_spi_init();
    //pub fn ili9342_lcd_reset();
    pub fn ili9342_lcd_init();

    pub fn hito_pin_config();

    // Brightness control
    pub fn ili9342_lcd_set_brightness(brightness: u8);

    // LED control
    pub fn ili9342_lcd_led_on();
    pub fn ili9342_lcd_led_off();

    // Power and sleep management
    pub fn ili9342_lcd_sleep_in();
    pub fn ili9342_lcd_sleep_out();
    pub fn ili9342_lcd_display_on();
    pub fn ili9342_lcd_display_off();
    pub fn ili9342_lcd_power_on();
    pub fn ili9342_lcd_power_off();

    // QR code drawing
    pub fn ili9342_lcd_draw_qr(
        qr_data: *const u8,
        x: u16,
        y: u16,
        qr_width: u8,
        image_max_width: u16,
        center: bool,
    ) -> bool;

    // Rectangle operations
    pub fn ili9342_lcd_fill_rect(x: u16, y: u16, w: u16, h: u16, color565: u16);

    // Screen corner drawing
    pub fn ili9342_lcd_draw_screen_corners(all_corners: bool);

    // Bitmap drawing
    pub fn ili9342_lcd_draw_bitmap(
        x: u16,
        y: u16,
        width: u16,
        height: u16,
        bitmap: *const u8,
        bitmap_row_pitch: u16,
    );

    pub fn rust_k_uptime_get() -> u64;

    pub fn ft6336_ctp_power_on();
    pub fn ft6336_ctp_init();
    pub fn ft6336_ctp_has_touch() -> bool;
    pub fn ft6336_ctp_is_pressed() -> bool;
    pub fn ft6336_ctp_is_touch_released() -> bool;

    pub fn ft6336_ctp_read_touch() -> bool;
    pub fn ft6336_ctp_touch_x() -> u16;
    pub fn ft6336_ctp_touch_y() -> u16;

    // LED functions from hito_pin_config.c
    pub fn hito_led_on(led: u8);
    pub fn hito_led_off();
    pub fn hito_led_off_one(led: u8);
    pub fn hito_debug_led_on();
    pub fn hito_debug_led_off();
    pub fn hito_debug_led_blink(n: u8);

    // Logging functions - raw FFI
    pub fn printk(fmt: *const core::ffi::c_char, ...);
}