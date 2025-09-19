use super::ffi;
use core::ffi::c_char;

pub fn crypt0_bech32_encode(data: &[u8], buf: &mut [u8]) -> Result<usize, ()> {
    unsafe {
        let result = ffi::crypt0_bech32_encode(data.as_ptr(), data.len() as i32, buf.as_mut_ptr() as *mut c_char, buf.len() as i32);
        if result > 0 {
            Ok(result as usize)
        } else {
            Err(())
        }
    }
}