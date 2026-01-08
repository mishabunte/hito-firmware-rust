use core::mem::size_of;

#[cfg(feature = "minifb")]
use std::{fs::{self, File}, io::{Read, Write}, path::PathBuf, sync::atomic::{AtomicBool, Ordering}};
#[cfg(feature = "minifb")]
use dirs;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct HitoBootloaderVersion {
  pub major: u8,
  pub minor: u8,
  pub revision: u8,
  pub build: u16,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct HitoBootloaderVersionBlock {
  pub magic: u32,            // bootloader version block magic 0xB00710AD
  pub version: HitoBootloaderVersion, // bootloader version
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct HitoSealBlock {
  pub magic: u32,            // seal block magic 0x5EA1A55E
  pub reset_counter_bits: u32,
  pub serial_number_salt: [u8; 32],
}

const HITO_BOOTLOADER_MAGIC: u32 = 0x2144df1c;
const HITO_BOOTLOADER_VERSION_ADDRESS: u32 = 0x100000 - size_of::<HitoBootloaderVersionBlock>() as u32;

#[cfg(feature = "minifb")]
const SEAL_FILE: &str = "hito_vault_seal.bin";

#[cfg(feature = "minifb")]
const DEFAULT_SEAL: HitoSealBlock = HitoSealBlock {
  magic: HITO_BOOTLOADER_MAGIC,
  reset_counter_bits: 0xffffffff,
  serial_number_salt: [0u8; 32],
};

#[cfg(feature = "minifb")]
static SEAL_STORAGE_INITIALIZED: AtomicBool = AtomicBool::new(false);

#[cfg(feature = "minifb")]
static mut SEAL_STORAGE: HitoSealBlock = HitoSealBlock {
  magic: HITO_BOOTLOADER_MAGIC,
  reset_counter_bits: 0xffffffff,
  serial_number_salt: [0u8; 32],
};

#[cfg(feature = "minifb")]
const TEST_BOOTLOADER_VERSION: HitoBootloaderVersion = HitoBootloaderVersion {
  major: 1,
  minor: 0,
  revision: 0,
  build: 42,
};
#[cfg(feature = "zephyr")]
const HITO_BOOTLOADER_SEAL_ADDRESS: u32 = 0x100000 - size_of::<HitoSealBlock>() as u32 - size_of::<HitoBootloaderVersionBlock>() as u32;

/// Get the directory path for seal storage file
#[cfg(feature = "minifb")]
fn get_seal_storage_dir() -> PathBuf {
    dirs::data_local_dir()
        .or_else(|| dirs::home_dir())
        .unwrap_or_else(|| PathBuf::from("."))
        .join("hito_wallet")
}

/// Load seal block from file
#[cfg(feature = "minifb")]
fn load_seal_from_file() {
    let path = get_seal_storage_dir().join(SEAL_FILE);
    if let Ok(mut file) = File::open(&path) {
        let storage_bytes = unsafe {
            core::slice::from_raw_parts_mut(
                &mut SEAL_STORAGE as *mut HitoSealBlock as *mut u8,
                core::mem::size_of::<HitoSealBlock>()
            )
        };
        if file.read_exact(storage_bytes).is_ok() {
            // Verify magic
            unsafe {
                if SEAL_STORAGE.magic != HITO_BOOTLOADER_MAGIC {
                    SEAL_STORAGE = DEFAULT_SEAL;
                }
            }
        }
    }
}

/// Save seal block to file
#[cfg(feature = "minifb")]
fn save_seal_to_file(seal: &HitoSealBlock) -> bool {
    let dir = get_seal_storage_dir();
    if fs::create_dir_all(&dir).is_err() {
        return false;
    }
    let path = dir.join(SEAL_FILE);
    match File::create(&path) {
        Ok(mut file) => {
            let storage_bytes = unsafe {
                core::slice::from_raw_parts(
                    seal as *const HitoSealBlock as *const u8,
                    core::mem::size_of::<HitoSealBlock>()
                )
            };
            file.write_all(storage_bytes).is_ok()
        }
        Err(_) => false,
    }
}

/// Initialize seal storage from file (called once)
#[cfg(feature = "minifb")]
fn init_seal_storage() {
    if SEAL_STORAGE_INITIALIZED.compare_exchange(
        false,
        true,
        Ordering::SeqCst,
        Ordering::SeqCst
    ).is_ok() {
        load_seal_from_file();
    }
}

impl HitoBootloaderVersion {
  pub fn get() -> Option<Self> {
    #[cfg(feature = "minifb")]
    {
      Some(TEST_BOOTLOADER_VERSION)
    }
    #[cfg(feature = "zephyr")]
    {
      let version_block_ptr = HITO_BOOTLOADER_VERSION_ADDRESS as *const HitoBootloaderVersionBlock;
      unsafe {
        let version_block = &*version_block_ptr;
        if version_block.magic != HITO_BOOTLOADER_MAGIC {
          return None;
        }
        Some(version_block.version)
      }
    }
  }
}

impl HitoSealBlock {
  pub fn get() -> Option<Self> {
    #[cfg(feature = "minifb")]
    {
      init_seal_storage();
      unsafe {
        if SEAL_STORAGE.magic != HITO_BOOTLOADER_MAGIC {
          return None;
        }
        Some(SEAL_STORAGE)
      }
    }
    #[cfg(feature = "zephyr")]
    {
      let seal_block_ptr = HITO_BOOTLOADER_SEAL_ADDRESS as *const HitoSealBlock;
      unsafe {
        use crate::log_info;

        let seal_block = &*seal_block_ptr;
        // log_info!("Read seal block from address {:x?}: {:?}", HITO_BOOTLOADER_SEAL_ADDRESS, seal_block);
        if seal_block.magic != HITO_BOOTLOADER_MAGIC {
          return None;
        }
        Some(*seal_block)
      }
    }
  }

  #[cfg(feature = "minifb")]
  pub fn save(seal: &HitoSealBlock) -> bool {
    init_seal_storage();
    unsafe {
      SEAL_STORAGE = *seal;
    }
    save_seal_to_file(seal)
  }
}
