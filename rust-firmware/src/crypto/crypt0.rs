use super::ffi;
use core::ffi::c_char;
use core::ffi::CStr;
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

/// Get a BIP39 word by its index (0-2047)
pub fn bip39_word_by_index(index: u16) -> Option<&'static str> {
    let idx = index as usize;
    let max = ffi::CRYPT0_BIP39_MNEMONIC_ENGLISH_MAXWORDS as usize;
    if idx >= max {
        return None;
    }

    let ptr = unsafe { ffi::crypt0_bip39_english[idx] };
    if ptr.is_null() {
        return None;
    }

    unsafe { CStr::from_ptr(ptr) }.to_str().ok()
}

/// Get the BIP39 index for a word (case-insensitive)
pub fn bip39_index_by_word(word: &str) -> Option<u16> {
    let word_lc = word.to_ascii_lowercase();
    // Trim trailing null bytes (from fixed-size buffers) and whitespace
    let word_trimmed = word_lc.trim_end_matches('\0').trim();
    let word_bytes = word_trimmed.as_bytes();

    let max = ffi::CRYPT0_BIP39_MNEMONIC_ENGLISH_MAXWORDS as usize;

    for i in 0..max {
        let ptr = unsafe { ffi::crypt0_bip39_english[i] };
        if ptr.is_null() {
            break;
        }

        let w = unsafe { CStr::from_ptr(ptr) }.to_bytes();

        if w == word_bytes {
            return Some(i as u16);
        }
    }

    None
}

/// Convert a mnemonic phrase (space-separated words) to an array of word indices
pub fn mnemonic_to_indices(mnemonic: &str) -> Result<Vec<u16>, CryptoError> {
    let mut indices = Vec::new();
    
    for word in mnemonic.split_whitespace() {
        match bip39_index_by_word(word) {
            Some(idx) => indices.push(idx),
            None => {
                log_info!("mnemonic_to_indices: invalid word '{}' not found in BIP39 wordlist", word);
                return Err(CryptoError::InvalidSeed);
            }
        }
    }

    Ok(indices)
}

/// Convert a mnemonic phrase to entropy bytes
/// Returns (entropy, entropy_len) where entropy_len is 16, 24, or 32
pub fn mnemonic_to_entropy(mnemonic: &str) -> Result<([u8; 32], usize), CryptoError> {
    let indices_unwrapped = mnemonic_to_indices(mnemonic)?;
    mnemonic_indices_to_entropy(indices_unwrapped)
}

/// Convert mnemonic indices to entropy bytes
/// Returns (entropy, entropy_len) where entropy_len is 16, 24, or 32
pub fn mnemonic_indices_to_entropy(indices: Vec<u16>) -> Result<([u8; 32], usize), CryptoError> {
    let word_count = indices.len();
    log_info!("indices: {:?}", indices);

    let phrase = indices.iter().map(|i| bip39_word_by_index(*i).unwrap_or("???")).collect::<Vec<&str>>().join(" ");
    log_info!("mnemonic phrase: {}", phrase);
    
    let entropy_len = match word_count {
        12 => 16,
        18 => 24,
        24 => 32,
        _ => return Err(CryptoError::InvalidSeed),
    };
    
    let mut entropy = [0u8; 32];
    
    let result = unsafe {
        ffi::crypt0_bip39_mnemonic_to_entropy(
            indices.as_ptr(),
            word_count as u16,
            entropy.as_mut_ptr(),
            entropy_len as u16,
        )
    };

    log_info!("mnemonic_to_entropy result: {}", result);
    
    if !result {
        return Err(CryptoError::CryptoError);
    }
    
    Ok((entropy, entropy_len))
}

/// Convert entropy bytes to a mnemonic phrase (as bytes)
/// Returns a 215-byte buffer containing the null-terminated mnemonic string
pub fn entropy_to_mnemonic(entropy: &[u8], entropy_len: usize) -> Result<[u8; 215], CryptoError> {
    let mut mnemonic_buf = [0u8; 215];
    let rc = unsafe {
        ffi::crypt0_bip39_entropy_to_mnemonic_en(
            entropy.as_ptr(),
            entropy_len as u8,
            mnemonic_buf.as_mut_ptr(),
            mnemonic_buf.len()
        )
    };
    if rc < ffi::CRYPT0_OK {
        return Err(CryptoError::CryptoError);
    }
    Ok(mnemonic_buf)
}

/// Generate a BIP39 seed from entropy
/// Returns a 64-byte seed derived from the entropy
pub fn entropy_to_seed(entropy: &[u8], entropy_len: usize) -> Result<[u8; 64], CryptoError> {
    let mut seed_buf = [0u8; 64];
    let rc = unsafe {
        ffi::crypt0_bip39_entropy_to_seed_en(
            entropy.as_ptr(),
            entropy_len as u16,
            seed_buf.as_mut_ptr(),
            64
        )
    };
    if rc != ffi::CRYPT0_OK {
        return Err(CryptoError::CryptoError);
    }
    Ok(seed_buf)
}

/// Generate random entropy bytes
/// Returns a 32-byte buffer with random entropy (only entropy_len bytes are meaningful)
pub fn generate_entropy(entropy_len: usize) -> Result<[u8; 32], CryptoError> {
    let mut entropy_buf = [0u8; 32];
    let rc = unsafe {
        ffi::crypt0_rng(entropy_buf.as_mut_ptr(), entropy_len)
    };
    if !rc {
        return Err(CryptoError::CryptoError);
    }
    Ok(entropy_buf)
}