use crate::slint_generatedMainWindow::MainWindow;

use crate::hito_firmware::HitoFirmware;

pub trait CallbackController: Sync {
  fn register_main_window_callbacks(&self, ui: &MainWindow, firmware: &mut HitoFirmware);
  fn handle_loop_events(&self, ui: &MainWindow, firmware: &mut HitoFirmware);
}

mod main_callback_controller;
pub use main_callback_controller::MainCallbackController;

mod enter_pin_callback_controller;
pub use enter_pin_callback_controller::EnterPinCallbackController;

mod device_info_callback_controller;
pub use device_info_callback_controller::DeviceInfoCallbackController;

mod receive_data_callback_controller;
pub use receive_data_callback_controller::ReceiveDataCallbackController;

mod send_screen_callback_controller;
pub use send_screen_callback_controller::SendScreenCallbackController;

mod signed_data_callback_controller;
pub use signed_data_callback_controller::SignedDataCallbackController;

pub static UI_CALLBACK_CONTROLLERS: &[&dyn CallbackController] = &[
    &MainCallbackController,
    &EnterPinCallbackController,
    &DeviceInfoCallbackController,
    &ReceiveDataCallbackController,
    &SendScreenCallbackController,
    &SignedDataCallbackController,
];


