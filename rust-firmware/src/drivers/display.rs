pub trait Display {
    fn init(&mut self);
    fn draw_rect(&self, x: u16, y: u16, w: u16, h: u16, color: u16);
    fn fill_rect(&mut self, x: u16, y: u16, w: u16, h: u16, color: u16);
    fn draw_line(&mut self, y: u16, x_start: u16, x_end: u16, pixels: &[u16]);
    fn update(&mut self);
    fn set_brightness(&self, brightness: u8);
}