use crate::drivers::{ Display, DisplayImpl };
use crate::drivers::{ Battery, BatteryImpl };
use crate::drivers::{ Touch,     TouchImpl };
use crate::drivers::{ Indicator, IndicatorImpl, LedColor, BlinkSpeed };

//use crate::lib::crypt0;

use crate::crypto::crypt0::hex_to_bytes;
use crate::log_info;
extern crate alloc;
use alloc::sync::Arc;
use spin::Mutex;

#[cfg(feature = "zephyr")]
use crate::drivers::zephyr::logging;
use crate::vault::vault::HitoVault;
#[cfg(feature = "minifb")]
use crate::drivers::minifb::SocketProtocol;

const TEST_PASSCODE: &[u8] = b"000000";

pub struct HitoFirmware {
    pub display:   Arc<Mutex<DisplayImpl>>,
    pub touch:     Arc<Mutex<TouchImpl>>,
    pub indicator: Arc<Mutex<IndicatorImpl>>,
    pub battery:   Arc<Mutex<BatteryImpl>>,
    pub vault:     Arc<Mutex<HitoVault>>,
}


impl HitoFirmware {
    pub fn new() -> Self {
        Self {
            indicator: Arc::new(Mutex::new(IndicatorImpl::new())),
            display:   Arc::new(Mutex::new(DisplayImpl::new())),
            touch:     Arc::new(Mutex::new(TouchImpl::new())),
            battery:   Arc::new(Mutex::new(BatteryImpl::new())),
            vault:     Arc::new(Mutex::new(HitoVault::new())),
        }
    }

    pub fn init_hardware(&mut self) {
        #[cfg(feature = "zephyr")]
        logging::log_info("Initializing hardware...");

        self.display.lock().init();
        self.indicator.lock().init();
        self.touch.lock().init();
        //self.vault.lock().init();
        // #[cfg(feature = "minifb")]
        // {
        //     let mut vault_lock = self.vault.lock(); 
        //     vault_lock.unlock_with_password(TEST_PASSCODE, None).expect("Failed to unlock vault");
        // }

        log_info!("Hardware initialization complete");

        //self.display.fill_rect(10, 10, 100, 50, 0xF800); // Red rectangle
    }
}
