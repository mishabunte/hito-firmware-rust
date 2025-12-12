pub struct EnterPinCallbackController;

impl CallbackController for EnterPinCallbackController {
  fn register_main_window_callbacks(&self, ui: &MainWindow, firmware: &mut HitoFirmware) {
      ui.global::<ChangePasscodeState>().on_append_char(move |digit| {
          log_info!("Append char to PIN: {}", digit);
      });
  }
}