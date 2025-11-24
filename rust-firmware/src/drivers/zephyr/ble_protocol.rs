use crate::drivers::zephyr::ffi;

pub struct BLEProtocol;
extern crate alloc;
use alloc::vec::Vec;

impl BLEProtocol {
    pub const fn new() -> Self {
        BLEProtocol {}
    }
    pub fn init(&self) -> Result<(), ()> {
        unsafe {
            if ffi::hito_ble_init() {
                Ok(())
            } else {
                Err(())
            }
        }
    }
    pub fn stop(&self) {
        unsafe {
            ffi::hito_ble_stop();
        }
    }
    pub fn get_data(&self) -> Option<Vec<u8>> {
        unsafe {
            if ffi::hito_ble_has_data() {
                let len = ffi::hito_ble_datalen() as usize;
                let data_ptr = ffi::hito_ble_data();
                if !data_ptr.is_null() && len > 0 {
                    let data_slice = core::slice::from_raw_parts(data_ptr as *const u8, len);
                    let data = data_slice.to_vec();
                    ffi::hito_ble_data_clear();
                    Some(data)
                } else {
                    None
                }
            } else {
                None
            }
        }
    }
    pub fn get_data_len(&self) -> u16 {
        unsafe { ffi::hito_ble_datalen() }
    }
}