use crate::slint_generatedMainWindow::MainWindow;

use crate::drivers::{Battery, Display};
use crate::firmware;

use slint::ComponentHandle;

use crate::slint_generatedMainWindow::BrightnessController;
use crate::slint_generatedMainWindow::BatteryController;

pub fn register_main_window_callbacks(ui: &MainWindow) {
      // Brightness
      let display_arc = firmware().display.clone();
      ui.global::<BrightnessController>().on_brightness_changed(move |v: i32| {
          display_arc.lock().set_brightness(v as u8);
      });

      // Battery 
      let ui_weak = ui.as_weak();
      let battery_arc = firmware().battery.clone();
      ui.global::<BatteryController>().on_battery_level_request(move || {
          let level = battery_arc.lock().get_level();
          // log_info!("Battery level requested: {}", level);
          if let Some(ui) = ui_weak.upgrade() {
              let battery = ui.global::<BatteryController>();
              battery.set_battery_level(level);
          }
      });

      let ui_weak = ui.as_weak();
      ui.on_unlock_requested(move || {
          if let Some(ui) = ui_weak.upgrade() {
              ui.set_is_lockscreen(false);
              if firmware().vault.lock().is_empty() {
                navigate_to(screens::Screen::SetNewPasscode);
                return;
              }
              navigate_to(screens::Screen::EnterPasscode);
          }
      });

      ui.on_go_back(move || {
          go_back();
      });
}

pub mod router;
pub mod screens;

pub use router::{Router, init_global_router, navigate_to, go_back, process_pending_navigation, current_screen};


