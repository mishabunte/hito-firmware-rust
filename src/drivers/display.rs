
pub trait Display {
    fn init(&mut self);
    fn draw_rect(&self, x: u16, y: u16, w: u16, h: u16, color: u16);
    fn fill_rect(&mut self, x: u16, y: u16, w: u16, h: u16, color: u16);
    //fn draw_bitmap(&self, x: u16, y: u16, bmp: &Bitmap565);
    fn update(&mut self);
}
