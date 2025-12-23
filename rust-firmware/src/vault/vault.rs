use core::cell::Cell;
use core::ptr;
use core::slice;
use crate::crypto;
use crate::crypto::crypt0::bytes_to_hex;
use crate::crypto::crypt0::hex_to_bytes;
use crate::crypto::libcrypt0pro::stellar::StellarKeypair;
use crate::log_info;
use crate::now_us;
use crate::vault::ffi;
use alloc::format;
extern crate alloc;
use crate::vault::{NONCE_LEN, AAD_LEN, TAG_LEN};
use crate::vault::VaultEncryptedBlock;
use crate::vault::bootloader_version::*;
use crate::vault::firmware_version::HitoFirmwareVersion;
use crate::firmware_state::DeviceInfo;

use crate::crypto::libcrypt0pro::stellar::StellarWallet;

type NowFn = fn() -> u64;

#[cfg(feature = "minifb")]
use std::process::Command;
#[cfg(feature = "minifb")]
use std::sync::Once;
#[cfg(feature = "minifb")]
use sha2::{Sha256, Digest};
#[cfg(feature = "minifb")]
static INIT_HARDWARE_ID: Once = Once::new();
#[cfg(feature = "minifb")]
static mut HARDWARE_ID: [u8; 128] = [0; 128];
#[cfg(feature = "minifb")]
static INIT_HUK: Once = Once::new();
#[cfg(feature = "minifb")]
static mut HUK_WRITTEN: bool = false;
#[cfg(feature = "minifb")]
use std::thread;
#[cfg(feature = "minifb")]
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
enum entropy_len_t {
  ENTROPY_LEN_16 = 16,
  ENTROPY_LEN_24 = 24,
  ENTROPY_LEN_32 = 32
}

// Vault header magic constants - Alpha versions
const HITO_VAULT_HEADER_MAGIC_24_WORDS_ALPHA: u32 = 0xD0364141;
const HITO_VAULT_HEADER_MAGIC_12_WORDS_ALPHA: u32 = 0xD0364142;
const HITO_VAULT_HEADER_MAGIC_18_WORDS_ALPHA: u32 = 0xD0364143;

// Vault header magic constants - V1 versions
const HITO_VAULT_HEADER_MAGIC_12_WORDS_V1: u32 = 0xE0364142;
const HITO_VAULT_HEADER_MAGIC_18_WORDS_V1: u32 = 0xE0364143;
const HITO_VAULT_HEADER_MAGIC_24_WORDS_V1: u32 = 0xE0364141;

// Vault step counts
const HITO_VAULT_STEPS_COUNT_FLASH: u32 = 1500;
const HITO_VAULT_STEPS_COUNT_RAM: u32 = 100;
const HITO_VAULT_STEPS_COUNT_FACTORY_SETUP: u32 = 5;

const VAULT_PAGE_SIZE_BLOCKS: usize = 4096 / core::mem::size_of::<VaultEncryptedBlock>();

#[cfg(feature = "zephyr")]
const CONFIG_SRAM_BASE_ADDRESS: usize = 0x20000000; // Typical ARM Cortex-M SRAM base

#[cfg(feature = "zephyr")]
const VAULT_RAM_PAGE: *const VaultEncryptedBlock = (CONFIG_SRAM_BASE_ADDRESS + 0x6f800) as *const VaultEncryptedBlock;
#[cfg(feature = "zephyr")]
const VAULT_MAIN_PAGE: *const VaultEncryptedBlock = 0x2e000 as *const VaultEncryptedBlock;
#[cfg(feature = "zephyr")]
const VAULT_BACKUP_PAGE: *const VaultEncryptedBlock = 0x2f000 as *const VaultEncryptedBlock;


#[cfg(feature = "minifb")]
static mut VAULT_MAIN_STORAGE: [VaultEncryptedBlock; VAULT_PAGE_SIZE_BLOCKS] = [VaultEncryptedBlock {
    magic: 0xffffffff,
    steps_count: 0,
    encrypted: [0; 96],
    nonce: [0; NONCE_LEN],
    auth_data: [0; AAD_LEN],
    tag: [0; TAG_LEN],
    crc16_ccitt: 0,
}; VAULT_PAGE_SIZE_BLOCKS];

#[cfg(feature = "minifb")]
static mut VAULT_BACKUP_STORAGE: [VaultEncryptedBlock; VAULT_PAGE_SIZE_BLOCKS] = [VaultEncryptedBlock {
    magic: 0xffffffff,
    steps_count: 0,
    encrypted: [0; 96],
    nonce: [0; NONCE_LEN],
    auth_data: [0; AAD_LEN],
    tag: [0; TAG_LEN],
    crc16_ccitt: 0,
}; VAULT_PAGE_SIZE_BLOCKS];

#[cfg(feature = "minifb")]
static mut VAULT_RAM_STORAGE: [VaultEncryptedBlock; VAULT_PAGE_SIZE_BLOCKS] = [VaultEncryptedBlock {
    magic: 0xffffffff,
    steps_count: 0,
    encrypted: [0; 96],
    nonce: [0; NONCE_LEN],
    auth_data: [0; AAD_LEN],
    tag: [0; TAG_LEN],
    crc16_ccitt: 0,
}; VAULT_PAGE_SIZE_BLOCKS];

#[cfg(feature = "minifb")]
const VAULT_MAIN_PAGE: *const VaultEncryptedBlock = unsafe { VAULT_MAIN_STORAGE.as_ptr() };
#[cfg(feature = "minifb")]
const VAULT_BACKUP_PAGE: *const VaultEncryptedBlock = unsafe { VAULT_BACKUP_STORAGE.as_ptr() };
#[cfg(feature = "minifb")]
const VAULT_RAM_PAGE: *const VaultEncryptedBlock = unsafe { VAULT_RAM_STORAGE.as_ptr() };

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

pub type VaultResult<T> = Result<T, VaultError>;
use alloc::rc::Rc;

use core::cmp::min;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum UnlockPhase {
    DeriveKeyStep,  // runs HW-derives + a small PBKDF2 batch, contributes to one "step"
    Finalize,       // do the AES-CCM decrypt using derived key
    Done,
}

#[derive(Clone)]
struct UnlockJob {
    block: VaultEncryptedBlock,
    ram_block: Option<VaultEncryptedBlock>,
    pass_buf: [u8; 32],
    pass_len: usize,
    steps_total: u32,

    phase: UnlockPhase,
    step_idx: u32,
    key: [u8; 32],
    derived: [u8; 32],

