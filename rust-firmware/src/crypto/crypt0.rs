use super::ffi;
use core::ffi::c_char;
extern crate alloc;
use alloc::{string::String, vec::Vec};

/// Converts bytes to lowercase hex string (e.g. [0xDE, 0xAD] → "dead")
pub fn bytes_to_hex(bytes: &[u8]) -> String {
    const HEX_CHARS: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        s.push(HEX_CHARS[(b >> 4) as usize] as char);
        s.push(HEX_CHARS[(b & 0x0f) as usize] as char);
    }
    s
}

/// Converts hex string to bytes (returns `None` if invalid)
pub fn hex_to_bytes(hex: &str) -> Option<Vec<u8>> {
    let bytes = hex.as_bytes();
    if bytes.len() % 2 != 0 {
        return None;
    }

    let mut out = Vec::with_capacity(bytes.len() / 2);
    for i in (0..bytes.len()).step_by(2) {
        let high = from_hex_digit(bytes[i])?;
        let low = from_hex_digit(bytes[i + 1])?;
        out.push((high << 4) | low);
    }
    Some(out)
}

fn from_hex_digit(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

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