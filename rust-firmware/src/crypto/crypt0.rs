use super::ffi;
use core::ffi::c_char;
extern crate alloc;
use alloc::{string::String, vec::Vec};
use crate::log_info;

const ED25519_PRIVATE_KEY_SIZE: usize = 32;
const ED25519_CHAIN_CODE_SIZE: usize = 32;
const ED25519_DERIVE_DATA_SIZE: usize = 1 + ED25519_PRIVATE_KEY_SIZE + 4;

#[derive(Debug)]
pub enum CryptoError {
    InvalidSeed,
    DerivationError,
    AddressEncodeError,
    CryptoError,
}

impl core::fmt::Display for CryptoError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            CryptoError::InvalidSeed => write!(f, "Invalid seed"),
            CryptoError::DerivationError => write!(f, "Key derivation error"),
            CryptoError::AddressEncodeError => write!(f, "Address encoding error"),
            CryptoError::CryptoError => write!(f, "Cryptographic error"),
        }
    }
}


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

pub fn crypt0_ed25519_derive_secret_index(
    secret: &[u8; ED25519_PRIVATE_KEY_SIZE + ED25519_CHAIN_CODE_SIZE],
    index: u32,
) -> Result<[u8; ED25519_PRIVATE_KEY_SIZE + ED25519_CHAIN_CODE_SIZE], CryptoError> {
    // Only hardened indices allowed
    if index < 0x8000_0000 {
        return Err(CryptoError::InvalidSeed);
    }

    // 0x00 || private_key || index_be
    let mut data = [0u8; ED25519_DERIVE_DATA_SIZE];
    let mut hash = [0u8; 64];

    // data[0] = 0x00;
    data[0] = 0x00;

    // memcpy(data + 1, secret, ED25519_PRIVATE_KEY_SIZE);
    data[1..1 + ED25519_PRIVATE_KEY_SIZE]
        .copy_from_slice(&secret[..ED25519_PRIVATE_KEY_SIZE]);

    // index (big-endian) at the end
    let idx = ED25519_PRIVATE_KEY_SIZE;
    data[idx + 1] = ((index >> 24) & 0xFF) as u8;
    data[idx + 2] = ((index >> 16) & 0xFF) as u8;
    data[idx + 3] = ((index >> 8)  & 0xFF) as u8;
    data[idx + 4] = (index & 0xFF) as u8;

    let res = unsafe {
        ffi::crypt0_hmac_sha512(
            secret[ED25519_PRIVATE_KEY_SIZE..].as_ptr(),
            ED25519_CHAIN_CODE_SIZE as u16,
            data.as_ptr(),
            data.len() as u16,
            hash.as_mut_ptr(),
        )
    };
    if res != ffi::CRYPT0_OK {
        return Err(CryptoError::CryptoError);
    } else {
        Ok(hash)
    }
}