    hw_iters_left_in_this_step: u32,
    pbkdf2_iters_left_in_this_step: u32,
    now: NowFn,

    // outputs
    result: Option<VaultResult<()>>,
    decrypted: Option<[u8; 96]>,
}

impl UnlockJob {
    fn new(block: &VaultEncryptedBlock, ram_block: Option<VaultEncryptedBlock>, password: &[u8]) -> Self {
        let mut pass = [0u8; 32];
        let len = core::cmp::min(password.len(), 32);
        pass[..len].copy_from_slice(&password[..len]);
        let mut key = [0u8; 32];
        key[..len].copy_from_slice(&pass[..len]);
        let steps_total = if ram_block.is_none() {
            block.steps_count
        } else {
            ram_block.unwrap().steps_count
        };
        log_info!("UnlockJob created: steps_total={}", steps_total);

        Self {
            block: *block,
            ram_block: ram_block,
            pass_buf: pass,
            pass_len: len,
            steps_total: steps_total,
            phase: UnlockPhase::DeriveKeyStep,
            step_idx: 0,
            key,
            derived: [0u8; 32],
            hw_iters_left_in_this_step: 40,
            pbkdf2_iters_left_in_this_step: 10,
            now: now_us,
            result: None,
            decrypted: None,
        }
    }

    fn progress(&self) -> u8 {
        if self.steps_total == 0 { 100 }
        else { ((self.step_idx * 100) / self.steps_total) as u8 }
    }

    /// Advance a small chunk. Returns Some(progress) when a visible change happened.
    fn poll(&mut self) -> VaultResult<Option<u8>> {
      // log_info!("UnlockJob poll: phase={:?}, step_idx={}, hw_iters_left_in_this_step={}, pbkdf2_iters_left_in_this_step={}, total_steps={}",
        //self.phase, self.step_idx, self.hw_iters_left_in_this_step, self.pbkdf2_iters_left_in_this_step, self.steps_total);
    let start = (self.now)();            // monotonic tick (provide this)
    let budget_us = match (self.steps_total) {
        n if n <= HITO_VAULT_STEPS_COUNT_FACTORY_SETUP => 5000,          // 5ms for RAM-based vaults
        n if n <= HITO_VAULT_STEPS_COUNT_RAM => 20000, // 20ms for factory setup vaults
        _ => 50000,                                            // 50ms for flash-based vaults
    };

    let mut last_progress: Option<u8> = None;

    'budget: loop {
        match self.phase {
            UnlockPhase::DeriveKeyStep => {
                // complete the rest of this step
                while self.hw_iters_left_in_this_step > 0 {
                    let hw = match derive_hardware_key(&self.key) {
                        Ok(k) => k,
                        Err(_) => { self.result = Some(Err(VaultError::CryptoError)); self.phase = UnlockPhase::Done; last_progress = Some(100); break 'budget; }
                    };
                    self.derived.copy_from_slice(&hw);
                    self.key.copy_from_slice(&self.derived);
                    self.hw_iters_left_in_this_step -= 1;
                    if (self.now)() - start >= budget_us { 
                      return Ok(last_progress);
                    }
                }

                if self.hw_iters_left_in_this_step == 0 {
                    // do remaining PBKDF2 for this step
                    if self.pbkdf2_iters_left_in_this_step > 0 {
                        let rc = unsafe {
                            crypto::ffi::crypt0_pbkdf2_hmac_sha256(
                                self.pbkdf2_iters_left_in_this_step,
                                self.derived.as_ptr(),
                                self.derived.len() as u32,
                                self.pass_buf.as_ptr(),
                                self.pass_len as i32,
                                self.key.as_mut_ptr(),
                                32,
                            )
                        };
                        if rc != 0 {
                            self.result = Some(Err(VaultError::CryptoError));
                            self.phase = UnlockPhase::Done;
                            last_progress = Some(100);
                            break 'budget;
                        }
                        self.pbkdf2_iters_left_in_this_step = 0;
                    }

                    // step finished
                    self.step_idx += 1;
                    last_progress = Some(self.progress());
                    self.hw_iters_left_in_this_step = 40;
                    self.pbkdf2_iters_left_in_this_step = 10;

                    // if self.steps_total <= HITO_VAULT_STEPS_COUNT_RAM {
                    //   return Ok(last_progress);
                    // }

                    if self.step_idx >= self.steps_total {
                        // log_info!("All steps done, moving to Finalize phase, steps_total={}, current_step={}", self.steps_total, self.step_idx);
                        self.phase = UnlockPhase::Finalize;
                    }

                    if (self.now)() - start >= budget_us { 
                      return Ok(last_progress);
                    }
                } else {
                  // log_info!("Continuing HW derivation in next poll, steps left: {}", self.hw_iters_left_in_this_step);
                }
            }

            UnlockPhase::Finalize => {
                // log_info!("Finalizing vault unlock");
                let to_decode = if (self.ram_block.is_some()) {
                    self.ram_block.unwrap()
                } else {
                    self.block
                };
                match HitoVault::block_decrypt_with_key(&to_decode, &self.key) {
                    Ok(decrypted) => {
                        self.decrypted = Some(decrypted);
                        self.result = Some(Ok(()));
                    }
                    Err(e) => self.result = Some(Err(e)),
                }
                self.phase = UnlockPhase::Done;
                last_progress = Some(100);
                break 'budget;
            }

            UnlockPhase::Done => break 'budget,
        }
    }

    Ok(last_progress)
}
}


#[derive(Clone)]
#[repr(C)]
pub struct HitoVault {
  initialized: bool,
  vaultIsUnlocked: bool,
  entropy: [u8; 32],
  seed: [u8; 64],
  eth_key: [u8; 32],
  near_key: [u8; 32],
  solana_key: [u8; 32],
  solana_addr: [u8; 32],
  eth_addr: [u8; 43],
  near_addr: [u8; 64],
  btc_addr: [u8; 75],
  stellar_public: [u8; 32],
  stellar_secret: [u8; 32],
  mnemonic: [u8; 215],
  entropy_len: entropy_len_t,
  pub unlock_job: Option<UnlockJob>,
}

