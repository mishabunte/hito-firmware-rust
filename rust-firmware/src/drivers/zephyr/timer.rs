use super::ffi;

pub struct ZephyrTimer {
    start_time: u64,
}

impl ZephyrTimer {
    pub fn new() -> Self {
        Self { start_time: unsafe { ffi::rust_k_uptime_get() } }
    }

    pub fn get_time() -> u64 {
        unsafe { ffi::rust_k_uptime_get() }
    }
}