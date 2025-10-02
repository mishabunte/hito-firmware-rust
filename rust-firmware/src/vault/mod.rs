pub mod ffi;

use core::ptr;
use core::slice;
use crate::crypto;
use crate::log_info;

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

#[derive(Debug, Clone, Copy, PartialEq)]
enum entropy_len_t {
  ENTROPY_LEN_16 = 16,
  ENTROPY_LEN_24 = 24,
  ENTROPY_LEN_32 = 32
}

const NONCE_LEN: usize = 7;
const AAD_LEN: usize = 7;
const TAG_LEN: usize = 8;

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
const VAULT_MAIN_PAGE: *const VaultEncryptedBlock = unsafe { VAULT_MAIN_STORAGE.as_ptr() };
#[cfg(feature = "minifb")]
const VAULT_BACKUP_PAGE: *const VaultEncryptedBlock = unsafe { VAULT_BACKUP_STORAGE.as_ptr() };

#[derive(Debug, PartialEq)]
pub enum VaultError {
    EmptyVault,
    InvalidPassword,
    CryptoError,
    HardwareKeyError,
    InvalidKeyLength,
    BlockNotFound,
}

pub type VaultResult<T> = Result<T, VaultError>;

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
  mnemonic: [u8; 215],
  entropy_len: entropy_len_t,
}
#[derive(Copy, Clone)]
pub struct VaultEncryptedBlock {
  magic: u32,                    // encrypted block magic 0xD0364141
  steps_count: u32,              // pbkdf2 steps count for key generation
  encrypted: [u8; 96],           // encrypted entropy (32-bit) and seed (64-bit)
  nonce: [u8; NONCE_LEN],        // AES CCM nonce
  auth_data: [u8; AAD_LEN],      // AES CCM auth data
  tag: [u8; TAG_LEN],            // AES CCM tag
  crc16_ccitt: u16,              // checksum to check if block itself is valid
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
        return Err(VaultError::HardwareKeyError);
      }
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
           entropy_len: entropy_len_t::ENTROPY_LEN_32 }
  }

  pub fn initialize(&mut self) {
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
        return Ok(&vault_slice[i - 1]);
      }
    }
    
    // If we reach here, all blocks are filled, return the last one
    vault_slice.last().ok_or(VaultError::BlockNotFound)
  }

  /// Derive encryption key from password using the correct C implementation
  pub fn derive_encryption_key(
    &self,
    password: &[u8],
    steps_count: u32,
    progress_handler: Option<fn(u8)>
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
      // 40 iterations of hardware key derivation (~8ms time)
      for _j in 0..40 {
        let hw_key = derive_hardware_key(&key)?;
        key.copy_from_slice(&hw_key);
      }
      
      let pbkdf2_result = unsafe {
        crypto::ffi::crypt0_pbkdf2_hmac_sha256(
          10,
          key.as_ptr(),        // Use current key as salt
          32,
          password.as_ptr(),
          password.len() as i32,
          key.as_mut_ptr(),    // Output to key
          32
        )
      };
      
      if pbkdf2_result != crypto::ffi::CRYPT0_OK {
        return Err(VaultError::CryptoError);
      }
      
      // Progress reporting
      if let Some(handler) = progress_handler {
        let new_progress = ((i * 100) / steps_count) as u8;
        if progress != new_progress {
          progress = new_progress;
          handler(progress);
        }
      }
    }
    
    // Final progress callback
    if let Some(handler) = progress_handler {
      handler(100);
    }
    
    Ok(key)
  }

  fn block_decrypt(
    &self,
    block: &VaultEncryptedBlock,
    passcode: &[u8],
    progress_handler: Option<fn(u8)>
  ) -> VaultResult<[u8; 96]> {
    // Derive AES key using the unified key derivation method
    let aes_key = self.derive_encryption_key(passcode, block.steps_count, progress_handler)?;
    
    // Decrypt using AES-CCM
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
        decrypted.as_mut_ptr()
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
    progress_handler: Option<fn(u8)>
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
    let aes_key = self.derive_encryption_key(pass, block.steps_count, progress_handler)?;

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
      }
    }
    
    Ok(())
  }

  /// Save vault data with new passcode
  fn vault_save(
    &self,
    entropy: &[u8],
    entropy_len: usize,
    seed: &[u8],
    new_pass: &[u8],
    progress_handler: Option<fn(u8)>
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
    self.block_encrypt(&mut block_flash, entropy_enum.clone(), entropy, seed, new_pass, progress_handler)?;
    
    // Encrypt RAM block
    self.block_encrypt(&mut block_ram, entropy_enum, entropy, seed, new_pass, None)?;

    // Save both blocks using our Rust implementation
    self.vault_save_block(&block_flash, Some(&block_ram))?;

    Ok(())
  }

  /// Set a new passcode for the vault
  pub fn set_passcode(
    &mut self,
    new_pass: &[u8],
    progress_handler: Option<fn(u8)>
  ) -> VaultResult<()> {
    // If vault is empty but we have entropy, generate seed from entropy
    if self.is_empty() && self.entropy_len as usize != 0 {
      let result = unsafe {
        crypto::ffi::crypt0_bip39_entropy_to_seed_en(
          self.entropy.as_ptr(),
          self.entropy_len as usize,
          self.seed.as_mut_ptr(),
          64
        )
      };
      
      if result != crypto::ffi::CRYPT0_OK {
        return Err(VaultError::CryptoError);
      }
      
      self.vaultIsUnlocked = true;
    }

    // Check if vault is unlocked
    if !self.vaultIsUnlocked {
      return Err(VaultError::InvalidPassword);
    }

    // Save entropy with new passcode
    self.vault_save(
      &self.entropy,
      self.entropy_len as usize,
      &self.seed,
      new_pass,
      progress_handler
    )?;

    log_info!("Vault saved with new passcode");

    Ok(())
  }

  /// Attempt to unlock the vault with the given password
  pub fn unlock_with_password(&mut self, password: &[u8]) -> VaultResult<()> {
    // Get the last block
    let block = self.last_block()?;
    
    // Decrypt the block
    let decrypted = self.block_decrypt(block, password, None)?;
    
    // Parse the decrypted data (entropy + seed) and store in vault
    // The first 32 bytes are entropy, next 64 bytes are seed
    let entropy_len = self.entropy_len as usize;
    if entropy_len > 0 && entropy_len <= 32 {
      self.entropy[..entropy_len].copy_from_slice(&decrypted[..entropy_len]);
      self.seed.copy_from_slice(&decrypted[32..]);
    }
    
    self.vaultIsUnlocked = true;
    Ok(())
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
        ffi::hitoVaultWriteFlash(offset, data, len)
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