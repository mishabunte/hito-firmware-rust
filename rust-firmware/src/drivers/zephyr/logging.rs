use super::ffi;
use core::fmt::Write;

/// A buffer for formatting log messages
pub struct LogBuffer {
    buf: [u8; 256],
    len: usize,
}

impl LogBuffer {
    pub fn new() -> Self {
        Self {
            buf: [0; 256],
            len: 0,
        }
    }
    
    pub fn as_cstr(&self) -> *const core::ffi::c_char {
        // Ensure null termination
        self.buf.as_ptr() as *const core::ffi::c_char
    }
}

impl Write for LogBuffer {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let bytes = s.as_bytes();
        let remaining = self.buf.len().saturating_sub(self.len).saturating_sub(1); // Reserve 1 for null terminator
        let to_copy = bytes.len().min(remaining);
        
        if to_copy > 0 {
            self.buf[self.len..self.len + to_copy].copy_from_slice(&bytes[..to_copy]);
            self.len += to_copy;
        }
        
        // Ensure null termination
        if self.len < self.buf.len() {
            self.buf[self.len] = 0;
        }
        
        Ok(())
    }
}

/// Safe logging functions
pub fn log_info(msg: &str) {
    let mut buf = LogBuffer::new();
    let _ = write!(buf, "[INFO] {}\n\0", msg);
    unsafe {
        ffi::printk(buf.as_cstr());
    }
}

pub fn log_debug(msg: &str) {
    let mut buf = LogBuffer::new();
    let _ = write!(buf, "[DEBUG] {}\n\0", msg);
    unsafe {
        ffi::printk(buf.as_cstr());
    }
}

pub fn log_error(msg: &str) {
    let mut buf = LogBuffer::new();
    let _ = write!(buf, "[ERROR] {}\n\0", msg);
    unsafe {
        ffi::printk(buf.as_cstr());
    }
}

pub fn log_warn(msg: &str) {
    let mut buf = LogBuffer::new();
    let _ = write!(buf, "[WARN] {}\n\0", msg);
    unsafe {
        ffi::printk(buf.as_cstr());
    }
}

/// Formatted logging - like println! but for Zephyr
#[macro_export]
macro_rules! printk {
    ($($arg:tt)*) => {
        {
            use core::fmt::Write;
            let mut buf = $crate::drivers::zephyr::logging::LogBuffer::new();
            let _ = write!(buf, $($arg)*);
            let _ = write!(buf, "\n\0");
            unsafe {
                $crate::drivers::zephyr::ffi::printk(buf.as_cstr());
            }
        }
    };
}

/// Info level logging macro
#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        {
            use core::fmt::Write;
            let mut buf = $crate::drivers::zephyr::logging::LogBuffer::new();
            let _ = write!(buf, "[INFO] ");
            let _ = write!(buf, $($arg)*);
            let _ = write!(buf, "\n\0");
            unsafe {
                $crate::drivers::zephyr::ffi::printk(buf.as_cstr());
            }
        }
    };
}

/// Debug level logging macro  
#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        {
            use core::fmt::Write;
            let mut buf = $crate::drivers::zephyr::logging::LogBuffer::new();
            let _ = write!(buf, "[DEBUG] ");
            let _ = write!(buf, $($arg)*);
            let _ = write!(buf, "\n\0");
            unsafe {
                $crate::drivers::zephyr::ffi::printk(buf.as_cstr());
            }
        }
    };
}

/// Error level logging macro
#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        {
            use core::fmt::Write;
            let mut buf = $crate::drivers::zephyr::logging::LogBuffer::new();
            let _ = write!(buf, "[ERROR] ");
            let _ = write!(buf, $($arg)*);
            let _ = write!(buf, "\n\0");
            unsafe {
                $crate::drivers::zephyr::ffi::printk(buf.as_cstr());
            }
        }
    };
}

/// Warning level logging macro
#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        {
            use core::fmt::Write;
            let mut buf = $crate::drivers::zephyr::logging::LogBuffer::new();
            let _ = write!(buf, "[WARN] ");
            let _ = write!(buf, $($arg)*);
            let _ = write!(buf, "\n\0");
            unsafe {
                $crate::drivers::zephyr::ffi::printk(buf.as_cstr());
            }
        }
    };
}

