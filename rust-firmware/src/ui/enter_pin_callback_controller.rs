use crate::drivers::Display;
use crate::{STATE, ui::CallbackController, hito_firmware::HitoFirmware};
use crate::slint_generatedMainWindow::EnterPinController;
use crate::slint_generatedMainWindow::MainWindow;
use slint::ComponentHandle;
use crate::{firmware_state, log_info};
use alloc::rc::Rc;

pub struct EnterPinCallbackController;

impl CallbackController for EnterPinCallbackController {
    fn register_main_window_callbacks(&self, ui: &MainWindow, firmware: &mut HitoFirmware) {
        // PIN input mechanics
        ui.global::<EnterPinController>().on_append_char(move |digit: i32| {
            let s = STATE.get().unwrap().lock();
            s.append_to_pin(digit);
            log_info!("PIN code updated: {}", s.get_pin());
        });

        // Remove last char
        ui.global::<EnterPinController>().on_remove_char(move || {
            let s = STATE.get().unwrap().lock();
            s.remove_pin_char();
            log_info!("PIN code updated: {}", s.get_pin());
        });

        // Mark password is entered
        ui.global::<EnterPinController>().on_passcode_entered(move || {
            let s = STATE.get().unwrap().lock();
            s.mark_unlock_requested();
            log_info!("Passcode entered, requesting unlock");
        });
    }
    fn handle_loop_events(&self, ui: &MainWindow, firmware: &mut HitoFirmware) {
        let s = STATE.get().unwrap().lock();
        let pin_controller = ui.global::<EnterPinController>();
        // When the UI marks unlock requested:
        if s.is_unlock_in_progress() {
            // Start job once
            log_info!("Starting unlock job");
            if firmware.vault.unlock_job_is_none() {
                log_info!("Job is none, starting unlock");
                let password = s.get_pin();
                if let Err(e) = firmware.vault.start_unlock(password.as_bytes()) {
                    log_info!("Failed to start unlock: {:?}", e);
                    pin_controller.set_wrong_passcode(true);
                    pin_controller.invoke_set_progress(-1);
                    s.unlock_finished();
                } else {
                    pin_controller.invoke_set_progress(0);
                }
            }
            // Drive one small chunk per frame
            match firmware.vault.poll_unlock() {
                Ok(Some(p)) => {
                    // You can update Slint progress here too, or rely on vault.set_progress callback
                    ui.global::<EnterPinController>().invoke_set_progress(p as i32);
                }
                Ok(None) => {
                    // nothing changed this tick
                }
                Err(e) => {
                    log_info!("Vault unlock failed: {:?}", e);
                    // Mark wrong passcode in UI
                    pin_controller.set_wrong_passcode(true);
                    pin_controller.invoke_set_progress(0);
                    s.unlock_finished();
                    ui.global::<EnterPinController>().invoke_unlock(false);
                }
            }
            // If finished successfully, mark UI
            if firmware.vault.is_unlocked() {
                log_info!("Unlock successful");
                pin_controller.set_wrong_passcode(false);
                pin_controller.invoke_set_progress(-1);
                s.mark_device_info_requested();
                s.unlock_finished();
                ui.global::<EnterPinController>().invoke_unlock(true);
                firmware.vault.reset_unlock_job();
            }
        }
    }
}