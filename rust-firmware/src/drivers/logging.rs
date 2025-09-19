// Simple conditional logging - like C #define approach

#[cfg(feature = "minifb")]
#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        std::println!("[INFO] {}", format_args!($($arg)*));
    };
}

#[cfg(feature = "minifb")]
#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        std::println!("[DEBUG] {}", format_args!($($arg)*));
    };
}

#[cfg(feature = "minifb")]
#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        std::eprintln!("[ERROR] {}", format_args!($($arg)*));
    };
}

#[cfg(feature = "minifb")]
#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        std::println!("[WARN] {}", format_args!($($arg)*));
    };
}

#[cfg(feature = "minifb")]
#[macro_export]
macro_rules! printk {
    ($($arg:tt)*) => {
        std::println!($($arg)*);
    };
}

// For zephyr, keep the existing complex macros from zephyr/logging.rs
#[cfg(feature = "zephyr")]
pub use crate::drivers::zephyr::logging::*;