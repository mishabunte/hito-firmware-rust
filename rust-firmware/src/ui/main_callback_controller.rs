use crate::drivers::{Battery, Display};
use crate::{STATE, ui::CallbackController, hito_firmware::HitoFirmware};
use crate::slint_generatedMainWindow::BrightnessController;
use crate::slint_generatedMainWindow::MainWindow;
use crate::slint_generatedMainWindow::BatteryController;

#[cfg(feature = "zephyr")]
use crate::slint_generatedMainWindow::{StartScreen, ScreenEnum};

use slint::ComponentHandle;
use crate::log_info;

pub struct MainCallbackController;

impl CallbackController for MainCallbackController {
    fn register_main_window_callbacks(&self, ui: &MainWindow, firmware: &mut HitoFirmware) {
      // Brightness
      ui.global::<BrightnessController>().on_brightness_changed(move |v: i32| {
          let s = STATE.get().unwrap().lock();
          s.set_brightness(v as u8);
      });

      #[cfg(feature = "zephyr")]
      ui.global::<StartScreen>().set_screen(ScreenEnum::Lock);

      // Battery 
      ui.global::<BatteryController>().on_battery_level_request(move || {
          let s = STATE.get().unwrap().lock();
          s.set_battery_level_requested(true);
      });
    }
    fn handle_loop_events(&self, ui: &MainWindow, firmware: &mut HitoFirmware) {
      let s = STATE.get().unwrap().lock();
      if let Some(new_brightness) = s.take_brightness() {
          firmware.display.set_brightness(new_brightness);
          // log_info!("Brightness set to {}", new_brightness);
      }
      let battery = ui.global::<BatteryController>();
      if s.is_battery_level_requested() {
          let level = firmware.battery.get_level();
          // log_info!("Battery level requested: {}", level);
          battery.set_battery_level(level);
          s.set_battery_level_requested(false);
      }
    }
}