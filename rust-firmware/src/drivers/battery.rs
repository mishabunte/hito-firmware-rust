pub trait Battery {
    fn get_level(&self) -> i32; // Battery level as a percentage (0-100)
}