#[cfg(feature = "minifb")]
fn get_hardware_id() -> VaultResult<[u8; 128]> {
  unsafe {
    INIT_HARDWARE_ID.call_once(|| {
      #[cfg(target_os = "macos")]
      let cmd_output = Command::new("sh")
        .arg("-c")
        .arg("system_profiler SPHardwareDataType | grep 'Hardware UUID' | awk -F ':' '{print $2}'")
        .output();
      
      #[cfg(not(target_os = "macos"))]
      let cmd_output = Command::new("sh")
        .arg("-c")
        .arg("ip link show | grep ether | awk '{print $2}' | head -n 1")
        .output();
      
      if let Ok(output) = cmd_output {
        use std::string::String;
        let id_str = String::from_utf8_lossy(&output.stdout);
        let id_bytes = id_str.trim().as_bytes();
        let copy_len = core::cmp::min(id_bytes.len(), 128);
        HARDWARE_ID[..copy_len].copy_from_slice(&id_bytes[..copy_len]);
      }
    });
    
    Ok(HARDWARE_ID)
  }
}

fn derive_hardware_key(salt: &[u8]) -> VaultResult<[u8; 32]> {
  #[cfg(feature = "zephyr")]
  {
    let derived = unsafe {
      let mut derived = [0u8; 32];
      if (ffi::hw_unique_key_derive_key(
        ffi::HUK_KEYSLOT_MKEK,
        core::ptr::null(),
        0,
        salt.as_ptr(),
        salt.len() as i32,
        derived.as_mut_ptr(),
        derived.len() as i32
      )) != 0 {
        // log_info!("Failed to derive hardware key");
        return Err(VaultError::HardwareKeyError);
      }
      //// log_info!("Derived hardware key: {:?}", derived);
      derived
    };
    Ok(derived)
  }
  
  #[cfg(feature = "minifb")]
  {
    // Get hardware ID
    let hardware_id = get_hardware_id()?;
    
    // SHA256 hash of hardware ID
    let mut hasher = Sha256::new();
    hasher.update(&hardware_id);
    let hash = hasher.finalize();

    let derived = unsafe {
      let mut derived = [0u8; 32];
      if (crypto::ffi::crypt0_pbkdf2_hmac_sha256(
        1,
        hash.as_ptr(),
        32,
        salt.as_ptr(),
        salt.len() as i32,
        derived.as_mut_ptr(),
        derived.len() as u32
      )) != 0 {
        return Err(VaultError::HardwareKeyError);
      }
      derived
    };
    Ok(derived)
  }
}

impl HitoVault {
  pub fn new() -> Self {
    Self { initialized: false, vaultIsUnlocked: false,
           entropy: [0; 32], seed: [0; 64], eth_key: [0; 32], near_key: [0; 32],
           solana_key: [0; 32], solana_addr: [0; 32], eth_addr: [0; 43],
           near_addr: [0; 64], btc_addr: [0; 75], mnemonic: [0; 215],
           stellar_public: [0; 32], stellar_secret: [0; 32],
           entropy_len: entropy_len_t::ENTROPY_LEN_32,
           unlock_job: None
       }
  }

  pub fn init(&mut self) {
    if !self.initialized {
      Self::rust_hw_unique_key_is_written_impl();
      self.initialized = true;
    }
  }
  /// Safely find the last valid block in the vault
  pub fn last_block(&self) -> VaultResult<&VaultEncryptedBlock> {
    // Create a safe slice from the vault main page
    let vault_slice = unsafe {
      slice::from_raw_parts(VAULT_MAIN_PAGE, VAULT_PAGE_SIZE_BLOCKS)
    };
    
    // Find the first block with empty magic (0xffffffff)
    for (i, block) in vault_slice.iter().enumerate() {
      if block.magic == 0xffffffff {
        if i == 0 {
          // Vault is empty, return error
          return Err(VaultError::EmptyVault);
        }
        // Return the previous block (last valid one)
        // log_info!("Found last valid vault block at index {}", i - 1);
        return Ok(&vault_slice[i - 1]);
      }
    }
    // If we reach here, all blocks are filled, return the last one
    vault_slice.last().ok_or(VaultError::BlockNotFound)
  }

  /// Derive encryption key from password using the correct C implementation
  pub fn derive_encryption_key(
    password: &[u8],
    steps_count: u32
  ) -> VaultResult<[u8; 32]> {
    let mut key = [0u8; 32];
    if password.len() == 0 || password.len() > 32 {
      return Err(VaultError::InvalidKeyLength);
    }
    
    // Initialize key with password (zero-padded)
    key[..password.len()].copy_from_slice(password);
    
    let mut progress = 0u8;
    
    // Each step is 100ms time on NRF5340
    for i in 0..steps_count {
      let mut derived = [0u8; 32];
      // 40 iterations of hardware key derivation (~8ms time)
      for _j in 0..40 {
        let hw_key = derive_hardware_key(&key)?;
        derived.copy_from_slice(&hw_key);
        key.copy_from_slice(&derived);
      }
      
      let pbkdf2_result = unsafe {
        crypto::ffi::crypt0_pbkdf2_hmac_sha256(
          10,
          derived.as_ptr(),        // Use current key as salt
          derived.len() as u32,
          password.as_ptr(),
          password.len() as i32,
          key.as_mut_ptr(),    // Output to key
          32
        )
      };
    }
    
    Ok(key)
  }

  pub fn unlock_job_is_none(&self) -> bool {
    self.unlock_job.is_none()
  }

  pub fn reset_unlock_job(&mut self) {
    self.unlock_job = None;
  }

  pub fn block_decrypt(
    block: &VaultEncryptedBlock,
    passcode: &[u8]
  ) -> VaultResult<[u8; 96]> {
    // Derive AES key using the unified key derivation method
    let aes_key = HitoVault::derive_encryption_key(passcode, block.steps_count)?;
    HitoVault::block_decrypt_with_key(block, &aes_key)
  }

  pub fn block_decrypt_with_key(
      block: &VaultEncryptedBlock,
      aes_key: &[u8; 32],
  ) -> VaultResult<[u8; 96]> {
      let mut decrypted = [0u8; 96];
      let result = unsafe {
          crypto::ffi::crypt0_decrypt_aes_ccm(
              block.encrypted.as_ptr(),
              96,
              aes_key.as_ptr(),
              32,
              block.nonce.as_ptr(),
              NONCE_LEN,
              block.auth_data.as_ptr(),
              AAD_LEN,
              block.tag.as_ptr(),
              TAG_LEN,
              decrypted.as_mut_ptr(),
          )
      };
      if result != 96 {
          return Err(VaultError::CryptoError);
      }
      Ok(decrypted)
  }

  /// Check if the vault is empty (no valid blocks)
  pub fn is_empty(&self) -> bool {
    match self.last_block() {
      Err(VaultError::EmptyVault) => true,
      _ => false,
    }
  }

