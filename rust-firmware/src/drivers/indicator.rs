use core::any::Any;

#[cfg(feature = "minifb")]
use std::time::Duration;

// #[derive(Clone, Copy)]
pub trait Indicator {
    fn init(&mut self);
    fn turn_on(&mut self, color: LedColor);
    fn turn_off(&mut self);
    fn blink(&mut self, color: LedColor, speed: BlinkSpeed);
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

#[derive(Clone, Copy, Debug)]
pub enum LedColor {
    Red,
    Green,
    Blue,
}

#[derive(Clone, Copy, Debug)]
pub enum BlinkSpeed {
    Slow,
    Medium,
    Fast,
}

#[cfg(feature = "minifb")]
impl BlinkSpeed {
    pub fn duration(self) -> Duration {
        match self {
            BlinkSpeed::Slow   => Duration::from_millis(800),
            BlinkSpeed::Medium => Duration::from_millis(400),
            BlinkSpeed::Fast   => Duration::from_millis(150),
        }
    }
}
