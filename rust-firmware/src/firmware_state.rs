use core::cell::{Cell, RefCell};
extern crate alloc;
use alloc::string::String;
use crate::log_info;

pub struct DeviceInfo {
    pub firmware_version: String,
    pub bootloader_version: String,
    pub serial_number: String,
    pub factory_reset_count: i32,
}

impl Default for DeviceInfo {
    fn default() -> Self {
        Self {
            firmware_version: String::new(),
            bootloader_version: String::new(),
            serial_number: String::new(),
            factory_reset_count: 0,
        }
    }
}

impl DeviceInfo {
    pub fn new(
        firmware_version: String,
        bootloader_version: String,
        serial_number: String,
        factory_reset_count: i32,
    ) -> Self {
        Self {
            firmware_version,
            bootloader_version,
            serial_number,
            factory_reset_count,
        }
    }
}

pub struct FirmwareState {
  brightness: Cell<Option<u8>>,
  battery_req: Cell<bool>,
  pin: RefCell<String>,
  unlock_req: Cell<bool>,
  is_unlocked: Cell<bool>,
  device_info_requested: Cell<bool>,
  qr_data_requested: Cell<bool>,
  scale_shown_changed: Cell<bool>,
  protocol_requested: Cell<bool>,
}

impl FirmwareState {
  pub fn new() -> Self {
      Self {
          brightness: Cell::new(None),
          battery_req: Cell::new(false),
          pin: RefCell::new(String::new()),
          unlock_req: Cell::new(false),
          is_unlocked: Cell::new(false),
          device_info_requested: Cell::new(false),
          qr_data_requested: Cell::new(false),
          scale_shown_changed: Cell::new(false),
          protocol_requested: Cell::new(false),
      }
  }
  pub fn set_brightness(&self, v: u8)                 { self.brightness.set(Some(v)); }
  pub fn set_battery_level_requested(&self, v: bool)  { self.battery_req.set(v); }
  pub fn append_to_pin(&self, digit: i32) {
      let d = digit.clamp(0, 9) as u8;             
      let ch = (b'0' + d) as char;                 
      self.pin.borrow_mut().push(ch);
  }
  pub fn mark_device_info_requested(&self)          { self.device_info_requested.set(true); }
  pub fn is_device_info_requested(&self) -> bool     { self.device_info_requested.get() }
  pub fn clear_device_info_requested(&self)          { self.device_info_requested.set(false); }
  pub fn remove_pin_char(&self)                       { self.pin.borrow_mut().pop(); }
  pub fn mark_unlock_requested(&self)                 { self.unlock_req.set(true); }
  pub fn is_battery_level_requested(&self) -> bool    { self.battery_req.get() }
  pub fn is_unlock_in_progress(&self) -> bool         { self.unlock_req.get() }
  pub fn unlock_finished(&self) {
      self.unlock_req.set(false);
      self.pin.borrow_mut().clear();
  }
  pub fn mark_protocol_requested(&self) {
      self.protocol_requested.set(true);
  }
  pub fn is_protocol_change_requested(&self) -> bool {
      self.protocol_requested.get()
  }
  pub fn clear_protocol_change_requested(&self) {
      self.protocol_requested.set(false);
  }
    pub fn mark_scale_shown_changed(&self)          { self.scale_shown_changed.set(true); }
    pub fn is_scale_shown_changed(&self) -> bool     { self.scale_shown_changed.get() }
    pub fn clear_scale_shown_changed(&self)          { self.scale_shown_changed.set(false); }
  pub fn is_qr_data_requested(&self) -> bool     { self.qr_data_requested.get() }
  pub fn mark_qr_data_requested(&self)          { self.qr_data_requested.set(true); }
  pub fn mark_qr_data_success(&self)          { self.qr_data_requested.set(false); }
  pub fn unlock_succeeded(&self)                  { self.is_unlocked.set(true); }
  pub fn is_unlock_succeeded(&self) -> bool            { self.is_unlocked.get() }
  pub fn unlock_failed(&self)                     { self.is_unlocked.set(false); }

  // --- consumed in the main loop ---
  pub fn take_brightness(&self) -> Option<u8>         { self.brightness.take() }
  pub fn take_battery_req(&self) -> bool              { self.battery_req.replace(false) }
  pub fn get_pin(&self) -> String                     { self.pin.borrow().clone() }
}
