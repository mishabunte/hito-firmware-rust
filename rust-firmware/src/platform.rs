extern crate alloc;
use alloc::{rc::Rc, boxed::Box};
use slint::platform::{Platform, software_renderer::MinimalSoftwareWindow};
use slint::platform::software_renderer::{LineBufferProvider, Rgb565Pixel};
use crate::drivers::{Display, DisplayImpl};

#[cfg(feature = "zephyr")]
use crate::drivers::zephyr::timer::ZephyrTimer;
use crate::log_info;

#[cfg(feature = "minifb")]
use std::time::{SystemTime, UNIX_EPOCH};

pub struct Timer {
    start_time: u64,
}

impl Timer {
    pub fn new() -> Self {
        let start_time = {
            #[cfg(feature = "minifb")]
            {
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_nanos() as u64
            }
            #[cfg(feature = "zephyr")]
            {
                ZephyrTimer::get_time()
            }
        };
        Timer { start_time }
    }

    pub fn get_time(&self) -> u64 {
        #[cfg(feature = "minifb")]
        {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos() as u64
        }
        #[cfg(feature = "zephyr")]
        {
            ZephyrTimer::get_time()
        }
    }
}

pub struct MyPlatform {
    pub window: Rc<MinimalSoftwareWindow>,
    pub timer: Timer,
}

impl Platform for MyPlatform {
    fn create_window_adapter(&self) -> Result<Rc<dyn slint::platform::WindowAdapter>, slint::PlatformError> {
        // Since on MCUs, there can be only one window, just return a clone of self.window.
        Ok(self.window.clone())
    }
    
    fn duration_since_start(&self) -> core::time::Duration {
        let time = self.timer.get_time();
        core::time::Duration::from_nanos(time.into())
    }
}

pub struct DisplayWrapper<'a> {
    pub display: &'a mut DisplayImpl,
    pub line_buffer: &'a mut [Rgb565Pixel],
}

impl<'a> LineBufferProvider for DisplayWrapper<'a> {
    type TargetPixel = Rgb565Pixel;

    fn process_line(
        &mut self,
        line: usize,
        range: core::ops::Range<usize>,
        render_fn: impl FnOnce(&mut [Self::TargetPixel]),
    ) {
        // Render into the line buffer
        render_fn(&mut self.line_buffer[range.clone()]);

        // Convert Rgb565Pixel to raw u16 values and send to display
        let raw_pixels: &[u16] = unsafe {
            core::slice::from_raw_parts(
                self.line_buffer[range.clone()].as_ptr() as *const u16,
                range.len()
            )
        };

        // Send the line to the display
        self.display.draw_line(
            line as u16,
            range.start as u16,
            range.end as u16,
            raw_pixels,
        );
    }
}