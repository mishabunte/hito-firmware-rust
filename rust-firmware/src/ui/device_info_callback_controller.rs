use crate::log_info;
use crate::{STATE, ui::CallbackController, hito_firmware::HitoFirmware};
use crate::slint_generatedMainWindow::DeviceInfoState;
use crate::slint_generatedMainWindow::ShowSeedState;
use crate::slint_generatedMainWindow::MainWindow;
extern crate alloc;
use slint::SharedString;
use slint::{ComponentHandle};
use alloc::vec::Vec;
use alloc::format;

const WORDS_PER_PAGE: usize = 8;

pub struct DeviceInfoCallbackController;

impl CallbackController for DeviceInfoCallbackController {
    fn register_main_window_callbacks(&self, ui: &MainWindow, firmware: &mut HitoFirmware) {
        // Device info request
        ui.global::<DeviceInfoState>().on_request_device_info(move || {
            let s = STATE.get().unwrap().lock();
            s.mark_device_info_requested();
            // log_info!("Device info requested");
        });
        let ui_weak = ui.as_weak();

        ui.global::<ShowSeedState>().on_get_word(move |index| {
            // Try to upgrade the weak handle each time the callback is called
            if let Some(ui) = ui_weak.upgrade() {
                let show_seed = ui.global::<ShowSeedState>();
                let phrase = show_seed.get_seed_phrase_fragment(); // SharedString

                let word = phrase
                    .split_whitespace()
                    .nth(index as usize)
                    .unwrap_or("");

                SharedString::from(word)
            } else {
                // UI was destroyed
                SharedString::new()
            }
        });
    }
    fn handle_loop_events(&self, ui: &MainWindow, firmware: &mut HitoFirmware) {
        let s = STATE.get().unwrap().lock();
        // Device info request handling
        let device_info_state = ui.global::<DeviceInfoState>();
        let show_seed_state = ui.global::<ShowSeedState>();
        if s.is_device_info_requested() {
            let info = firmware.vault.get_device_info().unwrap();  
            device_info_state.set_firmware_version(slint::SharedString::from(&info.firmware_version[10..]));
            device_info_state.set_bootloader_version(slint::SharedString::from(&info.bootloader_version));
            device_info_state.set_serial_number(slint::SharedString::from(&info.serial_number));
            if info.factory_reset_count < 0 {
              device_info_state.set_factory_reset_string(slint::SharedString::from("Factory reset no data"));
            } else if info.factory_reset_count == 0  {
              device_info_state.set_factory_reset_string(slint::SharedString::from("Factory setup"));
            } else if info.factory_reset_count > 30 {
              device_info_state.set_factory_reset_string(slint::SharedString::from("Factory reset 30 plus"));
            } else {
              device_info_state.set_factory_reset_string(slint::SharedString::from(&format!("Factory reset {} times", info.factory_reset_count)));
            }
            show_seed_state.set_seed_phrase_fragment(slint::SharedString::from(&firmware.vault.get_mnemonic().expect("Error getting mnemonic")));
            show_seed_state.set_seed_len(firmware.vault.get_mnemonic_len() as i32);
            show_seed_state.set_total_pages((firmware.vault.get_mnemonic_len() / (WORDS_PER_PAGE + 1) + 1) as i32);
            s.clear_device_info_requested();
        }
    }
}