  /// Encrypt a vault block using V1 method
  fn block_encrypt(
    &self,
    block: &mut VaultEncryptedBlock,
    entropy_len: entropy_len_t,
    entropy: &[u8],
    seed: &[u8],
    pass: &[u8],
  ) -> VaultResult<()> {
    // Set magic based on entropy length
    block.magic = match entropy_len {
      entropy_len_t::ENTROPY_LEN_16 => HITO_VAULT_HEADER_MAGIC_12_WORDS_V1,
      entropy_len_t::ENTROPY_LEN_24 => HITO_VAULT_HEADER_MAGIC_18_WORDS_V1,
      _ => HITO_VAULT_HEADER_MAGIC_24_WORDS_V1,
    };

    // Validate step count based on reset count
    let reset_count = 0;
    let valid_steps = if reset_count != 0 {
      block.steps_count == HITO_VAULT_STEPS_COUNT_FLASH || 
      block.steps_count == HITO_VAULT_STEPS_COUNT_RAM
    } else {
      block.steps_count == 5
    };

    // Copy entropy and seed directly to encrypted buffer (will be encrypted in-place)
    block.encrypted[..32].copy_from_slice(&entropy[..32]);
    block.encrypted[32..].copy_from_slice(&seed[..64]);

    // Generate encryption key
    let aes_key = HitoVault::derive_encryption_key(pass, block.steps_count)?;

    // Generate random nonce and auth_data
    let nonce_result = unsafe { crypto::ffi::crypt0_rng(block.nonce.as_mut_ptr(), NONCE_LEN) };
    if !nonce_result {
      return Err(VaultError::CryptoError);
    }

    let auth_data_result = unsafe { crypto::ffi::crypt0_rng(block.auth_data.as_mut_ptr(), AAD_LEN) };
    if !auth_data_result {
      return Err(VaultError::CryptoError);
    }

    // Encrypt in-place using AES-CCM
    unsafe {
      crypto::ffi::crypt0_encrypt_aes_ccm(
        block.encrypted.as_ptr(),
        96,
        aes_key.as_ptr(),
        32,
        block.nonce.as_ptr(),
        NONCE_LEN,
        block.auth_data.as_ptr(),
        AAD_LEN,
        block.encrypted.as_mut_ptr(), // In-place encryption
        block.tag.as_mut_ptr(),
        TAG_LEN
      );
    }

    // Calculate CRC16 CCITT over the block excluding checksum and pin attempt bits
    let crc_size = core::mem::size_of::<VaultEncryptedBlock>() - 
                   core::mem::size_of::<u16>(); // crc16_ccitt
    
    block.crc16_ccitt = unsafe {
      crypto::ffi::crypt0_crc16_ccitt(block as *const VaultEncryptedBlock as *const u8, crc_size)
    };

    Ok(())
  }

  fn erase_ram_block() {
    unsafe {
      if !VAULT_RAM_PAGE.is_null() {
        let ram_slice = slice::from_raw_parts_mut(
          VAULT_RAM_PAGE as *mut VaultEncryptedBlock,
          VAULT_PAGE_SIZE_BLOCKS
        );
        for block in ram_slice.iter_mut() {
          block.magic = 0xffffffff;
        }
      }
    }
  }

  fn is_block_checksum_valid(block: Option<VaultEncryptedBlock>) -> bool {
    let block = match block {
      Some(b) => b,
      None => return false,
    };
    let crc_size = core::mem::size_of::<VaultEncryptedBlock>() - 
                   core::mem::size_of::<u16>(); // crc16_ccitt
    let calculated_crc = unsafe {
      crypto::ffi::crypt0_crc16_ccitt(&block as *const VaultEncryptedBlock as *const u8, crc_size)
    };
    calculated_crc == block.crc16_ccitt
  }

  pub fn start_unlock(&mut self, password: &[u8]) -> VaultResult<()> {
      log_info!("Starting vault unlock job");
      let block = self.last_block()?; // copies the block into RAM
      if password.is_empty() || password.len() > 32 {
          return Err(VaultError::InvalidKeyLength);
      }
      let mut ramBlock = unsafe {
          if VAULT_RAM_PAGE.is_null() {
              None
          } else {
              Some(*VAULT_RAM_PAGE)
          }
      };
      if !Self::is_block_checksum_valid(ramBlock) {
          ramBlock = None;
      } else if (block.magic == HITO_VAULT_HEADER_MAGIC_12_WORDS_ALPHA) ||
                (block.magic == HITO_VAULT_HEADER_MAGIC_18_WORDS_ALPHA) ||
                (block.magic == HITO_VAULT_HEADER_MAGIC_24_WORDS_ALPHA)
      {
          HitoVault::erase_ram_block();
          ramBlock = None;
      }
      self.unlock_job = Some(UnlockJob::new(&block, ramBlock, password));
      Ok(())
  }

  pub fn poll_unlock(&mut self) -> Result<Option<u8>, VaultError> {
      let Some(job) = self.unlock_job.as_mut() else {
          // log_info!("No unlock job in progress");
          return Ok(None);
      };

      if let Some(p) = job.poll().unwrap_or(None) {

          log_info!("Current phase: {:?}, progress: {}%", job.phase, p);
          // If the job reached Done, take it and commit results while holding &mut self.
          if matches!(job.phase, UnlockPhase::Done) {
              let job = self.unlock_job.take().unwrap();
              let result = job.result.unwrap_or(Err(VaultError::CryptoError));
              match result {
                  Ok(()) => {
                      log_info!("Vault unlocked successfully");
                      // Commit decrypted bytes into the vault state
                      let decrypted = job.decrypted.expect("decrypted present on Ok");
                      match (job.block.magic) {
                          HITO_VAULT_HEADER_MAGIC_12_WORDS_V1 => {
                              self.entropy_len = entropy_len_t::ENTROPY_LEN_16;
                          }
                          HITO_VAULT_HEADER_MAGIC_12_WORDS_ALPHA => {
                              self.entropy_len = entropy_len_t::ENTROPY_LEN_16;
                          }
                          HITO_VAULT_HEADER_MAGIC_18_WORDS_V1 => {
                              self.entropy_len = entropy_len_t::ENTROPY_LEN_24;
                          }
                          HITO_VAULT_HEADER_MAGIC_18_WORDS_ALPHA => {
                              self.entropy_len = entropy_len_t::ENTROPY_LEN_24;
                          }
                          HITO_VAULT_HEADER_MAGIC_24_WORDS_V1 => {
                              self.entropy_len = entropy_len_t::ENTROPY_LEN_32;
                          }
                          HITO_VAULT_HEADER_MAGIC_24_WORDS_ALPHA => {
                              self.entropy_len = entropy_len_t::ENTROPY_LEN_32;
                          }
                          _ => return Err(VaultError::CryptoError),
                      }
                      let entropy_len = self.entropy_len as usize;
                      if entropy_len == 0 || entropy_len > 32 {
                          return Err(VaultError::InvalidKeyLength);
                      }
                      self.entropy[..entropy_len].copy_from_slice(&decrypted[..entropy_len]);
                      self.seed.copy_from_slice(&decrypted[32..]);
                      if job.ram_block.is_none() {
                        let blockRam = VAULT_RAM_PAGE as *mut VaultEncryptedBlock;
                        unsafe {
                          if !blockRam.is_null() {
                            (*blockRam).steps_count = if self.is_factory_setup() {
                              HITO_VAULT_STEPS_COUNT_FACTORY_SETUP
                            } else {
                              HITO_VAULT_STEPS_COUNT_RAM
                            };
                          }
                          self.block_encrypt(blockRam.as_mut().unwrap(), self.entropy_len, &self.entropy, &self.seed, &job.pass_buf[..job.pass_len])?;
                        }
                      }
                      self.save_vault_data()?;
                      self.vaultIsUnlocked = true;
                  }
                  Err(e) => {
                    log_info!("Unlock failed with error: {:?}", e);
                    self.vaultIsUnlocked = false;
                    return Err(e)
                  }
              }
          } else {
            //log_info!("Nothing to commit yet, unlock job still in progress");
          }
          return Ok(Some(p));
      } else {
          log_info!("No progress update from unlock job");
      }

      Ok(None)
  }

