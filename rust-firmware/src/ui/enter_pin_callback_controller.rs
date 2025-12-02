use crate::{STATE, ui::CallbackController, hito_firmware::HitoFirmware};
use crate::slint_generatedMainWindow::EnterPinState;
use crate::slint_generatedMainWindow::MainWindow;
use slint::{ComponentHandle, ToSharedString};
use crate::{log_info, ui};


#[cfg(feature = "zephyr")]
pub fn shuffle_digits(mut arr: [u8; 10]) -> [u8; 10] {
    use crate::crypto::ffi::hito_sys_rand32_get;
    fn rand10() -> usize {
        unsafe { (hito_sys_rand32_get() % 10) as usize }
    }
    for _ in 0..128 {
        let i = rand10();
        let j = rand10();

        if i != j {
            arr.swap(i, j);
        }
    }
    arr
}

#[cfg(feature = "minifb")]
pub fn shuffle_digits(mut arr: [u8; 10]) -> [u8; 10] {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    for _ in 0..128 {
        let i = rng.gen_range(0..10);
        let j = rng.gen_range(0..10);

        if i != j {
            arr.swap(i, j);
        }
    }

    arr
}



pub struct EnterPinCallbackController;

impl CallbackController for EnterPinCallbackController {
    fn register_main_window_callbacks(&self, ui: &MainWindow, firmware: &mut HitoFirmware) {
        // PIN input mechanics
        // let arr = [b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9'];
        // let shuffled = shuffle_digits(arr);
        // let shuffled_str = alloc::string::String::from_utf8_lossy(&shuffled).to_shared_string();
        // ui.global::<EnterPinState>().set_password_sequence(shuffled_str);
        ui.global::<EnterPinState>().on_append_char(move |digit| {
            log_info!("Append char to PIN: {}", digit);
            let s = STATE.get().unwrap().lock();
            let int_digit = digit.as_bytes()[0] - b'0';
            s.append_to_pin(int_digit);
            // log_info!("PIN code updated: {}", s.get_pin());
        });

        let ui_weak = ui.as_weak();

        ui.global::<EnterPinState>().on_shuffle_password_sequence(move || {
            let ui = ui_weak.upgrade();
            if let Some(ui) = ui {
                let arr = [b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9'];
                let shuffled = shuffle_digits(arr);
                let shuffled_str = alloc::string::String::from_utf8_lossy(&shuffled).to_shared_string();
                ui.global::<EnterPinState>().set_password_sequence(shuffled_str);
            }
        });

        let ui_weak = ui.as_weak();

        ui.global::<EnterPinState>().on_get_char(move |index| {
            let ui = ui_weak.upgrade();
            if let Some(ui) = ui {
                let enter_pin = ui.global::<EnterPinState>();
                let password_sequence = enter_pin.get_password_sequence();
                let word = password_sequence[index as usize..(index as usize + 1)].to_ascii_lowercase();
                //log_info!("Get char at index {}: {}", index, word);
                slint::SharedString::from(word)
            } else {
                // UI was destroyed
                slint::SharedString::new()
            }
        });

        // Remove last char
        ui.global::<EnterPinState>().on_remove_char(move || {
            let s = STATE.get().unwrap().lock();
            s.remove_pin_char();
            // log_info!("PIN code updated: {}", s.get_pin());
        });

        // Mark password is entered
        ui.global::<EnterPinState>().on_passcode_entered(move || {
            let s = STATE.get().unwrap().lock();
            s.mark_unlock_requested();
            // log_info!("Passcode entered, requesting unlock");
        });
    }
    fn handle_loop_events(&self, ui: &MainWindow, firmware: &mut HitoFirmware) {
        let s = STATE.get().unwrap().lock();
        let pin_controller = ui.global::<EnterPinState>();
        // When the UI marks unlock requested:
        if s.is_unlock_in_progress() {
            // Start job once
            // log_info!("Starting unlock job");
            if firmware.vault.unlock_job_is_none() {
                // log_info!("Job is none, starting unlock");
                let password = s.get_pin();
                if let Err(e) = firmware.vault.start_unlock(password.as_bytes()) {
                    // log_info!("Failed to start unlock: {:?}", e);
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
                    ui.global::<EnterPinState>().invoke_set_progress(p as i32);
                }
                Ok(None) => {
                    // nothing changed this tick
                }
                Err(e) => {
                    // log_info!("Vault unlock failed: {:?}", e);
                    // Mark wrong passcode in UI
                    pin_controller.set_wrong_passcode(true);
                    pin_controller.invoke_set_progress(0);
                    s.unlock_finished();
                    ui.global::<EnterPinState>().invoke_unlock(false);
                }
            }
            // If finished successfully, mark UI
            if firmware.vault.is_unlocked() {
                // log_info!("Unlock successful");
                pin_controller.set_wrong_passcode(false);
                pin_controller.invoke_set_progress(-1);
                s.mark_device_info_requested();
                s.unlock_finished();
                ui.global::<EnterPinState>().invoke_unlock(true);
                firmware.vault.reset_unlock_job();
            }
        }
    }
}