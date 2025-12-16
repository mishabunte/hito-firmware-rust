use crate::slint_generatedMainWindow::MainWindow;

use crate::hito_firmware::HitoFirmware;

// pub trait CallbackController: Sync {
//   fn register_main_window_callbacks(&self, ui: &MainWindow, firmware: &mut HitoFirmware);
//   fn handle_loop_events(&self, ui: &MainWindow, firmware: &mut HitoFirmware);
// }

// mod main_window_wrapper;
// pub use main_window_wrapper::MainWindowWrapper;

// pub static UI_CALLBACK_CONTROLLERS: &[&dyn CallbackController] = &[
//     &MainWindowWrapper,
// ];

pub mod router;
pub mod screens;

pub use router::{Router, Screen, init_global_router, navigate_to, go_back, process_pending_navigation};