  fn is_factory_setup(&self) -> bool {
    self.get_reset_count() == 0
  }


  pub fn cancel_unlock(&mut self) {
      self.unlock_job = None;
      // self.set_progress(0);
  }
  /// Save vault blocks to flash and RAM (matches C hitoVaultSaveBlock)
  fn vault_save_block(
    &self,
    block_flash: &VaultEncryptedBlock,
    block_ram: Option<&VaultEncryptedBlock>
  ) -> VaultResult<()> {
    // Find next available slot in main vault page
    let vault_slice = unsafe {
      slice::from_raw_parts(VAULT_MAIN_PAGE, VAULT_PAGE_SIZE_BLOCKS)
    };
    
    let mut slot_index = VAULT_PAGE_SIZE_BLOCKS;
    for (i, block) in vault_slice.iter().enumerate() {
      if block.magic == 0xffffffff {
        slot_index = i;
        break;
      }
    }
    
    // Write flash block to main vault
    unsafe {
      let main_addr = VAULT_MAIN_PAGE.add(slot_index);
      if !Self::write_flash(
        main_addr,
        block_flash,
        core::mem::size_of::<VaultEncryptedBlock>()
      ) {
        return Err(VaultError::CryptoError);
      }
      
      // Write backup block to backup vault
      let backup_addr = VAULT_BACKUP_PAGE.add(slot_index);
      if !Self::write_flash(
        backup_addr,
        block_flash,
        core::mem::size_of::<VaultEncryptedBlock>()
      ) {
        return Err(VaultError::CryptoError);
      }
      
      // Write RAM block if provided
      if let Some(ram_block) = block_ram {
        // TODO: Implement RAM storage
        Self::write_ram_block(VAULT_RAM_PAGE, ram_block, core::mem::size_of::<VaultEncryptedBlock>());
      }
    }
    
    Ok(())
  }

  pub fn get_reset_count(&self) -> i32 {
    let seal = HitoSealBlock::get();
    if seal.is_none() { -1 } else {
      let mut factory_reset_count = 0;
      for i in 0..32 {
          if (seal.as_ref().unwrap().reset_counter_bits & (1 << i)) == 0 {
              factory_reset_count += 1;
          }
      }
      factory_reset_count
    }
  }

  pub fn get_firmware_version(&self) -> VaultResult<alloc::string::String> {
    let version = HitoFirmwareVersion::new();
    Ok(alloc::string::String::from(version.get_version()))
  }

  pub fn get_bootloader_version(&self) -> VaultResult<alloc::string::String> {
    let version = HitoBootloaderVersion::get();
    if version.is_none() {
      return Ok(alloc::string::String::from("legacy"));
    }
    let version_str = format!("{}.{}.{}'{}",
      version.as_ref().unwrap().major,
      version.as_ref().unwrap().minor,
      version.as_ref().unwrap().revision,
      version.as_ref().unwrap().build
    );
    Ok(alloc::string::String::from(version_str))
  }

  pub fn get_serial_number(&self) -> VaultResult<alloc::string::String> {
    if !self.is_unlocked() {
      return Err(VaultError::VaultLocked);
    }
    let seal = HitoSealBlock::get();

    if seal.is_none() {
      return Err(VaultError::CryptoError);
    }
    // log_info!("Deriving hardware key");
    let hardware_key_result = derive_hardware_key(&seal.as_ref().unwrap().serial_number_salt);
    if hardware_key_result.is_err() {
      return Err(VaultError::HardwareKeyError);
    }
    let hardware_key = hardware_key_result.unwrap();

    // log_info!("Generating public key from hardware key");
    let mut public_key = [0u8; 65];
    let decrypt_result = unsafe {
      crypto::ffi::crypt0_secp256k1_public_key(hardware_key.as_ptr(), hardware_key.len(), public_key.as_mut_ptr(), public_key.len())
    };
    // log_info!("Public key generated: {:?}", public_key);
    
    if decrypt_result != crypto::ffi::CRYPT0_OK {
      return Err(VaultError::CryptoError);
    }

    // log_info!("Calculating SHA3-256 of public key");
    let sha3 = unsafe {
      let mut sha3 = [0u8; 32];
      crypto::ffi::crypt0_sha3_keccak(
        public_key.as_ptr().add(1), // skip 0x04 prefix
        64,
        sha3.as_mut_ptr(),
        sha3.len()
      );
      sha3
    };
    // log_info!("SHA3-256 calculated: {:?}", sha3);
    let hex_str = crypto::crypt0::bytes_to_hex(&sha3[..10]);
    Ok(hex_str)
  }

  pub fn get_device_info(&self) -> VaultResult<DeviceInfo> {
    // log_info!("Getting device info");
    if !self.is_unlocked() {
      return Err(VaultError::VaultLocked);
    }
    // log_info!("Device info retrieved successfully");
    Ok(DeviceInfo::new(self.get_firmware_version()?,
      self.get_bootloader_version()?,
      self.get_serial_number()?,
      self.get_reset_count()
    ))
  }
  
