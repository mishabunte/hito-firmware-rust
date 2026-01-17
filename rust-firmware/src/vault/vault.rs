use core::cell::Cell;
use core::ffi::CStr;
use core::ptr;
use core::slice;
use crate::crypto;
use crate::crypto::crypt0::{entropy_to_mnemonic, entropy_to_seed, generate_entropy};
#[cfg(feature = "minifb")]
use crate::vault::desktop_storage::{save_flash_vault_storage, save_ram_vault_storage, get_vault_storage_dir};
use crate::vault::{VaultError, VaultResult};
use crate::crypto::libcrypt0pro::stellar::StellarKeypair;
use crate::log_info;
use crate::now_us;
use alloc::format;
extern crate alloc;
use crate::vault::{NONCE_LEN, AAD_LEN, TAG_LEN};
use crate::vault::VaultEncryptedBlock;
use crate::vault::bootloader_version::*;
use crate::vault::firmware_version::HitoFirmwareVersion;
use crate::firmware_state::DeviceInfo;

use super::ffi;

use crate::crypto::libcrypt0pro::stellar::StellarWallet;

#[cfg(feature = "minifb")]
use crate::vault::desktop_storage;

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
const CONFIG_SRAM_BASE_ADDRESS: usize = 0x20000000;

#[cfg(feature = "zephyr")]
const VAULT_RAM_PAGE: *const VaultEncryptedBlock = (CONFIG_SRAM_BASE_ADDRESS + 0x6f800) as *const VaultEncryptedBlock;
#[cfg(feature = "zephyr")]
const VAULT_MAIN_PAGE: *const VaultEncryptedBlock = 0x2e000 as *const VaultEncryptedBlock;
#[cfg(feature = "zephyr")]
const VAULT_BACKUP_PAGE: *const VaultEncryptedBlock = 0x2f000 as *const VaultEncryptedBlock;

// Desktop storage - import from desktop_storage module
#[cfg(feature = "minifb")]
const VAULT_MAIN_PAGE: *const VaultEncryptedBlock = unsafe { desktop_storage::VAULT_MAIN_STORAGE.as_ptr() };
#[cfg(feature = "minifb")]
const VAULT_BACKUP_PAGE: *const VaultEncryptedBlock = unsafe { desktop_storage::VAULT_BACKUP_STORAGE.as_ptr() };
#[cfg(feature = "minifb")]
const VAULT_RAM_PAGE: *const VaultEncryptedBlock = desktop_storage::VAULT_RAM_PAGE;

#[derive(Clone)]
struct VaultNetworkData {
  eth_key: Option<[u8; 32]>,
  near_key: Option<[u8; 32]>,
  solana_key: Option<[u8; 32]>,
  solana_addr: Option<[u8; 32]>,
  eth_addr: Option<[u8; 43]>,
  near_addr: Option<[u8; 64]>,
  btc_addr: Option<[u8; 75]>,
  stellar_public: Option<[u8; 32]>,
  stellar_secret: Option<[u8; 32]>,
}

#[derive(Clone)]
struct HitoVaultData {
  entropy: [u8; 32],
  entropy_len: entropy_len_t,
  seed: [u8; 64],
  mnemonic: [u8; 215],
  network_data: Option<VaultNetworkData>,
}


#[derive(Clone)]
#[repr(C)]
pub struct HitoVault {
  initialized: bool,
  vault_is_unlocked: bool,
  data: Option<HitoVaultData>,
  progress_handler: Option<fn(u8)>,
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
    desktop_storage::derive_hardware_key(salt)
  }
}

impl HitoVault {
  pub fn new() -> Self {
    #[cfg(feature = "minifb")]
    {
      // Load vault storage from persistent files
      desktop_storage::init_vault_storage_from_files();
    }
    Self {
          initialized: false,
          vault_is_unlocked: false,
          data: None,
          progress_handler: None,
       }
  }

  /// Get a mutable reference to data, returns error if not initialized
  fn data_mut(&mut self) -> VaultResult<&mut HitoVaultData> {
    self.data.as_mut().ok_or(VaultError::VaultLocked)
  }

  /// Get a reference to data, returns error if not initialized
  fn data_ref(&self) -> VaultResult<&HitoVaultData> {
    self.data.as_ref().ok_or(VaultError::VaultLocked)
  }

