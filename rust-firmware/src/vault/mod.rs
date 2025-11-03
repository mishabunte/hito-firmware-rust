pub mod ffi;
pub mod vault;
pub mod bootloader_version;
pub mod firmware_version;

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