  pub fn get_stellar_address(&self) -> VaultResult<alloc::string::String> {
    if !self.is_unlocked() {
      return Err(VaultError::VaultLocked);
    }
    Ok(StellarWallet::encode_stellar_address(&self.stellar_public).unwrap())
  }

  /// Save vault data with new passcode
  fn vault_save(
    &self,
    entropy: &[u8],
    entropy_len: usize,
    seed: &[u8],
    new_pass: &[u8]
  ) -> VaultResult<()> {
    let mut block_flash = VaultEncryptedBlock {
      magic: 0,
      steps_count: HITO_VAULT_STEPS_COUNT_FLASH,
      encrypted: [0; 96],
      nonce: [0; NONCE_LEN],
      auth_data: [0; AAD_LEN],
      tag: [0; TAG_LEN],
      crc16_ccitt: 0,
    };

    let mut block_ram = VaultEncryptedBlock {
      magic: 0,
      steps_count: HITO_VAULT_STEPS_COUNT_RAM,
      encrypted: [0; 96],
      nonce: [0; NONCE_LEN],
      auth_data: [0; AAD_LEN],
      tag: [0; TAG_LEN],
      crc16_ccitt: 0,
    };

    let entropy_enum = match entropy_len {
      16 => entropy_len_t::ENTROPY_LEN_16,
      24 => entropy_len_t::ENTROPY_LEN_24,
      32 => entropy_len_t::ENTROPY_LEN_32,
      _ => return Err(VaultError::InvalidKeyLength),
    };

    // Encrypt flash block
    self.block_encrypt(&mut block_flash, entropy_enum.clone(), entropy, seed, new_pass)?;

    // Encrypt RAM block
    self.block_encrypt(&mut block_ram, entropy_enum, entropy, seed, new_pass)?;

    // Save both blocks using our Rust implementation
    self.vault_save_block(&block_flash, Some(&block_ram))?;

    Ok(())
  }

  pub fn set_entropy(&mut self, entropy: &[u8], entropy_len: usize) {
    self.entropy[..entropy_len].copy_from_slice(&entropy[..entropy_len]);
    self.entropy_len = match entropy_len {
      16 => entropy_len_t::ENTROPY_LEN_16,
      24 => entropy_len_t::ENTROPY_LEN_24,
      32 => entropy_len_t::ENTROPY_LEN_32,
      _ => entropy_len_t::ENTROPY_LEN_32, // Default to 32 if invalid
    };
  }

  fn save_mnemonic(&mut self) -> VaultResult<()> {
    // log_info!("Saving mnemonic");
    let result = unsafe {
      let mut mnemonic_buf = [0u8; 215];
      let len = mnemonic_buf.len();
      let rc = crypto::ffi::crypt0_bip39_entropy_to_mnemonic_en(
        self.entropy.as_ptr(),
        self.entropy_len as u8,
        mnemonic_buf.as_mut_ptr(),
        mnemonic_buf.len()
      );  
      // log_info!("Converted entropy to mnemonic, len : {}", rc);
      if rc < crypto::ffi::CRYPT0_OK {
        return Err(VaultError::CryptoError);
      }
      self.mnemonic[..len].copy_from_slice(&mnemonic_buf[..len]);
      // log_info!("Mnemonic saved: {:?}", &self.mnemonic[..len]);
    };
    Ok(())
  }

  pub fn generate_mnemonic(&mut self) -> VaultResult<()> {
    let entropy_len = 32;
    // log_info!("Generating new mnemonic with entropy length: {}", entropy_len);
    let entropy_enum = match entropy_len {
      16 => entropy_len_t::ENTROPY_LEN_16,
      24 => entropy_len_t::ENTROPY_LEN_24,
      32 => entropy_len_t::ENTROPY_LEN_32,
      _ => return Err(VaultError::InvalidKeyLength),
    };
    
    // Generate random entropy
    let entropy_result = unsafe {
      let mut entropy_buf = [0u8; 32];
      let rc = crypto::ffi::crypt0_rng(entropy_buf.as_mut_ptr(), entropy_len as usize);
      if !rc {
        return Err(VaultError::CryptoError);
      }
      self.entropy[..entropy_len].copy_from_slice(&entropy_buf[..entropy_len]);
      self.entropy_len = entropy_enum;
      // log_info!("Generated random entropy: {:?}", &self.entropy[..entropy_len]);
    };
    
    // Save mnemonic
    self.save_mnemonic()?;
    
    // Generate seed from entropy
    let seed_result = unsafe {
      let mut seed_buf = [0u8; 64];
      let rc = crypto::ffi::crypt0_bip39_entropy_to_seed_en(
        self.entropy.as_ptr(),
        self.entropy_len as u16,
        seed_buf.as_mut_ptr(),
        64
      );
      if rc != crypto::ffi::CRYPT0_OK {
        return Err(VaultError::CryptoError);
      }
      self.seed.copy_from_slice(&seed_buf);
      // log_info!("Generated seed from entropy: {:?}", &self.seed[..64]);
    };
    
    Ok(())
  }

  fn save_stellar(&mut self) -> VaultResult<()> {
    // log_info!("Saving Stellar secret key");
    let wallet = StellarWallet::from_seed(self.seed);
    let keypair = wallet.derive_keypair(0).unwrap();
    self.stellar_secret[..keypair.secret_key.len()].copy_from_slice(&keypair.secret_key);
    self.stellar_public[..keypair.public_key.len()].copy_from_slice(&keypair.public_key);
    //log_info!("Stellar keys saved: public={:?}, secret={:?}", bytes_to_hex(&self.stellar_public), bytes_to_hex(&self.stellar_secret));
    Ok(())
  }

  pub fn get_stellar_keypair(&self) -> VaultResult<StellarKeypair> {
    if !self.is_unlocked() {
      return Err(VaultError::VaultLocked);
    }
    Ok(StellarKeypair {
      public_key: self.stellar_public,
      secret_key: self.stellar_secret,
    })
  }

  fn save_vault_data(&mut self) -> VaultResult<()> {
    // log_info!("Saving vault data");
    self.save_mnemonic()?;
    self.save_stellar()?;
    // log_info!("Vault data saved successfully");
    Ok(())
  }


  pub fn get_mnemonic(&self) -> VaultResult<alloc::string::String> {
      // log_info!("Getting mnemonic");
      if !self.is_unlocked() {
          // log_info!("Vault is locked, cannot get mnemonic");
          return Err(VaultError::VaultLocked);
      }

      let mnemonic_str = match core::str::from_utf8(&self.mnemonic) {
          Ok(s) => alloc::string::String::from(s),
          Err(_) => return Err(VaultError::InvalidMnemonicUtf8),
      };
      Ok(mnemonic_str)
  }

