use crate::drivers::{Battery, Display};
use crate::{STATE, ui::CallbackController, hito_firmware::HitoFirmware};
use crate::slint_generatedMainWindow::DeviceInfoState;
use crate::slint_generatedMainWindow::ShowSeedState;
use crate::slint_generatedMainWindow::MainWindow;
extern crate alloc;
use alloc::rc::Rc;
use alloc::vec;
use slint::SharedString;
use slint::VecModel;
use slint::{ComponentHandle, ModelRc};
use crate::log_info;

pub struct DeviceInfoCallbackController;

impl CallbackController for DeviceInfoCallbackController {
    fn register_main_window_callbacks(&self, ui: &MainWindow, firmware: &mut HitoFirmware) {
        // Device info request
        ui.global::<DeviceInfoState>().on_request_device_info(move || {
            let s = STATE.get().unwrap().lock();
            s.mark_device_info_requested();
            log_info!("Device info requested");
        });
    }
    fn handle_loop_events(&self, ui: &MainWindow, firmware: &mut HitoFirmware) {
        let s = STATE.get().unwrap().lock();
        // Device info request handling
        let device_info_state = ui.global::<DeviceInfoState>();
        let show_seed_state = ui.global::<ShowSeedState>();
        if s.is_device_info_requested() {
            log_info!("Providing device info to UI");
            let info = firmware.vault.get_device_info().unwrap();
            let mnemonic = firmware.vault.get_mnemonic().expect("Error getting mnemonic");
            log_info!("Mnemonic retrieved: {}\n", mnemonic);    
            log_info!("Device info: FW ver {}, BL ver {}, SN {}, FR count {}",
                info.firmware_version, info.bootloader_version, info.serial_number, info.factory_reset_count);
            device_info_state.set_firmware_version(slint::SharedString::from(&info.firmware_version[10..]));
            device_info_state.set_bootloader_version(slint::SharedString::from(&info.bootloader_version));
            device_info_state.set_serial_number(slint::SharedString::from(&info.serial_number));
            device_info_state.set_factory_reset_count(info.factory_reset_count);
            device_info_state.invoke_request_factory_reset_string();
            let the_model : Rc<VecModel<SharedString>> =
                Rc::new(VecModel::from(mnemonic
                    .split(' ')
                    .map(|s| SharedString::from(s))
                    .collect::<alloc::vec::Vec<SharedString>>()));
            let the_model_rc = ModelRc::from(the_model.clone());
            show_seed_state.set_seed_phrase(the_model_rc);
            s.clear_device_info_requested();
        }
    }
}