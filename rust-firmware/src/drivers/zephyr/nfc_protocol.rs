use crate::drivers::zephyr::ffi;
extern crate alloc;
use alloc::vec::Vec;

pub struct NFCProtocol;

impl NFCProtocol {
    pub const fn new() -> Self {
        NFCProtocol {}
    }
    pub fn init(&self, message: &str) -> Result<(), ()> {
        unsafe {
            if ffi::hito_nfc_start(message.as_ptr()) {
                Ok(())
            } else {
                Err(())
            }
        }
    }
    pub fn stop(&self) {
        unsafe {
            ffi::hito_nfc_stop();
        }
    }
    pub fn get_data(&self) -> Option<Vec<u8>> {
        unsafe {
            if ffi::hito_nfc_has_data() {
                let len = ffi::hito_nfc_data_len() as usize;
                let data_ptr = ffi::hito_nfc_data();
                if !data_ptr.is_null() && len > 0 {
                    let data_slice = core::slice::from_raw_parts(data_ptr, len);
                    let data = data_slice.to_vec();
                    ffi::hito_nfc_data_clear();
                    Some(data)
                } else {
                    None
                }
            } else {
                None
            }
        }
    }
    pub fn get_data_len(&self) -> u32 {
        unsafe { ffi::hito_nfc_data_len() }
    }
}