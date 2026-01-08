use core::cell::{Cell, RefCell};
extern crate alloc;
use alloc::string::String;
use crate::log_info;
use crate::crypto::libcrypt0pro::stellar::{TransactionEnvelope};

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
  pin: RefCell<String>,
  derive_key_requested: Cell<bool>,
  is_unlocked: Cell<bool>,
  device_info_requested: Cell<bool>,
  qr_data_requested: Cell<bool>,
  scale_shown_changed: Cell<bool>,
  protocol_requested: Cell<bool>,
  parsed_tx: RefCell<Option<TransactionEnvelope>>,
  stellar_address: RefCell<Option<String>>,
  seed_length: RefCell<Option<usize>>,
}

impl FirmwareState {
  pub fn new() -> Self {
      Self {
          brightness: Cell::new(None),
          pin: RefCell::new(String::new()),
          derive_key_requested: Cell::new(false),
          is_unlocked: Cell::new(false),
          device_info_requested: Cell::new(false),
          qr_data_requested: Cell::new(false),
          scale_shown_changed: Cell::new(false),
          protocol_requested: Cell::new(false),
          parsed_tx: RefCell::new(None),  
          stellar_address: RefCell::new(None),
          seed_length: RefCell::new(Some(12)),
      }
  }
  pub fn set_brightness(&self, v: u8)                 { self.brightness.set(Some(v)); }
  pub fn append_to_pin(&self, digit: u8) {
      let ch = (b'0' + digit) as char;
      self.pin.borrow_mut().push(ch);
  }
  pub fn mark_device_info_requested(&self)          { self.device_info_requested.set(true); }
  pub fn is_device_info_requested(&self) -> bool     { self.device_info_requested.get() }
  pub fn clear_device_info_requested(&self)          { self.device_info_requested.set(false); }
  pub fn remove_pin_char(&self)                       { self.pin.borrow_mut().pop(); }
  pub fn mark_derive_key_requested(&self)                 { self.derive_key_requested.set(true); }
  pub fn is_derive_key_in_progress(&self) -> bool         { self.derive_key_requested.get() }
  pub fn derive_key_finished(&self) {
      self.derive_key_requested.set(false);
      self.pin.borrow_mut().clear();
  }

  pub fn set_seed_length(&self, length: usize) {
      self.seed_length.borrow_mut().replace(length);
  }
  pub fn get_seed_length(&self) -> Option<usize> {
      self.seed_length.borrow().clone()
  }

  pub fn set_stellar_address(&self, address: String) {
      self.stellar_address.borrow_mut().replace(address);
  }
  pub fn get_stellar_address(&self) -> Option<String> {
      self.stellar_address.borrow().clone()
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
  pub fn get_parsed_tx(&self) -> Option<TransactionEnvelope> {
      self.parsed_tx.borrow().clone()
  }
  pub fn set_parsed_tx(&self, parsed_tx: TransactionEnvelope) {
    self.parsed_tx.borrow_mut().replace(parsed_tx);
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
  pub fn get_pin(&self) -> String                     { self.pin.borrow().clone() }
  pub fn set_pin(&self, pin: String)              { *self.pin.borrow_mut() = pin; }
}
