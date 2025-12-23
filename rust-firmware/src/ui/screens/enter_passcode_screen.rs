//! Screen creation functions for the firmware UI
//! 
//! This module contains functions to create and configure each screen in the application.
//! Each function sets up the UI items, header, and callbacks for its respective screen.

extern crate alloc;

use alloc::string::String;
use alloc::vec;
use alloc::rc::Rc;

use slint::{ModelRc, VecModel};

use crate::slint_generatedMainWindow::{MainWindow, ScreenItem, ScreenButton};
use crate::log_info;
use crate::ui::router::navigate_to;
use crate::ui::router::previous_screen;
use crate::firmware;

use super::Screen;

/// Static storage for pending passcode during change passcode flow
/// Used to pass passcode from set screen to confirm screen
static mut PENDING_PASSCODE: Option<String> = None;

/// Set the pending passcode (called when navigating to confirm screen)
pub fn set_pending_passcode(passcode: String) {
    unsafe {
        PENDING_PASSCODE = Some(passcode);
    }
}

/// Get and clear the pending passcode
fn take_pending_passcode() -> Option<String> {
    unsafe {
        PENDING_PASSCODE.take()
    }
}

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

const CHAR_BUTTON_X: f32 = 10.0;
const CHAR_BUTTON_Y: f32 = 115.0;
const CHAR_BUTTON_W: f32 = 60.0;
const CHAR_BUTTON_H: f32 = 60.0;
const COLS: usize = 5;
const PASSCODE_LENGTH: usize = 6;

fn show_keyboard(ui: &Rc<MainWindow>) {
    let arr = [b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9'];
    let shuffled = shuffle_digits(arr);

    let mut buttons: vec::Vec<ScreenButton> = shuffled
        .iter()
        .enumerate()
        .map(|(i, &digit)| {
            let row = i / COLS;
            let col = i % COLS;
            ScreenButton {
                text: (digit as char).into(),
                width: CHAR_BUTTON_W,
                height: CHAR_BUTTON_H,
                has_border: false,
                x: CHAR_BUTTON_X + CHAR_BUTTON_W * col as f32,
                y: CHAR_BUTTON_Y + CHAR_BUTTON_H * row as f32,
                inverted: true,
            }
        })
        .collect();

    // Add backspace button
    buttons.push(ScreenButton {
        text: "<".into(),
        width: CHAR_BUTTON_W,
        height: CHAR_BUTTON_H,
        has_border: false,
        x: CHAR_BUTTON_X + CHAR_BUTTON_W * 4.0 - CHAR_BUTTON_W / 3.0,
        y: CHAR_BUTTON_Y - CHAR_BUTTON_H,
        inverted: true,
    });

    ui.set_buttons(ModelRc::new(VecModel::from(buttons)));
    ui.set_back_shown(false);
}

/// Build passcode display string: "* * * * * *" (space-separated asterisks)
fn build_passcode_display(len: usize) -> String {
    if len == 0 {
        return String::new();
    }
    let mut s = String::with_capacity(len * 2);
    s.push('*');
    for _ in 1..len {
        s.push_str(" *");
    }
    s
}

fn display_passcode(ui: &Rc<MainWindow>, passcode_len: usize) {
    let pin_area_x = CHAR_BUTTON_X + CHAR_BUTTON_W + CHAR_BUTTON_W / 3.0;
    let pin_area_y = CHAR_BUTTON_Y - CHAR_BUTTON_H / 2.0 - 10.0;
    let pin_area_w = 320.0 - CHAR_BUTTON_X * 2.0 - CHAR_BUTTON_W * 2.0 - CHAR_BUTTON_W * 2.0 / 3.0;

    let passcode_displayed = build_passcode_display(passcode_len);
    
    // Approximate text width: each char ~12px
    let text_width = passcode_displayed.len() as f32 * 12.0;
    let centered_x = pin_area_x + (pin_area_w - text_width) / 2.0;

    ui.set_items(ModelRc::new(VecModel::from(vec![
        ScreenItem {
            text: passcode_displayed.into(),
            width: pin_area_w,
            height: 25.0,
            x: centered_x,
            y: pin_area_y,
        },
    ])));
}

/// Action to perform when passcode reaches required length
enum PasscodeAction {
    /// Unlock vault and navigate to success screen
    Unlock(Screen),
    /// Store passcode and navigate to confirmation screen
    SetNew,
    /// Confirm passcode matches expected value
    Confirm(String),
}

/// Generic passcode screen setup - reduces duplication across all passcode screens
fn setup_passcode_screen(ui: &Rc<MainWindow>, title: &str, action: PasscodeAction) {
    ui.set_header_title(slint::SharedString::from(title));
    show_keyboard(ui);

    let back_shown = if let Some(previous) = previous_screen() {
        match previous {
            Screen::Lock => false,
            _ => true,
        }
    } else {
        false
    };

    ui.set_back_shown(back_shown);

    let mut passcode_entered = String::new();
    let ui_weak = Rc::downgrade(ui);

    ui.on_pressed(move |item| {
        let Some(ui) = ui_weak.upgrade() else { return };

        // Handle digit input
        if item.text.parse::<u8>().is_ok() {
            passcode_entered.push_str(&item.text);

            if passcode_entered.len() >= PASSCODE_LENGTH {
                match &action {
                    PasscodeAction::Unlock(success_screen) => {
                        let mut vault = firmware().vault.lock();
                        match vault.unlock_with_password(passcode_entered.as_bytes()) {
                            Ok(_) => {
                                log_info!("Unlock successful!");
                                navigate_to(success_screen.clone());
                            }
                            Err(e) => {
                                log_info!("Unlock failed: {:?}", e);
                                passcode_entered.clear();
                            }
                        }
                    }
                    PasscodeAction::SetNew => {
                        set_pending_passcode(passcode_entered.clone());
                        navigate_to(Screen::ChangePasscodeConfirm);
                    }
                    PasscodeAction::Confirm(expected) => {
                        if &passcode_entered == expected {
                            log_info!("Set passcode successful!");
                            let mut vault = firmware().vault.lock();
                            if vault.is_empty() {
                              navigate_to(Screen::WalletSetup);
                            } else {
                              let _ = vault.set_passcode(passcode_entered.as_bytes());
                              navigate_to(Screen::ChangePasscodeSet);
                            }
                        } else {
                            log_info!("Set passcode failed: confirmation does not match");
                            passcode_entered.clear();
                        }
                    }
                }
            }
        }

        // Handle backspace
        if item.text == "<" {
            passcode_entered.pop();
        }

        display_passcode(&ui, passcode_entered.len());
    });
}

pub fn create_confirm_passcode_screen(ui: &Rc<MainWindow>) {
    let expected = take_pending_passcode().unwrap_or_default();
    setup_passcode_screen(ui, "Confirm Passcode", PasscodeAction::Confirm(expected));
}

pub fn create_set_passcode_screen(ui: &Rc<MainWindow>) {
    setup_passcode_screen(ui, "Set Passcode", PasscodeAction::SetNew);
}

/// Create the "Enter Passcode" screen
pub fn create_enter_passcode_screen(ui: &Rc<MainWindow>, success_screen: Screen) {
    let vault_arc = firmware().vault.clone();
    let vault = vault_arc.lock();
    if vault.is_empty() {
      create_set_passcode_screen(ui);
      return;
    }
    setup_passcode_screen(ui, "Enter Passcode", PasscodeAction::Unlock(success_screen));
}
