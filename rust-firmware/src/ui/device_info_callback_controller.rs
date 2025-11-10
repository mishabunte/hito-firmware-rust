use crate::drivers::{Battery, Display};
use crate::{STATE, ui::CallbackController, hito_firmware::HitoFirmware};
use crate::slint_generatedMainWindow::DeviceInfoController;
use crate::slint_generatedMainWindow::MainWindow;
use slint::ComponentHandle;
use crate::log_info;

pub struct DeviceInfoCallbackController;

impl CallbackController for DeviceInfoCallbackController {
    fn register_main_window_callbacks(&self, ui: &MainWindow, firmware: &mut HitoFirmware) {
        // Device info request
        ui.global::<DeviceInfoController>().on_request_device_info(move || {
            let s = STATE.get().unwrap().lock();
            s.mark_device_info_requested();
            log_info!("Device info requested");
        });
    }
    fn handle_loop_events(&self, ui: &MainWindow, firmware: &mut HitoFirmware) {
        let s = STATE.get().unwrap().lock();
        // Device info request handling
        let device_info_controller = ui.global::<DeviceInfoController>();
        if s.is_device_info_requested() {
            log_info!("Providing device info to UI");
            let info = firmware.vault.get_device_info().unwrap();
            log_info!("Device info: FW ver {}, BL ver {}, SN {}, FR count {}", 
                info.firmware_version, info.bootloader_version, info.serial_number, info.factory_reset_count);
            device_info_controller.set_firmware_version(slint::SharedString::from(&info.firmware_version[10..]));
            device_info_controller.set_bootloader_version(slint::SharedString::from(&info.bootloader_version));
            device_info_controller.set_serial_number(slint::SharedString::from(&info.serial_number));
            device_info_controller.set_factory_reset_count(info.factory_reset_count);
            device_info_controller.invoke_request_factory_reset_string();
            s.clear_device_info_requested();
        }
    }
}