//! Desktop storage implementation for vault (minifb feature)
//! 
//! This module handles persistent file-based storage for vault data
//! when running on desktop platforms (macOS, Linux, Windows).

use std::{process::Command, fs::{self, File}, io::{Read, Write}, path::PathBuf, sync::atomic::{AtomicBool, Ordering}};
use sha2::{Sha256, Digest};
use crate::vault::{VaultEncryptedBlock, VaultError, VaultResult};
use crate::log_info;
use crate::crypto;
use crate::vault::{NONCE_LEN, AAD_LEN, TAG_LEN};

// Static storage arrays for vault data
pub static mut VAULT_MAIN_STORAGE: [VaultEncryptedBlock; VAULT_PAGE_SIZE_BLOCKS] = 
    [VaultEncryptedBlock {
        magic: 0xffffffff,
        steps_count: 0,
        encrypted: [0; 96],
        nonce: [0; NONCE_LEN],
        auth_data: [0; AAD_LEN],
        tag: [0; TAG_LEN],
        crc16_ccitt: 0,
    }; VAULT_PAGE_SIZE_BLOCKS];
pub static mut VAULT_BACKUP_STORAGE: [VaultEncryptedBlock; VAULT_PAGE_SIZE_BLOCKS] = 
    [VaultEncryptedBlock {
        magic: 0xffffffff,
        steps_count: 0,
        encrypted: [0; 96],
        nonce: [0; NONCE_LEN],
        auth_data: [0; AAD_LEN],
        tag: [0; TAG_LEN],
        crc16_ccitt: 0,
    }; VAULT_PAGE_SIZE_BLOCKS];
pub static mut VAULT_RAM_STORAGE: [VaultEncryptedBlock; VAULT_PAGE_SIZE_BLOCKS] = 
    [VaultEncryptedBlock {
        magic: 0xffffffff,
        steps_count: 0,
        encrypted: [0; 96],
        nonce: [0; NONCE_LEN],
        auth_data: [0; AAD_LEN],
        tag: [0; TAG_LEN],
        crc16_ccitt: 0,
    }; VAULT_PAGE_SIZE_BLOCKS];

// Initialization flags
static HARDWARE_ID_INITIALIZED: AtomicBool = AtomicBool::new(false);
static mut HARDWARE_ID: [u8; 128] = [0; 128];
static HUK_INITIALIZED: AtomicBool = AtomicBool::new(false);
static mut HUK_WRITTEN: bool = false;
static VAULT_STORAGE_INITIALIZED: AtomicBool = AtomicBool::new(false);

// File paths for persistent vault storage
const VAULT_MAIN_FILE: &str = "hito_vault_main.bin";
const VAULT_BACKUP_FILE: &str = "hito_vault_backup.bin";
const VAULT_RAM_FILE: &str = "hito_vault_ram.bin";
pub const VAULT_SEAL_FILE: &str = "hito_vault_seal.bin";

// Test-specific file paths
#[cfg(test)]
const VAULT_TEST_MAIN_FILE: &str = "hito_vault_test_main.bin";
#[cfg(test)]
const VAULT_TEST_BACKUP_FILE: &str = "hito_vault_test_backup.bin";
#[cfg(test)]
const VAULT_TEST_RAM_FILE: &str = "hito_vault_test_ram.bin";

// Flag to indicate we're running in test mode
#[cfg(test)]
static TEST_MODE: AtomicBool = AtomicBool::new(false);

const VAULT_PAGE_SIZE_BLOCKS: usize = 4096 / core::mem::size_of::<VaultEncryptedBlock>();

// Pointer to RAM storage (for compatibility with vault.rs interface)
pub const VAULT_RAM_PAGE: *const VaultEncryptedBlock = unsafe { VAULT_RAM_STORAGE.as_ptr() };

/// Get the directory path for vault storage files
pub fn get_vault_storage_dir() -> PathBuf {
    // Use home directory or current directory as fallback
    dirs::data_local_dir()
        .or_else(|| dirs::home_dir())
        .unwrap_or_else(|| PathBuf::from("."))
        .join("hito_wallet")
}

