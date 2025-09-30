#![no_std]
extern crate alloc;
use alloc::{boxed::Box, rc::Rc};
use core::mem::MaybeUninit;

#[cfg(feature = "minifb")]
extern crate std;

#[cfg(feature = "zephyr")]
extern crate panic_halt;

mod hito_firmware;
mod drivers;
mod crypto;
mod platform;
mod ui;

use hito_firmware::HitoFirmware;
use slint::platform::software_renderer::MinimalSoftwareWindow;
use ui::{ScreenManager};

#[cfg(any(feature = "minifb", feature = "zephyr"))]
slint::include_modules!();

use crate::{
    drivers::{Battery, Display, Indicator, LedColor, Touch}, 
    platform::{DisplayWrapper, MyPlatform, Timer}, ui::screen_manager::ScreenType
};

// Constants in flash memory
const DISPLAY_WIDTH: u32 = 320;
const DISPLAY_HEIGHT: u32 = 240;
const INVALID_MOUSE_POS: (u16, u16) = (0xffff, 0xffff);

// For desktop, we can use regular static storage
pub static mut LINE_BUFFER: [slint::platform::software_renderer::Rgb565Pixel; 320] = 
    [slint::platform::software_renderer::Rgb565Pixel(0); 320];

pub static mut PLATFORM_STORAGE: MaybeUninit<MyPlatform> = MaybeUninit::uninit();

pub static mut LAST_MOUSE_POS: (u16, u16) = INVALID_MOUSE_POS;

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

fn handle_touch_events(
    firmware: &mut HitoFirmware,
    window: &MinimalSoftwareWindow,
) {
    let is_pressed = firmware.touch.is_pressed();
    
    match is_pressed {
        Some(true) => {
            let pos = firmware.touch.get_position();
            let event = slint::platform::WindowEvent::PointerPressed {
                position: slint::LogicalPosition {
                    x: pos.0 as f32,
                    y: pos.1 as f32,
                },
                button: slint::platform::PointerEventButton::Left,
            };
            
            let _ = window.try_dispatch_event(event);
        }
        Some(false) => {
            let pos = firmware.touch.get_position();
            let event = slint::platform::WindowEvent::PointerReleased {
                position: slint::LogicalPosition {
                    x: pos.0 as f32,
                    y: pos.1 as f32,
                },
                button: slint::platform::PointerEventButton::Left,
            };
            
            let _ = window.try_dispatch_event(event);
        }
        None => {}
    }
    if firmware.touch.has_touch() {
        let pos = firmware.touch.get_position();
        
        unsafe {
            if LAST_MOUSE_POS != pos {
                LAST_MOUSE_POS = pos;
                let event = slint::platform::WindowEvent::PointerMoved {
                    position: slint::LogicalPosition {
                        x: pos.0 as f32,
                        y: pos.1 as f32,
                    },
                };
                let _ = window.try_dispatch_event(event);
            }
        }
    }
}

fn run_main_loop(
    mut firmware: HitoFirmware,
    window: Rc<MinimalSoftwareWindow>,
    mut screen_manager: ScreenManager,
) -> ! {
    firmware.indicator.turn_on(LedColor::Blue);
    log_info!("Starting embedded event loop");

    loop {
        slint::platform::update_timers_and_animations();
        let brightness_request = screen_manager.brightness_request.clone();
        let battery_request = screen_manager.battery_request.clone();

        // Handle brightness changes
        if let Some(new_brightness) = brightness_request.take() {
            firmware.display.set_brightness(new_brightness);
            log_info!("Brightness set to {}", new_brightness);
        }

        // Handle battery requests (currently no UI to update, but keep for future)
        if battery_request.replace(false) {
            let level = firmware.battery.get_level();
            log_info!("Battery level: {}", level);
        }

        // Handle touch events
        handle_touch_events(&mut firmware, &*window);

        // Check for screen navigation (e.g., LockScreen unlock)
        let _ = screen_manager.handle_navigation();

        // Render frame using static line buffer
        window.draw_if_needed(|renderer| {
            unsafe {
                renderer.render_by_line(DisplayWrapper {
                    display: &mut firmware.display,
                    line_buffer: &mut LINE_BUFFER,
                });
            }
        });

        firmware.display.update();
    }
}

// Common initialization function
fn initialize_platform(window: Rc<MinimalSoftwareWindow>) {
    unsafe {
        let platform = MyPlatform {
            window: window,
            timer: Timer::new(),
        };
        
        PLATFORM_STORAGE.write(platform);
        let platform_box = Box::from_raw(PLATFORM_STORAGE.as_mut_ptr());
        slint::platform::set_platform(platform_box).unwrap();
    }
}

/// Unified main function callable from C (for embedded target) or regular main (for desktop)
#[no_mangle]
pub extern "C" fn rust_main() -> ! {
    // Initialize firmware
    let mut firmware = HitoFirmware::new();
    firmware.init_hardware();
    
    // Create window with appropriate buffer type
    let window = MinimalSoftwareWindow::new(
            slint::platform::software_renderer::RepaintBufferType::ReusedBuffer
    );

    window.set_size(slint::PhysicalSize::new(DISPLAY_WIDTH, DISPLAY_HEIGHT));

    log_info!("Initializing platform");

    // Initialize platform (common code)
    initialize_platform(window.clone());

    log_info!("Platform initialized");

    // Create working screen manager
    let mut screen_manager = ScreenManager::new().expect("Failed to create screen manager");

    // Start with the Lock screen
    screen_manager.navigate_to(ScreenType::Lock).unwrap();

    // Run platform-specific main loop
    run_main_loop(firmware, window, screen_manager)
}
// ARM EABI unwinding stub for embedded targets only
#[cfg(feature = "zephyr")]
#[no_mangle]
pub extern "C" fn __aeabi_unwind_cpp_pr0() {
    // Stub for C++ exception unwinding (unused in no_std)
}