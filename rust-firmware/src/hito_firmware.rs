use crate::drivers::{ Display, DisplayImpl };
use crate::drivers::{ Battery, BatteryImpl };
use crate::drivers::{ Touch,     TouchImpl };
use crate::drivers::{ Indicator, IndicatorImpl, LedColor, BlinkSpeed };

//use crate::lib::crypt0;

#[cfg(feature = "zephyr")]
use crate::drivers::zephyr::logging;

pub struct HitoFirmware {
    pub display:   DisplayImpl,
    pub touch:     TouchImpl,
    pub indicator: IndicatorImpl,
    pub battery:   BatteryImpl,
}


impl HitoFirmware {
    pub fn new() -> Self {
        Self {
            indicator: IndicatorImpl::new(),
            display:   DisplayImpl::new(),
            touch:     TouchImpl::new(),
            battery:   BatteryImpl::new(),
        }
    }

    pub fn init_hardware(&mut self) {
        #[cfg(feature = "zephyr")]
        logging::log_info("Initializing hardware...");

        self.display.init();
        self.indicator.init();
        self.touch.init();

        #[cfg(feature = "zephyr")]
        logging::log_info("Hardware initialization complete");

        //self.display.fill_rect(10, 10, 100, 50, 0xF800); // Red rectangle
    }

    pub fn main_loop(&mut self) {
        self.indicator.turn_on(LedColor::Blue);

        let mut x = 0xffff;
        let mut y = 0xffff;

        loop {
            self.indicator.blink(LedColor::Red, BlinkSpeed::Slow);
            if self.touch.is_pressed().unwrap_or(false) {

                // clear screen
                if x != 0xffff {
                    self.display.fill_rect(x as u16, y as u16, 20, 20, 0xffff); // Red rectangle at touch position
                }

                (x, y) = self.touch.get_position();
                self.display.fill_rect(x, y, 20, 20, 0x07E0); // Green rectangle at touch position
            }

            self.display.update()
        }
    }

}

