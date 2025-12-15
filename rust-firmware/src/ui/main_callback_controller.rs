use crate::drivers::{Battery, Display};
use crate::{STATE, ui::CallbackController, hito_firmware::HitoFirmware};
use crate::slint_generatedMainWindow::BrightnessController;
use crate::slint_generatedMainWindow::MainWindow;
use crate::slint_generatedMainWindow::BatteryController;

#[cfg(feature = "zephyr")]
use crate::slint_generatedMainWindow::{StartScreen, ScreenEnum};

use slint::ComponentHandle;
use crate::{log_info, ui};

pub struct MainCallbackController;

impl CallbackController for MainCallbackController {
    fn register_main_window_callbacks(&self, ui: &MainWindow, firmware: &mut HitoFirmware) {
      // Brightness
      let display_arc = firmware.display.clone();
      ui.global::<BrightnessController>().on_brightness_changed(move |v: i32| {
          display_arc.lock().set_brightness(v as u8);
      });

      #[cfg(feature = "zephyr")]
      ui.global::<StartScreen>().set_screen(ScreenEnum::Lock);

      // Battery 
      let ui_weak = ui.as_weak();
      let battery_arc = firmware.battery.clone();
      ui.global::<BatteryController>().on_battery_level_request(move || {
          let level = battery_arc.lock().get_level();
          // log_info!("Battery level requested: {}", level);
          if let Some(ui) = ui_weak.upgrade() {
              let battery = ui.global::<BatteryController>();
              battery.set_battery_level(level);
          }
      });
    }
    fn handle_loop_events(&self, _ui: &MainWindow, _firmware: &mut HitoFirmware) {
      // All logic now handled in callbacks
    }
}