  pub fn init(&mut self, entropy: &[u8; 32], entropy_len: usize, new_pass: &[u8], progress_handler: Option<fn(u8)>) -> VaultResult<()> {
    if !self.initialized {
      Self::rust_hw_unique_key_is_written_impl();

      let seed = entropy_to_seed(entropy, entropy_len).map_err(|_| VaultError::CryptoError)?;

      self.save(entropy, entropy_len, &seed, new_pass, progress_handler)?;

      self.initialized = true;

      Ok(())
    } else {
      Err(VaultError::VaultLocked)
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
        log_info!("Found last valid vault block at index {}", i - 1);
        return Ok(&vault_slice[i - 1]);
      }
    }
    // If we reach here, all blocks are filled, return the last one
    vault_slice.last().ok_or(VaultError::BlockNotFound)
  }

  /// Derive encryption key from password using the correct C implementation
  pub fn derive_encryption_key(
    password: &[u8],
    steps_count: u32,
    progress_handler: Option<fn(u8)>,
  ) -> VaultResult<[u8; 32]> {
    let mut key = [0u8; 32];
    if password.len() == 0 || password.len() > 32 {
      return Err(VaultError::InvalidKeyLength);
    }

    log_info!("Deriving encryption key with {} steps", steps_count);
    
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

      if pbkdf2_result != 0 {
        return Err(VaultError::CryptoError);
      }
      
      if let Some(handler) = progress_handler {
        let new_progress = ((i + 1) * 100 / steps_count) as u8;
        if new_progress != progress {
          progress = new_progress;
          handler(progress);
        }
        if progress >= 100 {
          progress = 100;
        }
      }
    }
    
    Ok(key)
  }

  pub fn block_decrypt(
    block: &VaultEncryptedBlock,
    passcode: &[u8],
    progress_handler: Option<fn(u8)>,
  ) -> VaultResult<[u8; 96]> {
    // Derive AES key using the unified key derivation method
    let aes_key = HitoVault::derive_encryption_key(passcode, block.steps_count, progress_handler)?;
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
    progress_handler: Option<fn(u8)>,
  ) -> VaultResult<()> {
    // Set magic based on entropy length
    block.magic = match entropy_len {
      entropy_len_t::ENTROPY_LEN_16 => HITO_VAULT_HEADER_MAGIC_12_WORDS_V1,
      entropy_len_t::ENTROPY_LEN_24 => HITO_VAULT_HEADER_MAGIC_18_WORDS_V1,
      entropy_len_t::ENTROPY_LEN_32 => HITO_VAULT_HEADER_MAGIC_24_WORDS_V1,
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
    let aes_key = HitoVault::derive_encryption_key(pass, block.steps_count, progress_handler)?;

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
    
    #[cfg(feature = "minifb")]
    {
      // Persist cleared RAM storage to file

    use crate::vault::desktop_storage::save_ram_vault_storage;
      save_ram_vault_storage();
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


  fn is_factory_setup(&self) -> bool {
    self.get_reset_count() == 0
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
    let data = self.data_ref()?;
    let stellar_public = match &data.network_data {
      Some(nd) => match &nd.stellar_public {
        Some(pk) => pk,
        None => return Err(VaultError::VaultLocked),
      },
      None => return Err(VaultError::VaultLocked),
    };
    Ok(StellarWallet::encode_stellar_address(&stellar_public).unwrap())
  }

  /// Save vault data with new passcode
  fn save(
    &mut self,
    entropy: &[u8; 32],
    entropy_len: usize,
    seed: &[u8; 64],
    new_pass: &[u8],
    progress_handler: Option<fn(u8)>,
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
    self.block_encrypt(&mut block_ram, entropy_enum, entropy, seed, new_pass, progress_handler)?;

    // Save both blocks using our Rust implementation
    self.vault_save_block(&block_flash, Some(&block_ram))?;

    let vault_data: HitoVaultData = HitoVaultData {
      entropy: *entropy,
      entropy_len: entropy_enum,
      seed: *seed,
      mnemonic: entropy_to_mnemonic(entropy, entropy_len).map_err(|_| VaultError::CryptoError)?,
      network_data: None,
    };

    self.data = Some(vault_data);

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
    let entropy = generate_entropy(entropy_len).map_err(|_| VaultError::CryptoError)?;
    //log_info!("Generated entropy: {:?}", &entropy[..entropy_len]);
    
    // Save mnemonic
    let mnemonic = entropy_to_mnemonic(&entropy, entropy_len).map_err(|_| VaultError::CryptoError)?;
    //log_info!("Generated mnemonic: {:?}", core::str::from_utf8(&mnemonic).unwrap_or("Invalid UTF-8"));
    
    // Generate seed from entropy
    let seed = entropy_to_seed(&entropy, entropy_len).map_err(|_| VaultError::CryptoError)?;
    //log_info!("Generated seed from entropy: {:?}", &seed[..64]);
    self.data = Some(HitoVaultData {
      entropy: entropy,
      entropy_len: entropy_enum,
      seed,
      mnemonic,
      network_data: None,
    });
    
    Ok(())
  }

  fn generate_stellar_keypair(&self, data: &HitoVaultData) -> VaultResult<StellarKeypair> {
    // log_info!("Saving Stellar secret key");

    let wallet = StellarWallet::from_seed(data.seed);
    Ok(wallet.derive_keypair(0).unwrap())
  }

  pub fn get_stellar_keypair(&self) -> VaultResult<StellarKeypair> {
    if !self.is_unlocked() {
      return Err(VaultError::VaultLocked);
    }
    let data = self.data.as_ref().ok_or(VaultError::VaultLocked)?;
    let network_data = data.network_data.as_ref().ok_or(VaultError::VaultLocked)?;
    let stellar_public = match &network_data.stellar_public {
      Some(pk) => pk,
      None => return Err(VaultError::VaultLocked),
    };
    let stellar_secret = match &network_data.stellar_secret {
      Some(sk) => sk,
      None => return Err(VaultError::VaultLocked),
    };
    Ok(StellarKeypair {
      public_key: *stellar_public,
      secret_key: *stellar_secret,
    })
  }

  fn save_network_data(&mut self) -> VaultResult<()> {
    // log_info!("Saving vault data");
    let stellar_keypair = self.generate_stellar_keypair(self.data_ref()?)?;
    let data = VaultNetworkData {
      eth_key: None,
      near_key: None,
      solana_key: None,
      solana_addr: None,
      eth_addr: None,
      near_addr: None,
      btc_addr: None,
      stellar_public: Some(stellar_keypair.public_key),
      stellar_secret: Some(stellar_keypair.secret_key),
    };

    let vault_data = self.data_mut()?;
    vault_data.network_data = Some(data);

    // log_info!("Vault data saved successfully");
    Ok(())
  }


  pub fn get_mnemonic(&self) -> VaultResult<alloc::string::String> {
      let data = self.data.as_ref().ok_or(VaultError::VaultLocked)?;
      let mnemonic_str = match core::str::from_utf8(&data.mnemonic) {
          Ok(s) => alloc::string::String::from(s),
          Err(_) => return Err(VaultError::InvalidMnemonicUtf8),
      };
      Ok(mnemonic_str)
  }

  pub fn get_mnemonic_len(&self) -> VaultResult<usize> {
    let data = self.data.as_ref().ok_or(VaultError::VaultLocked)?;
    Ok(match data.entropy_len {
      entropy_len_t::ENTROPY_LEN_16 => 12,
      entropy_len_t::ENTROPY_LEN_24 => 18,
      entropy_len_t::ENTROPY_LEN_32 => 24,
    })
  }

  /// Set a new passcode for the vault
  pub fn set_passcode(
    &mut self,
    new_pass: &[u8],
    progress_handler: Option<fn(u8)>,
  ) -> VaultResult<()> {
    // Check if vault is empty before borrowing data
    let vault_is_empty = self.is_empty();
    
    // Ensure data is initialized
    let data = self.data.as_mut().ok_or(VaultError::VaultLocked)?;
    
    // If vault is empty but we have entropy, generate seed from entropy
    if vault_is_empty {
      // log_info!("Generating seed from entropy");
      let result = unsafe {
        crypto::ffi::crypt0_bip39_entropy_to_seed_en(
          data.entropy.as_ptr(),
          data.entropy_len as u16,
          data.seed.as_mut_ptr(),
          64
        )
      };
    
      log_info!("Generated seed from entropy: {:?}", &data.seed[..64]);
      
      if result != crypto::ffi::CRYPT0_OK {
        return Err(VaultError::CryptoError);
      }
    }

    // Save entropy with new passcode
    let entropy_copy = data.entropy;
    let entropy_len = data.entropy_len as usize;
    let seed_copy = data.seed;
    
    self.save(
      &entropy_copy,
      entropy_len,
      &seed_copy,
      new_pass,
      progress_handler
    )?;

    // log_info!("Vault saved with new passcode");

    Ok(())
  }

  /// Attempt to unlock the vault with the given password
  pub fn unlock_with_password(&mut self, password: &[u8], progress_handler: Option<fn(u8)>) -> VaultResult<()> {
    // Get the last block
    let block = self.last_block()?;
    let block_magic = block.magic; // Copy before mutable borrow

    let mut ram_block = unsafe {
        if VAULT_RAM_PAGE.is_null() {
            None
        } else {
            Some(*VAULT_RAM_PAGE)
        }
    };

    if !Self::is_block_checksum_valid(ram_block) {
        ram_block = None;
    } else if (block_magic == HITO_VAULT_HEADER_MAGIC_12_WORDS_ALPHA) ||
              (block_magic == HITO_VAULT_HEADER_MAGIC_18_WORDS_ALPHA) ||
              (block_magic == HITO_VAULT_HEADER_MAGIC_24_WORDS_ALPHA)
    {
        HitoVault::erase_ram_block();
        ram_block = None;
    }

    let to_decode = if let Some(ram) = ram_block {
        ram
    } else {
        *block
    };
    // Release immutable borrow of self by dropping block reference
    drop(block);

    // Decrypt the block
    let decrypted = HitoVault::block_decrypt(&to_decode, password, progress_handler)?;
    
    // Parse entropy length from block magic
    let entropy_len_enum = match block_magic {
        HITO_VAULT_HEADER_MAGIC_12_WORDS_V1 |
        HITO_VAULT_HEADER_MAGIC_12_WORDS_ALPHA => entropy_len_t::ENTROPY_LEN_16,
        HITO_VAULT_HEADER_MAGIC_18_WORDS_V1 |
        HITO_VAULT_HEADER_MAGIC_18_WORDS_ALPHA => entropy_len_t::ENTROPY_LEN_24,
        HITO_VAULT_HEADER_MAGIC_24_WORDS_V1 |
        HITO_VAULT_HEADER_MAGIC_24_WORDS_ALPHA => entropy_len_t::ENTROPY_LEN_32,
        _ => return Err(VaultError::CryptoError),
    };
    
    let entropy_len = entropy_len_enum as usize;

    log_info!("Parsed entropy length: {}, block_magic: {}", entropy_len, block_magic);

    if entropy_len == 0 || entropy_len > 32 {
        return Err(VaultError::InvalidKeyLength);
    }

    // Extract entropy and seed from decrypted data
    let mut entropy = [0u8; 32];
    entropy[..32].copy_from_slice(&decrypted[..32]);
    
    let mut seed = [0u8; 64];
    seed.copy_from_slice(&decrypted[32..(32 + 64)]);
    
    log_info!("Vault decrypted successfully, decrypted: {:?}, entropy_len: {}", &decrypted[..(32 + 64)], entropy_len);

    let mnemonic = entropy_to_mnemonic(&entropy, entropy_len).map_err(|_| VaultError::CryptoError)?;

    // Construct the complete HitoVaultData
    let vault_data = HitoVaultData {
        entropy,
        entropy_len: entropy_len_enum,
        seed,
        mnemonic,
        network_data: None,
    };

    // Assign the constructed data to self
    self.data = Some(vault_data);

    // Write RAM block if needed
    if ram_block.is_none() {
      let block_ram = VAULT_RAM_PAGE as *mut VaultEncryptedBlock;
      unsafe {
        if !block_ram.is_null() {
          (*block_ram).steps_count = if self.is_factory_setup() {
            HITO_VAULT_STEPS_COUNT_FACTORY_SETUP
          } else {
            HITO_VAULT_STEPS_COUNT_RAM
          };
        }
        self.block_encrypt(block_ram.as_mut().unwrap(), entropy_len_enum, &entropy, &seed, password, None)?;
      }
    }
    
    self.save_network_data()?;
    self.vault_is_unlocked = true;
    Ok(())
  }

  pub fn is_unlocked(&self) -> bool {
    self.vault_is_unlocked
  }

  fn rust_hw_unique_key_is_written_impl() {
    #[cfg(feature = "zephyr")]
    {
      unsafe { ffi::rust_hw_unique_key_is_written() };
    }
    
    #[cfg(feature = "minifb")]
    {
      if !desktop_storage::hw_unique_key_is_written_sim() {
        desktop_storage::hw_unique_key_write_random_sim();
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
      
      // Increment reset counter if erasing both main and backup
      if main && backup {
        if let Some(seal) = HitoSealBlock::get() {
          let mut seal_update = seal;
          seal_update.reset_counter_bits = seal_update.reset_counter_bits << 1;
          
          if !HitoSealBlock::save(&seal_update) {
            log_info!("Failed to update seal block during vault erase");
            return false;
          }
        }
      }
      
      if main {
        unsafe {
          ptr::write_bytes(VAULT_MAIN_PAGE as *mut u8, 0xff, VAULT_PAGE_SIZE_BYTES);
        }
        desktop_storage::save_vault_to_file(desktop_storage::get_vault_main_file(), unsafe { &desktop_storage::VAULT_MAIN_STORAGE });
      }
      if backup {
        unsafe {
          ptr::write_bytes(VAULT_BACKUP_PAGE as *mut u8, 0xff, VAULT_PAGE_SIZE_BYTES);
        }
        desktop_storage::save_vault_to_file(desktop_storage::get_vault_backup_file(), unsafe { &desktop_storage::VAULT_BACKUP_STORAGE });
      }
    }

    // Lock the vault
    self.vault_is_unlocked = false;
    self.data = None;
    true
  }

  fn write_ram_block(
    offset: *const VaultEncryptedBlock,
    data: *const VaultEncryptedBlock,
    len: usize
  ) -> bool {
    unsafe {
      core::ptr::copy_nonoverlapping(data as *const u8, offset as *mut u8, len);
    }
    
    #[cfg(feature = "minifb")]
    {
      // Persist RAM storage to file
      save_ram_vault_storage()
    }
    
    #[cfg(not(feature = "minifb"))]
    {
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
      // For simulation, copy memory and persist to file
      unsafe {
        core::ptr::copy_nonoverlapping(data as *const u8, offset as *mut u8, len);
      }
      // Persist to files after write
      save_flash_vault_storage()
    }
  }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    use crate::{crypto::crypt0::mnemonic_to_entropy, vault::desktop_storage::{get_vault_storage_dir, reset_vault_storage_init}};

    const TEST_MNEMONIC: &str = "zero zero zero zero zero zero zero zero zero zero zero zoo";

    /// Helper function to get entropy from the test mnemonic
    fn get_test_entropy() -> ([u8; 32], usize) {
        mnemonic_to_entropy(TEST_MNEMONIC).expect("Failed to convert test mnemonic to entropy")
    }

    /// Helper function to clear vault storage before each test
    fn reset_vault_for_test() {
        // Reset the initialization flag so storage can be reloaded
        desktop_storage::reset_vault_storage_init();
        
        // Delete existing test vault files so they don't get loaded in next iteration
        let storage_dir = desktop_storage::get_vault_storage_dir();
        let _ = fs::remove_file(storage_dir.join(desktop_storage::get_vault_main_file()));
        let _ = fs::remove_file(storage_dir.join(desktop_storage::get_vault_backup_file()));
        let _ = fs::remove_file(storage_dir.join(desktop_storage::get_vault_ram_file()));
        
        // Create an empty block template (simulates erased flash with 0xff)
        let empty_block = VaultEncryptedBlock {
            magic: 0xffffffff,
            steps_count: 0,
            encrypted: [0xff; 96],
            nonce: [0xff; NONCE_LEN],
            auth_data: [0xff; AAD_LEN],
            tag: [0xff; TAG_LEN],
            crc16_ccitt: 0xffff,
        };
        
        // Clear all vault storage to empty state
        unsafe {
            for block in desktop_storage::VAULT_MAIN_STORAGE.iter_mut() {
                *block = empty_block;
            }
            for block in desktop_storage::VAULT_BACKUP_STORAGE.iter_mut() {
                *block = empty_block;
            }
            for block in desktop_storage::VAULT_RAM_STORAGE.iter_mut() {
                *block = empty_block;
            }
        }
    }

    #[test]
    fn test_vault_create_set_passcode_and_unlock() {
        reset_vault_for_test();
        
        let mut vault = HitoVault::new();
        let passcode = b"test_password_123";

        let (entropy, entropy_len) = get_test_entropy();

        log_info!("Passcode is {:?}", passcode);

        // Initialize vault with entropy and passcode
        assert!(vault.init(&entropy, entropy_len, passcode, None).is_ok());
        
        let seed = vault.data.as_ref().unwrap().seed;

        // Create new vault instance and unlock with the same passcode
        let mut vault2 = HitoVault::new();

        // Unlock with the passcode
        log_info!("Passcode is {:?}", passcode);

        assert!(vault2.unlock_with_password(passcode, None).is_ok());

        // Verify vault is unlocked
        assert!(vault2.is_unlocked());

        // Verify entropy and seed match
        let data2 = vault2.data.as_ref().unwrap();
        assert_eq!(data2.entropy[..entropy_len], entropy[..entropy_len]);
        assert_eq!(data2.seed, seed);
    }

    #[test]
    fn test_vault_unlock_with_wrong_passcode() {
        reset_vault_for_test();
        
        let mut vault = HitoVault::new();

        let (entropy, entropy_len) = get_test_entropy();
        let correct_passcode = b"correct_password";
        let wrong_passcode = b"wrong_password!!";

        // Initialize vault with entropy and correct passcode
        assert!(vault.init(&entropy, entropy_len, correct_passcode, None).is_ok());

        let mut vault2 = HitoVault::new();

        // Try to unlock with wrong passcode
        let result = vault2.unlock_with_password(wrong_passcode, None);
        assert!(result.is_err());
        assert!(!vault2.is_unlocked());
    }

    #[test]
    fn test_vault_empty_raises_error() {
        reset_vault_for_test();
        
        let vault = HitoVault::new();
        assert!(vault.is_empty());
        assert!(vault.last_block().is_err());
    }

    #[test]
    fn test_vault_unlock_empty_vault_fails() {
        reset_vault_for_test();
        
        let mut vault = HitoVault::new();

        let result = vault.unlock_with_password(b"any_password", None);
        assert!(result.is_err());
        assert!(!vault.is_unlocked());
    }

    #[test]
    fn test_vault_multiple_passcode_changes() {
        reset_vault_for_test();
        
        let mut vault = HitoVault::new();

        let (entropy, entropy_len) = get_test_entropy();
        let passcode1 = b"first_password";
        let passcode2 = b"second_password";

        // Initialize vault with entropy and first passcode
        assert!(vault.init(&entropy, entropy_len, passcode1, None).is_ok());

        // Change passcode
        vault.vault_is_unlocked = true; // Simulate unlocked state for passcode change
        assert!(vault.set_passcode(passcode2, None).is_ok());

        let mut vault2 = HitoVault::new();
        assert!(vault2.unlock_with_password(passcode2, None).is_ok());
        assert!(vault2.is_unlocked());
    }

    #[test]
    fn test_vault_different_entropy_lengths() {
        // Test with 12-word mnemonic
        reset_vault_for_test();
        
        let (entropy, entropy_len) = get_test_entropy(); // 12 words = 16 bytes
        let passcode = b"test_pass";

        let mut vault = HitoVault::new();
        // Initialize vault with entropy and passcode
        assert!(vault.init(&entropy, entropy_len, passcode, None).is_ok());

        let mut vault2 = HitoVault::new();
        assert!(vault2.unlock_with_password(passcode, None).is_ok());
        assert_eq!(vault2.data.as_ref().unwrap().entropy_len as usize, entropy_len);
    }

    #[test]
    fn test_vault_passcode_length_validation() {
        reset_vault_for_test();
        
        let mut vault = HitoVault::new();

        let (entropy, entropy_len) = get_test_entropy();

        // Valid passcode lengths (1-32 bytes)
        // Initialize vault with entropy and single-char passcode
        assert!(vault.init(&entropy, entropy_len, b"a", None).is_ok());
        
        let mut vault2 = HitoVault::new();
        assert!(vault2.unlock_with_password(b"a", None).is_ok());
    }
}