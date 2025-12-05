use crate::drivers::{ Display, DisplayImpl };
use crate::drivers::{ Battery, BatteryImpl };
use crate::drivers::{ Touch,     TouchImpl };
use crate::drivers::{ Indicator, IndicatorImpl, LedColor, BlinkSpeed };

//use crate::lib::crypt0;

use crate::crypto::crypt0::hex_to_bytes;

#[cfg(feature = "zephyr")]
use crate::drivers::zephyr::logging;
use crate::vault::vault::HitoVault;
#[cfg(feature = "minifb")]
use crate::drivers::minifb::SocketProtocol;

const TEST_PASSCODE: &[u8] = b"000000";

pub struct HitoFirmware {
    pub display:   DisplayImpl,
    pub touch:     TouchImpl,
    pub indicator: IndicatorImpl,
    pub battery:   BatteryImpl,
    pub vault:     HitoVault,
}


impl HitoFirmware {
    pub fn new() -> Self {
        Self {
            indicator: IndicatorImpl::new(),
            display:   DisplayImpl::new(),
            touch:     TouchImpl::new(),
            battery:   BatteryImpl::new(),
            vault:     HitoVault::new(),
        }
    }

    pub fn init_hardware(&mut self) {
        #[cfg(feature = "zephyr")]
        logging::log_info("Initializing hardware...");

        self.display.init();
        self.indicator.init();
        self.touch.init();
        self.vault.init();
        #[cfg(feature = "minifb")]
        {
            self.vault.set_entropy(hex_to_bytes("ffbff7feffdffbff7feffdffbff7feff").unwrap().as_slice(), 16);
            self.vault.set_passcode(TEST_PASSCODE).expect("Failed to set passcode");
            self.vault.unlock_with_password(TEST_PASSCODE).expect("Failed to unlock vault");
        }

        #[cfg(feature = "zephyr")]
        logging::log_info("Hardware initialization complete");

        //self.display.fill_rect(10, 10, 100, 50, 0xF800); // Red rectangle
    }
}

