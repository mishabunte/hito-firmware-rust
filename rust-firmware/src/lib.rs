#![no_std]
extern crate alloc;
use alloc::{boxed::Box};

#[cfg(feature = "minifb")]
extern crate std;

#[cfg(feature = "zephyr")]
extern crate panic_halt;

mod hito_firmware;
mod drivers;
mod crypto;
mod platform;

pub use drivers::logging::*;

use hito_firmware::HitoFirmware;

// For now, let's use a simpler approach with the existing platform
use slint::platform::software_renderer::MinimalSoftwareWindow;

#[cfg(any(feature = "minifb", feature = "zephyr"))]
slint::include_modules!();

use crate::{drivers::{Display, Indicator, LedColor, Touch}, platform::{MyPlatform, Timer}, platform::DisplayWrapper};

#[cfg(feature = "zephyr")]
extern "C" {
    fn k_malloc(size: usize) -> *mut u8;
    fn k_free(ptr: *mut u8);
}

#[cfg(feature = "zephyr")]
struct ZephyrAllocator;

#[cfg(feature = "zephyr")]
unsafe impl core::alloc::GlobalAlloc for ZephyrAllocator {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        k_malloc(layout.size())
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: core::alloc::Layout) {
        k_free(ptr)
    }
}

#[cfg(feature = "zephyr")]
#[global_allocator]
static GLOBAL: ZephyrAllocator = ZephyrAllocator;

/// Unified main function callable from C (for embedded target)
#[no_mangle]
pub extern "C" fn rust_main() -> ! {
    let mut firmware = HitoFirmware::new();
    firmware.init_hardware();
    
    // Use ReusedBuffer for embedded displays (more memory efficient)
    let window = MinimalSoftwareWindow::new(
        slint::platform::software_renderer::RepaintBufferType::ReusedBuffer
    );
    
    window.set_size(slint::PhysicalSize::new(320, 240));

    log_info!("Creating platform");

    let platform = MyPlatform {
        window: window.clone(),
        timer: Timer::new(),
    };

    log_info!("Setting platform");

    let platform_box = Box::new(platform);

    log_info!("Platform boxed");

    slint::platform::set_platform(platform_box).unwrap();

    log_info!("Platform set successfully");

    let mut line_buffer = [slint::platform::software_renderer::Rgb565Pixel(0); 320];

    let mut last_mouse_pos = (0xffff, 0xffff);

    log_info!("Creating Slint UI");
    match MainWindow::new() {
        Ok(ui) => {
          
            log_info!("Starting Slint UI event loop");

            firmware.indicator.turn_on(LedColor::Blue);

            log_info!("Starting main UI loop with line-by-line rendering");
            
            // Super loop for embedded systems with line-by-line rendering
            loop {
                slint::platform::update_timers_and_animations();

                let is_pressed = firmware.touch.is_pressed();

                // Check the touch screen or input device using your driver.
                if is_pressed == Some(true) {
                  // convert the event from the driver into a `slint::platform::WindowEvent`
                  // and pass it to the window.
                  let pos = firmware.touch.get_position();
                  let event = slint::platform::WindowEvent::PointerPressed {
                      position: slint::LogicalPosition {
                          x: pos.0 as f32,
                          y: pos.1 as f32,
                      },
                      button: slint::platform::PointerEventButton::Left,
                  };
                  log_info!("Touch pressed at ({}, {})", pos.0, pos.1);
                  window.try_dispatch_event(event).unwrap();
                }

                if firmware.touch.has_touch() {
                    let pos = firmware.touch.get_position();
                    if last_mouse_pos != pos {
                      last_mouse_pos = pos;
                      let event = slint::platform::WindowEvent::PointerMoved {
                        position: slint::LogicalPosition {
                            x: pos.0 as f32,
                            y: pos.1 as f32,
                        },
                      };
                      window.try_dispatch_event(event).unwrap();
                    }
                  }

                if is_pressed == Some(false) {
                    let pos = firmware.touch.get_position();
                    let event = slint::platform::WindowEvent::PointerReleased {
                        position: slint::LogicalPosition {
                            x: pos.0 as f32,
                            y: pos.1 as f32,
                        },
                        button: slint::platform::PointerEventButton::Left,
                    };
                    log_info!("Touch released at ({}, {})", pos.0, pos.1);
                    window.try_dispatch_event(event).unwrap();
                }

                window.draw_if_needed(|renderer| {                    
                    // Render line by line using our display wrapper
                    renderer.render_by_line(DisplayWrapper {
                        display: &mut firmware.display,
                        line_buffer: &mut line_buffer,
                    });
                });

                firmware.display.update();
            }
        }
        Err(_) => {
            log_info!("Failed to create Slint UI");
            panic!("Failed to create Slint UI");
        }
    }
}

// ARM EABI unwinding stub for embedded targets
#[cfg(feature = "zephyr")]
#[no_mangle]
pub extern "C" fn __aeabi_unwind_cpp_pr0() {
    // Stub implementation - do nothing
    // This function is used for C++ exception unwinding, 
    // which we don't use in our no_std embedded environment
}