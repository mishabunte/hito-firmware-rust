pub mod ffi;
pub mod vault;
pub mod bootloader_version;
pub mod firmware_version; 
#[cfg(feature = "minifb")]
mod desktop_storage;

const NONCE_LEN: usize = 7;
const AAD_LEN: usize = 7;
const TAG_LEN: usize = 8;

#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct VaultEncryptedBlock {
  magic: u32,                    // encrypted block magic 0xD0364141
  steps_count: u32,              // pbkdf2 steps count for key generation
  encrypted: [u8; 96],           // encrypted entropy (32-bit) and seed (64-bit)
  nonce: [u8; NONCE_LEN],        // AES CCM nonce
  auth_data: [u8; AAD_LEN],      // AES CCM auth data
  tag: [u8; TAG_LEN],            // AES CCM tag
  crc16_ccitt: u16,              // checksum to check if block itself is valid
}

#[derive(Debug, PartialEq, Clone)]
pub enum VaultError {
    EmptyVault,
    InvalidPassword,
    CryptoError,
    HardwareKeyError,
    InvalidKeyLength,
    BlockNotFound,
    VaultLocked,
    InvalidMnemonicUtf8
}

impl core::fmt::Display for VaultError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            VaultError::EmptyVault => write!(f, "Vault is empty"),
            VaultError::InvalidPassword => write!(f, "Password is invalid"),
            VaultError::CryptoError => write!(f, "Cryptographic error"),
            VaultError::HardwareKeyError => write!(f, "Hardware key error"),
            VaultError::InvalidKeyLength => write!(f, "Invalid key length"),
            VaultError::BlockNotFound => write!(f, "Block not found"),
            VaultError::VaultLocked => write!(f, "Vault is locked"),
            VaultError::InvalidMnemonicUtf8 => write!(f, "Invalid mnemonic"),
        }
    }
}

pub type VaultResult<T> = Result<T, VaultError>;