  pub fn get_mnemonic_len(&self) -> usize {
    match self.entropy_len {
      entropy_len_t::ENTROPY_LEN_16 => 12,
      entropy_len_t::ENTROPY_LEN_24 => 18,
      entropy_len_t::ENTROPY_LEN_32 => 24,
    }
  }

  /// Set a new passcode for the vault
  pub fn set_passcode(
    &mut self,
    new_pass: &[u8]
  ) -> VaultResult<()> {
    // If vault is empty but we have entropy, generate seed from entropy
    if self.is_empty() && self.entropy_len as usize != 0 {
      // log_info!("Generating seed from entropy");
      let result = unsafe {
        crypto::ffi::crypt0_bip39_entropy_to_seed_en(
          self.entropy.as_ptr(),
          self.entropy_len as u16,
          self.seed.as_mut_ptr(),
          64
        )
      };

      log_info!("Generated seed from entropy: {:?}", &self.seed[..64]);
      
      if result != crypto::ffi::CRYPT0_OK {
        return Err(VaultError::CryptoError);
      }
      
      //self.vaultIsUnlocked = true;
    }

    // Check if vault is unlocked
    // if !self.vaultIsUnlocked {
    //   return Err(VaultError::InvalidPassword);
    // }

    // Save entropy with new passcode
    self.vault_save(
      &self.entropy,
      self.entropy_len as usize,
      &self.seed,
      new_pass
    )?;

    log_info!("Vault saved with new passcode");

    Ok(())
  }

  /// Attempt to unlock the vault with the given password
  pub fn unlock_with_password(&mut self, password: &[u8]) -> VaultResult<()> {
    // Get the last block
    let block = self.last_block()?;

    let mut ram_block = unsafe {
        if VAULT_RAM_PAGE.is_null() {
            None
        } else {
            Some(*VAULT_RAM_PAGE)
        }
    };

    if !Self::is_block_checksum_valid(ram_block) {
        ram_block = None;
    } else if (block.magic == HITO_VAULT_HEADER_MAGIC_12_WORDS_ALPHA) ||
              (block.magic == HITO_VAULT_HEADER_MAGIC_18_WORDS_ALPHA) ||
              (block.magic == HITO_VAULT_HEADER_MAGIC_24_WORDS_ALPHA)
    {
        HitoVault::erase_ram_block();
        ram_block = None;
    }

    let to_decode = if let Some(ram) = ram_block {
        ram
    } else {
        *block
    };

    // Decrypt the block
    let decrypted = HitoVault::block_decrypt(&to_decode, password)?;
    // Parse the decrypted data (entropy + seed) and store in vault
    // The first 32 bytes are entropy, next 64 bytes are seed
    match block.magic {
        HITO_VAULT_HEADER_MAGIC_12_WORDS_V1 => {
            self.entropy_len = entropy_len_t::ENTROPY_LEN_16;
        }
        HITO_VAULT_HEADER_MAGIC_12_WORDS_ALPHA => {
            self.entropy_len = entropy_len_t::ENTROPY_LEN_16;
        }
        HITO_VAULT_HEADER_MAGIC_18_WORDS_V1 => {
            self.entropy_len = entropy_len_t::ENTROPY_LEN_24;
        }
        HITO_VAULT_HEADER_MAGIC_18_WORDS_ALPHA => {
            self.entropy_len = entropy_len_t::ENTROPY_LEN_24;
        }
        HITO_VAULT_HEADER_MAGIC_24_WORDS_V1 => {
            self.entropy_len = entropy_len_t::ENTROPY_LEN_32;
        }
        HITO_VAULT_HEADER_MAGIC_24_WORDS_ALPHA => {
            self.entropy_len = entropy_len_t::ENTROPY_LEN_32;
        }
        _ => return Err(VaultError::CryptoError),
    }
    let entropy_len = self.entropy_len as usize;
    if entropy_len == 0 || entropy_len > 32 {
        return Err(VaultError::InvalidKeyLength);
    }

    self.entropy[..entropy_len].copy_from_slice(&decrypted[..entropy_len]);
    self.seed.copy_from_slice(&decrypted[entropy_len..(entropy_len + 64)]);
    log_info!("Vault decrypted successfully, decrypted: {:?}, entropy_len: {}", &decrypted[..(entropy_len + 64)], entropy_len);

    if ram_block.is_none() {
      let blockRam = VAULT_RAM_PAGE as *mut VaultEncryptedBlock;
      unsafe {
        if !blockRam.is_null() {
          (*blockRam).steps_count = if self.is_factory_setup() {
            HITO_VAULT_STEPS_COUNT_FACTORY_SETUP
          } else {
            HITO_VAULT_STEPS_COUNT_RAM
          };
        }
        self.block_encrypt(blockRam.as_mut().unwrap(), self.entropy_len, &self.entropy, &self.seed, password)?;
      }
    }
    
    self.save_vault_data()?;
    //log_info!("Vault unlocked successfully");
    self.vaultIsUnlocked = true;
    Ok(())
  }

  pub fn is_unlocked(&self) -> bool {
    self.vaultIsUnlocked
  }

  #[cfg(feature = "minifb")]
  fn hw_unique_key_is_written_sim() -> bool {
    unsafe {
      INIT_HUK.call_once(|| {
        // In simulation, we simulate the HUK as being written
        // This mimics the behavior of the real hardware
        HUK_WRITTEN = true;
      });
      HUK_WRITTEN
    }
  }

  #[cfg(feature = "minifb")]
  fn hw_unique_key_write_random_sim() {
    unsafe {
      HUK_WRITTEN = true;
    }
  }

  fn rust_hw_unique_key_is_written_impl() {
    #[cfg(feature = "zephyr")]
    {
      unsafe { ffi::rust_hw_unique_key_is_written() };
    }
    
    #[cfg(feature = "minifb")]
    {
      if !Self::hw_unique_key_is_written_sim() {
        Self::hw_unique_key_write_random_sim();
      }
    }
  }

