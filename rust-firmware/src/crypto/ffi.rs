use core::ffi::{c_char, c_int};

extern "C" {
  pub fn crypt0_bech32_encode(data: *const u8, datalen: c_int, buf: *mut c_char, buflen: c_int) -> c_int;
  pub fn crypt0_bip39_entropy_to_seed_en(
    entropy: *const u8,
    entropy_len: usize,
    seed: *mut u8,
    seed_len: usize
  ) -> c_int;
  pub fn crypt0_crc16_ccitt(data: *const u8, len: usize) -> u16;
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
  ) -> c_int;
  pub fn crypt0_secp256k1_public_key(
    privkey: *const u8,
    privlen: usize,
    pubkey: *mut u8,
    publen: usize
  ) -> i32;
  pub fn crypt0_sha3_keccak(
    data: *const u8,
    data_len: usize,
    out: *mut u8,
    outlen: usize
  ) -> c_int;
  pub fn crypt0_pbkdf2_hmac_sha256(
    steps: u32,
    salt: *const u8,
    salt_len: u32,
    pass: *const u8,
    pass_len: c_int,
    key: *mut u8,
    key_len: u32
  ) -> c_int;
  pub fn crypt0_rng(output: *mut u8, len: usize) -> bool;
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
  ) -> c_int;
  pub fn crypt0_ed25519_public_key(
    privkey: *const u8,
    privlen: usize,
    pubkey: *mut u8,
    publen: usize
  ) -> c_int;
  pub fn crypt0_hmac_sha512(
    key: *const u8,
    key_len: u16,
    msg: *const u8,
    msg_len: u16,
    digest: *mut u8
  ) -> c_int;
  pub fn crypt0_bip39_entropy_to_mnemonic_en(
    entropy: *const u8,
    entropy_len: u8,
    mnemonic: *mut u8,
    mnemonic_len: usize
  ) -> c_int;
  pub fn crypt0_sha256(
    data: *const u8,
    data_len: usize,
    out: *mut u8,
    outlen: usize
  ) -> bool;
  pub fn crypt0_ed25519_sign(
    message: *const u8,
    messagelen: usize,
    privkey: *const u8,
    privlen: usize,
    pubkey: *const u8,
    publen: usize,
    sig: *mut u8,
    siglen: usize
  ) -> c_int;
}
pub const CRYPT0_OK: c_int = 0;