/// Ensure the vault storage directory exists
fn ensure_vault_storage_dir() -> std::io::Result<PathBuf> {
    let dir = get_vault_storage_dir();
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// Load vault storage from file into the static array
fn load_vault_from_file(filename: &str, storage: &mut [VaultEncryptedBlock; VAULT_PAGE_SIZE_BLOCKS]) {
    let path = get_vault_storage_dir().join(filename);
    if let Ok(mut file) = File::open(&path) {
        let storage_bytes = unsafe {
            core::slice::from_raw_parts_mut(
                storage.as_mut_ptr() as *mut u8,
                core::mem::size_of::<[VaultEncryptedBlock; VAULT_PAGE_SIZE_BLOCKS]>()
            )
        };
        if let Err(e) = file.read_exact(storage_bytes) {
            log_info!("Warning: Could not read vault file {}: {}", filename, e);
        } else {
            log_info!("Loaded vault storage from {}", path.display());
        }
    } else {
        log_info!("No existing vault file at {}, using empty storage", path.display());
    }
}

/// Save vault storage to file from the static array
pub fn save_vault_to_file(filename: &str, storage: &[VaultEncryptedBlock; VAULT_PAGE_SIZE_BLOCKS]) -> bool {
    match ensure_vault_storage_dir() {
        Ok(dir) => {
            let path = dir.join(filename);
            match File::create(&path) {
                Ok(mut file) => {
                    let storage_bytes = unsafe {
                        core::slice::from_raw_parts(
                            storage.as_ptr() as *const u8,
                            core::mem::size_of::<[VaultEncryptedBlock; VAULT_PAGE_SIZE_BLOCKS]>()
                        )
                    };
                    match file.write_all(storage_bytes) {
                        Ok(_) => {
                            log_info!("Saved vault storage to {}", path.display());
                            true
                        }
                        Err(e) => {
                            log_info!("Error writing vault file {}: {}", filename, e);
                            false
                        }
                    }
                }
                Err(e) => {
                    log_info!("Error creating vault file {}: {}", filename, e);
                    false
                }
            }
        }
        Err(e) => {
            log_info!("Error creating vault storage directory: {}", e);
            false
        }
    }
}

/// Initialize vault storage from persistent files (called once on startup)
pub fn init_vault_storage_from_files() {
    // Use compare_exchange to ensure only one thread initializes
    if VAULT_STORAGE_INITIALIZED.compare_exchange(
        false,
        true,
        Ordering::SeqCst,
        Ordering::SeqCst
    ).is_ok() {
        unsafe {
            load_vault_from_file(get_vault_main_file(), &mut VAULT_MAIN_STORAGE);
            load_vault_from_file(get_vault_backup_file(), &mut VAULT_BACKUP_STORAGE);
            load_vault_from_file(get_vault_ram_file(), &mut VAULT_RAM_STORAGE);
        }
    }
}

/// Reset vault storage initialization flag (for testing purposes)
#[cfg(test)]
pub fn reset_vault_storage_init() {
    VAULT_STORAGE_INITIALIZED.store(false, Ordering::SeqCst);
    TEST_MODE.store(true, Ordering::SeqCst);
}

/// Get the appropriate vault file name based on test mode
pub fn get_vault_main_file() -> &'static str {
    #[cfg(test)]
    {
        if TEST_MODE.load(Ordering::SeqCst) {
            return VAULT_TEST_MAIN_FILE;
        }
    }
    VAULT_MAIN_FILE
}

pub fn get_vault_backup_file() -> &'static str {
    #[cfg(test)]
    {
        if TEST_MODE.load(Ordering::SeqCst) {
            return VAULT_TEST_BACKUP_FILE;
        }
    }
    VAULT_BACKUP_FILE
}

pub fn get_vault_ram_file() -> &'static str {
    #[cfg(test)]
    {
        if TEST_MODE.load(Ordering::SeqCst) {
            return VAULT_TEST_RAM_FILE;
        }
    }
    VAULT_RAM_FILE
}

/// Get test file names (for test cleanup)
#[cfg(test)]
pub fn get_vault_test_main_file() -> &'static str {
    VAULT_TEST_MAIN_FILE
}

#[cfg(test)]
pub fn get_vault_test_backup_file() -> &'static str {
    VAULT_TEST_BACKUP_FILE
}

#[cfg(test)]
pub fn get_vault_test_ram_file() -> &'static str {
    VAULT_TEST_RAM_FILE
}

/// Save all vault storage to persistent files
pub fn save_all_vault_storage() -> bool {
    unsafe {
        let main_ok = save_vault_to_file(get_vault_main_file(), &VAULT_MAIN_STORAGE);
        let backup_ok = save_vault_to_file(get_vault_backup_file(), &VAULT_BACKUP_STORAGE);
        let ram_ok = save_vault_to_file(get_vault_ram_file(), &VAULT_RAM_STORAGE);
        main_ok && backup_ok && ram_ok
    }
}

/// Save only the main and backup vault storage (not RAM)
pub fn save_flash_vault_storage() -> bool {
    unsafe {
        let main_ok = save_vault_to_file(get_vault_main_file(), &VAULT_MAIN_STORAGE);
        let backup_ok = save_vault_to_file(get_vault_backup_file(), &VAULT_BACKUP_STORAGE);
        main_ok && backup_ok
    }
}

/// Save only the RAM vault storage
pub fn save_ram_vault_storage() -> bool {
    unsafe {
        save_vault_to_file(get_vault_ram_file(), &VAULT_RAM_STORAGE)
    }
}

/// Get hardware ID for desktop platforms
fn get_hardware_id() -> VaultResult<[u8; 128]> {
    // Use compare_exchange to ensure only one thread initializes
    if HARDWARE_ID_INITIALIZED.compare_exchange(
        false,
        true,
        Ordering::SeqCst,
        Ordering::SeqCst
    ).is_ok() {
        unsafe {
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
        }
    }
    
    unsafe { Ok(HARDWARE_ID) }
}

/// Derive hardware key for desktop platforms (simulated using PBKDF2)
pub fn derive_hardware_key(salt: &[u8]) -> VaultResult<[u8; 32]> {
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

/// Simulate HUK (Hardware Unique Key) initialization
pub fn hw_unique_key_is_written_sim() -> bool {
    // Use compare_exchange to ensure only one thread initializes
    if HUK_INITIALIZED.compare_exchange(
        false,
        true,
        Ordering::SeqCst,
        Ordering::SeqCst
    ).is_ok() {
        unsafe {
            // In simulation, we simulate the HUK as being written
            // This mimics the behavior of the real hardware
            HUK_WRITTEN = true;
        }
    }
    unsafe { HUK_WRITTEN }
}

/// Simulate writing random HUK
pub fn hw_unique_key_write_random_sim() {
    unsafe {
        HUK_WRITTEN = true;
    }
}
