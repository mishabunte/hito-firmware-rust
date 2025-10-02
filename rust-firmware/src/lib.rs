#![no_std]
extern crate alloc;
use alloc::{boxed::Box, rc::Rc};
use core::{cell::Cell, mem::MaybeUninit};

#[cfg(feature = "minifb")]
extern crate std;

#[cfg(feature = "minifb")]
use std::sync::atomic::{AtomicUsize, Ordering};

#[cfg(feature = "zephyr")]
extern crate panic_halt;

mod hito_firmware;
mod drivers;
mod crypto;
mod platform;
pub mod vault;

pub use vault::{HitoVault, VaultError, VaultResult};

use hito_firmware::HitoFirmware;
use slint::platform::software_renderer::MinimalSoftwareWindow;

#[cfg(any(feature = "minifb", feature = "zephyr"))]
slint::include_modules!();

#[cfg(feature = "minifb")]
static BASE_STACK_REMAINING: AtomicUsize = AtomicUsize::new(8388608);

#[cfg(feature = "minifb")]
pub fn init_stack_baseline() {
    // let rem = stacker::remaining_stack();
    // log_info!("Initial stack remaining: {:?}", rem);
    // BASE_STACK_REMAINING.store(rem.unwrap_or(0), Ordering::Relaxed);
}

#[cfg(feature = "minifb")]
pub fn current_stack_used() -> usize {
    let base = BASE_STACK_REMAINING.load(Ordering::Relaxed);
    let now  = stacker::remaining_stack();
    return base.saturating_sub(now.unwrap_or(0))
}

use crate::{
    drivers::{Battery, Display, Indicator, LedColor, Touch}, 
    platform::{DisplayWrapper, MyPlatform, Timer},
};


// Constants in flash memory
const DISPLAY_WIDTH: usize = 320;
const DISPLAY_HEIGHT: usize = 240;
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
) -> ! {
    firmware.indicator.turn_on(LedColor::Blue);
    log_info!("Starting embedded event loop");

    let ui = MainWindow::new().expect("Failed to create MainWindow");

    let brightness_request = Rc::new(Cell::new(None));
    {
      let br = brightness_request.clone();
      ui.global::<BrightnessController>().on_brightness_changed(move |brightness_value| {
          br.set(Some(brightness_value as u8));
      });
    }

    let battery_request: Rc<Cell<bool>> = Rc::new(Cell::new(false));
    let battery = ui.global::<BatteryController>();
    {
        let br = battery_request.clone();
        battery.on_battery_level_request(move || {
            br.set(true);
        });
    }

    loop {
        slint::platform::update_timers_and_animations();

        // Handle brightness changes
        if let Some(new_brightness) = brightness_request.take() {
            firmware.display.set_brightness(new_brightness);
            log_info!("Brightness set to {}", new_brightness);
        }

        if battery_request.get() {
            let level = firmware.battery.get_level();
            log_info!("Battery level requested: {}", level);
            battery.set_battery_level(level);
            battery_request.set(false);
        }

        // Handle touch events
        handle_touch_events(&mut firmware, &*window);

        // Render frame using static line buffer
        window.draw_if_needed(|renderer| {
            unsafe {
                #[cfg(feature = "minifb")]
                {
                    let heap_bytes = get_heap_usage();
                    // Approximate stack usage - rough estimate based on thread stack
                    let stack_bytes = current_stack_used(); // Placeholder - actual measurement is platform-specific
                    drivers::minifb::simulator_window_set_memory_stats(heap_bytes, stack_bytes);
                }
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
            slint::platform::software_renderer::RepaintBufferType::ReusedBuffer,
    );

    window.set_size(slint::PhysicalSize::new(320, 240));

    log_info!("Initializing platform");

    // Initialize platform (common code)
    initialize_platform(window.clone());

    #[cfg(feature = "minifb")]
    let _profiler = dhat::Profiler::builder().build();

    log_info!("Platform initialized");

    let mut vault = HitoVault::new();
    log_info!("Vault created");
    vault.initialize();
    log_info!("Vault initialized");
    //vault.set_passcode(b"000000", None).expect("Failed to set passcode");
    log_info!("Passcode set");
    //vault.unlock_with_password(b"001000").expect("Failed to unlock vault");
    vault.unlock_with_password(b"000000").expect("Failed to unlock vault");
    log_info!("Vault unlocked");

    // Run platform-specific main loop
    run_main_loop(firmware, window);
}
// ARM EABI unwinding stub for embedded targets only
#[cfg(feature = "zephyr")]
#[no_mangle]
pub extern "C" fn __aeabi_unwind_cpp_pr0() {
    // Stub for C++ exception unwinding (unused in no_std)
}