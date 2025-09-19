//pub mod display;

pub trait Touch {
    fn init(&mut self);
    fn is_pressed(&mut self) -> Option<bool>;
    fn get_position(&self) -> (u16, u16);
    fn has_touch(&self) -> bool;
}