  /// Erase the vault (main and/or backup pages)
  /// Returns true on success
  pub fn erase(&mut self, main: bool, backup: bool) -> bool {
    // Erase RAM block
    Self::erase_ram_block();

    #[cfg(feature = "zephyr")]
    {
      const VAULT_PAGE_SIZE_BYTES: usize = 4096;

      // Increment reset counter if erasing both main and backup
      if main && backup {
        if let Some(seal) = HitoSealBlock::get() {
          let mut seal_update = seal;
          seal_update.reset_counter_bits = seal_update.reset_counter_bits << 1;
          
          const HITO_BOOTLOADER_SEAL_ADDRESS: u32 = 0x100000 
            - core::mem::size_of::<HitoSealBlock>() as u32 
            - core::mem::size_of::<HitoBootloaderVersionBlock>() as u32;
          
          let success = unsafe {
            ffi::hitoVaultWriteFlash(
              HITO_BOOTLOADER_SEAL_ADDRESS as *const u8,
              &seal_update as *const HitoSealBlock as *const u8,
              core::mem::size_of::<HitoSealBlock>()
            )
          };

          if !success {
            log_info!("Failed to update seal block during vault erase");
            return false;
          }
        }
      }

      if main {
        unsafe {
          ffi::hitoVaultEraseFlash(VAULT_MAIN_PAGE as u32, VAULT_PAGE_SIZE_BYTES);
        }
      }
      if backup {
        unsafe {
          ffi::hitoVaultEraseFlash(VAULT_BACKUP_PAGE as u32, VAULT_PAGE_SIZE_BYTES);
        }
      }
    }

    #[cfg(feature = "minifb")]
    {
      const VAULT_PAGE_SIZE_BYTES: usize = 4096;
      
      if main {
        unsafe {
          ptr::write_bytes(VAULT_MAIN_PAGE as *mut u8, 0xff, VAULT_PAGE_SIZE_BYTES);
        }
      }
      if backup {
        unsafe {
          ptr::write_bytes(VAULT_BACKUP_PAGE as *mut u8, 0xff, VAULT_PAGE_SIZE_BYTES);
        }
      }
    }

    // Lock the vault
    self.vaultIsUnlocked = false;
    true
  }

  fn write_ram_block(
    offset: *const VaultEncryptedBlock,
    data: *const VaultEncryptedBlock,
    len: usize
  ) -> bool {
    unsafe {
      core::ptr::copy_nonoverlapping(data as *const u8, offset as *mut u8, len);
      true
    }
    
  }

  /// Flash write implementation - handles both Zephyr flash and simulation memory copy
  fn write_flash(
    offset: *const VaultEncryptedBlock,
    data: *const VaultEncryptedBlock,
    len: usize
  ) -> bool {
    #[cfg(feature = "zephyr")]
    {
      // For Zephyr target, use actual flash operations
      unsafe {
        ffi::hitoVaultWriteFlash(offset as *const u8, data as *const u8, len)
      }
    }
    
    #[cfg(feature = "minifb")]
    {
      // For simulation, just copy memory
      unsafe {
        core::ptr::copy_nonoverlapping(data as *const u8, offset as *mut u8, len);
        true
      }
    }
  }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vault_create_set_passcode_and_unlock() {
        let mut vault = HitoVault::new();
        vault.init();

        // Set up initial entropy and seed
        let entropy = [0x12u8; 32];
        let passcode = "test_password_123".as_bytes();

        // Set entropy and seed
        vault.set_entropy(&entropy, 32);

        // Set passcode on empty vault
        assert!(vault.set_passcode(passcode).is_ok());
        
        let seed = vault.seed;

        // Create new vault instance and unlock with the same passcode
        let mut vault2 = HitoVault::new();
        vault2.init();

        // Unlock with the passcode
        let passcode = b"test_password_123";
        assert!(vault2.unlock_with_password(passcode).is_ok());

        // Verify vault is unlocked
        assert!(vault2.is_unlocked());

        // Verify entropy and seed match
        assert_eq!(vault2.entropy[..32], entropy);
        assert_eq!(vault2.seed, seed);
    }

    #[test]
    fn test_vault_unlock_with_wrong_passcode() {
        let mut vault = HitoVault::new();
        vault.init();

        let entropy = [0xAAu8; 32];
        let correct_passcode = b"correct_password";
        let wrong_passcode = b"wrong_password!!";

        vault.set_entropy(&entropy, 32);
        assert!(vault.set_passcode(correct_passcode).is_ok());

        let mut vault2 = HitoVault::new();
        vault2.init();

        // Try to unlock with wrong passcode
        let result = vault2.unlock_with_password(wrong_passcode);
        assert!(result.is_err());
        assert!(!vault2.is_unlocked());
    }

    #[test]
    fn test_vault_empty_raises_error() {
        let vault = HitoVault::new();
        assert!(vault.is_empty());
        assert!(vault.last_block().is_err());
    }

    #[test]
    fn test_vault_unlock_empty_vault_fails() {
        let mut vault = HitoVault::new();
        vault.init();

        let result = vault.unlock_with_password(b"any_password");
        assert!(result.is_err());
        assert!(!vault.is_unlocked());
    }

    #[test]
    fn test_vault_multiple_passcode_changes() {
        let mut vault = HitoVault::new();
        vault.init();

        let entropy = [0xCCu8; 32];
        let passcode1 = b"first_password";
        let passcode2 = b"second_password";

        vault.set_entropy(&entropy, 32);
        assert!(vault.set_passcode(passcode1).is_ok());

        // Change passcode
        vault.vaultIsUnlocked = true; // Simulate unlocked state for passcode change
        assert!(vault.set_passcode(passcode2).is_ok());

        assert!(vault.unlock_with_password(passcode2).is_ok());
        assert!(vault.is_unlocked());
    }

    #[test]
    fn test_vault_different_entropy_lengths() {
        for entropy_len in &[16, 24, 32] {
            let mut vault = HitoVault::new();
            vault.init();

            let entropy: [u8; 32] = [0xEEu8; 32];
            let passcode = b"test_pass";

            vault.set_entropy(&entropy, *entropy_len);
            assert!(vault.set_passcode(passcode).is_ok());

            let mut vault2 = HitoVault::new();
            vault2.init();
            assert!(vault2.unlock_with_password(passcode).is_ok());
            assert_eq!(vault2.entropy_len as usize, *entropy_len);
        }
    }

    #[test]
    fn test_vault_passcode_length_validation() {
        let mut vault = HitoVault::new();
        vault.init();

        let entropy = [0x99u8; 32];

        vault.set_entropy(&entropy, 32);

        // Valid passcode lengths (1-32 bytes)
        assert!(vault.set_passcode(b"a").is_ok());
        
        let mut vault2 = HitoVault::new();
        vault2.init();
        assert!(vault2.unlock_with_password(b"a").is_ok());
    }
}