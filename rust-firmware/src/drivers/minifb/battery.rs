use super::super::Battery;
use std::cell::RefCell;
#[cfg(feature = "minifb")]
use rand::Rng;

#[derive(Clone)]
pub struct BatteryImpl {
    initialized: bool,
    rng: RefCell<rand::rngs::ThreadRng>,
}
impl BatteryImpl {
    pub fn new() -> Self {
        let rng = rand::rngs::ThreadRng::default();
        Self {
            initialized: false,
            rng: RefCell::new(rng),
        }
    }
}
impl Battery for BatteryImpl {
    fn get_level(&self) -> i32 {
        // generate a random battery level between 0 and 100 for testing
        rand::Rng::random_range(&mut *self.rng.borrow_mut(), 0..=100)
    }
}