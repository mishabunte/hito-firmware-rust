use core::cell::{Cell, RefCell};
use alloc::string::String;
use crate::log_info;

pub struct FirmwareState {
  brightness: Cell<Option<u8>>,
  battery_req: Cell<bool>,
  pin: RefCell<String>,
  unlock_req: Cell<bool>,
  is_unlocked: Cell<bool>,
}

impl FirmwareState {
  pub fn new() -> Self {
      Self {
          brightness: Cell::new(None),
          battery_req: Cell::new(false),
          pin: RefCell::new(String::new()),
          unlock_req: Cell::new(false),
          is_unlocked: Cell::new(false),
      }
  }
  pub fn set_brightness(&self, v: u8)                 { self.brightness.set(Some(v)); }
  pub fn set_battery_level_requested(&self, v: bool)  { self.battery_req.set(v); }
  pub fn append_to_pin(&self, digit: i32) {
      let d = digit.clamp(0, 9) as u8;             
      let ch = (b'0' + d) as char;                 
      self.pin.borrow_mut().push(ch);
  }
  pub fn remove_pin_char(&self)                       { self.pin.borrow_mut().pop(); }
  pub fn mark_unlock_requested(&self)                 { self.unlock_req.set(true); }
  pub fn is_battery_level_requested(&self) -> bool    { self.battery_req.get() }
  pub fn is_unlock_in_progress(&self) -> bool         { self.unlock_req.get() }
  pub fn unlock_finished(&self) {
      self.unlock_req.set(false);
      self.pin.borrow_mut().clear();
  }
  pub fn unlock_succeeded(&self)                  { self.is_unlocked.set(true); }
  pub fn is_unlock_succeeded(&self) -> bool            { self.is_unlocked.get() }
  pub fn unlock_failed(&self)                     { self.is_unlocked.set(false); }

  // --- consumed in the main loop ---
  pub fn take_brightness(&self) -> Option<u8>         { self.brightness.take() }
  pub fn take_battery_req(&self) -> bool              { self.battery_req.replace(false) }
  pub fn get_pin(&self) -> String                     { self.pin.borrow().clone() }
}
