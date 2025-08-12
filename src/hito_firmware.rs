use crate::drivers::{ Display,   DisplayImpl };
use crate::drivers::{ Touch,     TouchImpl };
use crate::drivers::{ Indicator, IndicatorImpl };

pub struct HitoFirmware {
    pub display:   Box<dyn Display>,
    pub touch:     Box<dyn Touch>,
    pub indicator: Box<dyn Indicator>,
}


impl HitoFirmware {
    pub fn new() -> Self {
        Self {
            indicator: Box::new(IndicatorImpl::new()),
            display:   Box::new(DisplayImpl::new()),
            touch:     Box::new(TouchImpl::new()),
        }
    }

    pub fn init_hardware(&mut self) {

        self.indicator.init();
        self.display.init();
        self.touch.init();

        //self.display.fill_rect(10, 10, 100, 50, 0xF800); // Red rectangle
    }

    pub fn main_loop(&mut self) {

        let mut x = 0xffff; 
        let mut y = 0xffff;

        loop {
            if self.touch.is_pressed() {

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

