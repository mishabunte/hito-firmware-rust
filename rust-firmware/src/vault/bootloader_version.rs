use core::mem::size_of;

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
const TEST_SEAL: HitoSealBlock = HitoSealBlock {
  magic: HITO_BOOTLOADER_MAGIC,
  reset_counter_bits: 0xfffffff,
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
      Some(TEST_SEAL)
    }
    #[cfg(feature = "zephyr")]
    {
      let seal_block_ptr = HITO_BOOTLOADER_SEAL_ADDRESS as *const HitoSealBlock;
      unsafe {
        use crate::log_info;

        let seal_block = &*seal_block_ptr;
        log_info!("Read seal block from address {:x?}: {:?}", HITO_BOOTLOADER_SEAL_ADDRESS, seal_block);
        if seal_block.magic != HITO_BOOTLOADER_MAGIC {
          return None;
        }
        Some(*seal_block)
      }
    }
  }
}
