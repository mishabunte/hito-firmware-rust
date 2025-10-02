#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub enum hw_unique_key_slot {
  HUK_KEYSLOT_KDR = 0,  // Device Root Key (when HUK_HAS_KMU is not defined)
  HUK_KEYSLOT_MKEK = 2, // Master Key Encryption Key  
  HUK_KEYSLOT_MEXT = 4, // Master External Storage Encryption Key
}

extern "C" {
  pub fn rust_hw_unique_key_is_written();
  pub fn hw_unique_key_derive_key(
    keyslot: hw_unique_key_slot,
    context: *const u8,
    context_len: usize,
    salt: *const u8,
    salt_len: i32,
    derived: *mut u8,
    derived_len: i32
  ) -> i32;
  pub fn crypt0_pbkdf2_hmac_sha256(
    steps: u32,
    salt: *const u8,
    salt_len: u32,
    pass: *const u8,
    pass_len: i32,
    key: *mut u8,
    key_len: u32
  ) -> i32;
  pub fn crypt0_decrypt_aes_ccm(
    encrypted: *const u8,
    encrypted_len: usize,
    key: *const u8,
    key_len: usize,
    nonce: *const u8,
    nonce_len: usize,
    auth_data: *const u8,
    auth_data_len: usize,
    tag: *const u8,
    tag_len: usize,
    decrypted: *mut u8
  ) -> i32;
  pub fn crypt0_bip39_entropy_to_seed_en(
    entropy: *const u8,
    entropy_len: usize,
    seed: *mut u8,
    seed_len: usize
  ) -> i32;
  pub fn crypt0_encrypt_aes_ccm(
    plaintext: *const u8,
    plaintext_len: usize,
    key: *const u8,
    key_len: usize,
    nonce: *const u8,
    nonce_len: usize,
    auth_data: *const u8,
    auth_data_len: usize,
    encrypted: *mut u8,
    tag: *mut u8,
    tag_len: usize
  ) -> i32;
  pub fn hitoVaultHasLegacyPasscode() -> bool;
  pub fn hitoVaultErase(main: bool, backup: bool) -> bool;
  pub fn hitoVaultSaveBlock(
    block_flash: *const super::VaultEncryptedBlock,
    block_ram: *const super::VaultEncryptedBlock
  ) -> bool;
  pub fn hitoVaultSaveToRam(block_ram: *const super::VaultEncryptedBlock);
  pub fn crypt0_rng(output: *mut u8, len: usize) -> bool;
  pub fn crypt0_crc16_ccitt(data: *const u8, len: usize) -> u16;
  pub fn hitoVaultResetCount() -> u32;

  //bridge to bool hitoVaultWriteFlash(const void *offset, const void *data, size_t len)
  pub fn hitoVaultWriteFlash(offset: *const u8, data: *const u8, len: usize) -> bool;
}

pub const HUK_KEYSLOT_MKEK: hw_unique_key_slot = hw_unique_key_slot::HUK_KEYSLOT_MKEK;
pub const CRYPT0_OK: i32 = 0;