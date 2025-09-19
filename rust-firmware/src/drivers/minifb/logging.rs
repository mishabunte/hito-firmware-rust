#[cfg(feature = "minifb")]
use std::{println, eprintln};

use crate::drivers::logging::Logger;

/// MiniFB logger implementation that prints to stdout
pub struct MiniFbLogger;

impl Logger for MiniFbLogger {
    fn log_info(&self, msg: &str) {
        #[cfg(feature = "minifb")]
        println!("[INFO] {}", msg);
    }
    
    fn log_debug(&self, msg: &str) {
        #[cfg(feature = "minifb")]
        println!("[DEBUG] {}", msg);
    }
    
    fn log_error(&self, msg: &str) {
        #[cfg(feature = "minifb")]
        eprintln!("[ERROR] {}", msg);
    }
    
    fn log_warn(&self, msg: &str) {
        #[cfg(feature = "minifb")]
        println!("[WARN] {}", msg);
    }
    
    fn log_raw(&self, msg: &str) {
        #[cfg(feature = "minifb")]
        println!("{}", msg);
    }
}

/// Global instance of the MiniFB logger
pub static MINIFB_LOGGER: MiniFbLogger = MiniFbLogger;

/// Formatted logging macro
#[macro_export]
macro_rules! printk {
    ($($arg:tt)*) => {
        {
            $crate::drivers::logging::format_and_log_raw(format_args!($($arg)*));
        }
    };
}

/// Info level logging macro
#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        {
            $crate::drivers::logging::format_and_log_info(format_args!($($arg)*));
        }
    };
}

/// Debug level logging macro  
#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        {
            $crate::drivers::logging::format_and_log_debug(format_args!($($arg)*));
        }
    };
}

/// Error level logging macro
#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        {
            $crate::drivers::logging::format_and_log_error(format_args!($($arg)*));
        }
    };
}

/// Warning level logging macro
#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        {
            $crate::drivers::logging::format_and_log_warn(format_args!($($arg)*));
        }
    };
}