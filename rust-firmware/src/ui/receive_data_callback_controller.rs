use crate::drivers::{Display};
use crate::{STATE, ui::CallbackController, hito_firmware::HitoFirmware};
use crate::slint_generatedMainWindow::ReceiveDataState;
use crate::slint_generatedMainWindow::BrightnessController;
use crate::slint_generatedMainWindow::MainWindow;
use slint::{ComponentHandle, ToSharedString};
use crate::slint_generatedMainWindow::Router;
use crate::slint_generatedMainWindow::ScreenEnum;
use crate::log_info;

fn shorten_address(address: &str) -> alloc::string::String {
    if address.len() <= 17 {
        return alloc::string::String::from(address);
    }
    let first_quarter = &address[0..4];
    let second_quarter = &address[4..8];
    let first = first_quarter.to_shared_string() + " " + second_quarter;

    let last_first_quarter = &address[address.len()-8..address.len()-4];
    let last_quarter = &address[address.len()-4..];
    let last = last_first_quarter.to_shared_string() + " " + last_quarter;
    alloc::format!("{}..{}", first, last)
}

pub struct ReceiveDataCallbackController;

impl CallbackController for ReceiveDataCallbackController {
    fn register_main_window_callbacks(&self, ui: &MainWindow, firmware: &mut HitoFirmware) {
      ui.global::<ReceiveDataState>().on_request_receive_data(move || {
          let s = STATE.get().unwrap().lock();
          s.mark_qr_data_requested();
          // log_info!("Receive data requested");
      });
      ui.global::<BrightnessController>().on_scale_shown_changed(move || {
          let s = STATE.get().unwrap().lock();
          s.mark_scale_shown_changed();
      });
    }
    fn handle_loop_events(&self, ui: &MainWindow, firmware: &mut HitoFirmware) {
      let s = STATE.get().unwrap().lock();
      let receive_data_state = ui.global::<ReceiveDataState>();

      if s.is_qr_data_requested() || s.is_scale_shown_changed() {
        if ui.global::<Router>().get_current() == ScreenEnum::Receive {
          let address = firmware.vault.get_stellar_address().unwrap();
          receive_data_state.set_address_short(slint::SharedString::from(shorten_address(&address)));
          firmware.display.draw_qr(25, 50, &address);
          s.mark_qr_data_success();
          s.clear_scale_shown_changed();
        }
      }
    }
}