#![no_std]
#![allow(dead_code)]
#[cfg(any(feature = "minifb", feature = "zephyr"))]
slint::include_modules!();
extern crate alloc;
use alloc::{boxed::Box, rc::Rc};
use core::mem::MaybeUninit;

#[cfg(feature = "minifb")]
extern crate std;

#[cfg(feature = "minifb")]
use std::sync::atomic::{AtomicUsize, Ordering};

#[cfg(feature = "zephyr")]
extern crate panic_halt;

pub use common::QR_CODE;

mod hito_firmware;
pub mod drivers; // TODO change to private
pub mod crypto {
    pub mod ffi;
    pub mod crypt0;
    pub mod libcrypt0pro {
        pub mod stellar {
            mod address;  // This will look for address.rs
            pub use address::*;
            
            mod transaction_parser;
            mod transaction_serializer;
            pub use transaction_parser::*;
            pub use transaction_serializer::*;
        }
    }
}
mod platform;
mod vault;
mod firmware_state;
mod ui;
mod common;

pub use common::QR_CODE;

pub use vault::vault::{HitoVault, VaultError, VaultResult};
pub use firmware_state::DeviceInfo;

use hito_firmware::HitoFirmware;
use firmware_state::FirmwareState;
use slint::platform::software_renderer::MinimalSoftwareWindow;

use crate::ui::register_main_window_callbacks;

slint::include_modules!();

#[cfg(feature = "minifb")]
static BASE_STACK_REMAINING: AtomicUsize = AtomicUsize::new(8388608);

use spin::{Once, Mutex};

//use crate::ui::UI_CALLBACK_CONTROLLERS;

static STATE: Once<Mutex<FirmwareState>> = Once::new();

#[cfg(feature = "minifb")]
pub fn current_stack_used() -> usize {
    let base = BASE_STACK_REMAINING.load(Ordering::Relaxed);
    let now  = stacker::remaining_stack();
    return base.saturating_sub(now.unwrap_or(0))
}

use crate::{
    drivers::{Display, Indicator, LedColor, Touch}, 
    platform::{DisplayWrapper, MyPlatform, Timer}
};

const INVALID_MOUSE_POS: (u16, u16) = (0xffff, 0xffff);

// For desktop, we can use regular static storage
pub static mut LINE_BUFFER: [slint::platform::software_renderer::Rgb565Pixel; 320] = 
    [slint::platform::software_renderer::Rgb565Pixel(0); 320];

pub static mut PLATFORM_STORAGE: MaybeUninit<MyPlatform> = MaybeUninit::uninit();

pub static mut LAST_MOUSE_POS: (u16, u16) = INVALID_MOUSE_POS;

#[cfg(feature = "minifb")]
static HEAP_ALLOCATED: AtomicUsize = AtomicUsize::new(0);

#[cfg(feature = "minifb")]
struct TrackingAllocator;

#[cfg(feature = "minifb")]
unsafe impl std::alloc::GlobalAlloc for TrackingAllocator {
    unsafe fn alloc(&self, layout: std::alloc::Layout) -> *mut u8 {
        let ptr = std::alloc::System.alloc(layout);
        if !ptr.is_null() {
            HEAP_ALLOCATED.fetch_add(layout.size(), Ordering::Relaxed);
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: std::alloc::Layout) {
        HEAP_ALLOCATED.fetch_sub(layout.size(), Ordering::Relaxed);
        std::alloc::System.dealloc(ptr, layout);
    }
}

#[cfg(feature = "minifb")]
#[global_allocator]
static GLOBAL: dhat::Alloc = dhat::Alloc;

#[cfg(feature = "minifb")]
pub fn get_heap_usage() -> usize {
    dhat::HeapStats::get().curr_bytes
}

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
    let is_pressed = firmware.touch.lock().is_pressed();
    
    match is_pressed {
        Some(true) => {
            let pos = firmware.touch.lock().get_position();
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
            let pos = firmware.touch.lock().get_position();
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
    if firmware.touch.lock().has_touch() {
        let pos = firmware.touch.lock().get_position();
        
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

#[inline]
pub fn now_us() -> u64 {
    unsafe {
        PLATFORM_STORAGE.assume_init_ref().timer.get_time() / 1_000
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
            slint::platform::software_renderer::RepaintBufferType::ReusedBuffer,
    );

    window.set_size(slint::PhysicalSize::new(320, 240));

    // log_info!("Initializing platform");

    // Initialize platform (common code)
    initialize_platform(window.clone());

    #[cfg(feature = "minifb")]
    let _profiler = dhat::Profiler::builder().build();

    //calculate_sizeof_screens();

    log_info!("Platform initialized");

    STATE.call_once(|| Mutex::new(FirmwareState::new()));
    // let state = FirmwareState::new();
    firmware.indicator.lock().turn_on(LedColor::Blue);
<<<<<<< HEAD
    log_info!("Starting embedded event loop");
=======
    // // log_info!("Starting embedded event loop");
>>>>>>> d8c3b522630af50ef5b218905f7180311b828eba
    
    // Run platform-specific main loop
    let ui = Rc::new(MainWindow::new().unwrap());

    register_main_window_callbacks(&ui, &mut firmware);
    
    // Initialize the global router and navigate to the initial screen (Menu)
    ui::init_global_router(ui.clone());

    // // Original code not for testing
    // ui.set_is_lockscreen(true);

    ui::navigate_to(ui::screens::Screen::EnterPasscode);

    loop {
        // Process any pending navigation requests (deferred from callbacks)
        ui::process_pending_navigation();

        slint::platform::update_timers_and_animations();
        
        handle_touch_events(&mut firmware, &*window);

        window.draw_if_needed(|renderer| {
            unsafe {
                renderer.render_by_line(DisplayWrapper {
                    display: &mut firmware.display,
                    line_buffer: &mut LINE_BUFFER,
                });
              let display_arc = firmware.display.clone();
              unsafe {
                if QR_CODE.is_some() {
                    let qr_data = QR_CODE.as_ref().unwrap().get_data();
                    let qr_width = QR_CODE.as_ref().unwrap().get_width() as usize;
                    let (x, y) = QR_CODE.as_ref().unwrap().get_coords();
                    display_arc.lock().draw_qr_from_buffer(x, y, &qr_data, qr_width);
                }
              }
            }
        });

        firmware.display.lock().update();
    }
}
// ARM EABI unwinding stub for embedded targets only
#[cfg(feature = "zephyr")]
#[no_mangle]
pub extern "C" fn __aeabi_unwind_cpp_pr0() {
    // Stub for C++ exception unwinding (unused in no_std)
}