//! Screen creation functions for the firmware UI
//! 
//! This module contains functions to create and configure each screen in the application.
//! Each function sets up the UI items, header, and callbacks for its respective screen.

extern crate alloc;
use alloc::vec;
use alloc::rc::Rc;

use slint::ModelRc;
use slint::VecModel;

use crate::slint_generatedMainWindow::{MainWindow, ScreenItem, ScreenButton, ScreenImage};
use crate::log_info;
use crate::ui::router::{navigate_to, go_back};

use super::Screen;
use crate::ui::screens::generic_question_screen::create_generic_question_screen;
use crate::ui::screens::enter_passcode_screen::create_enter_passcode_screen;

use crate::firmware;
use crate::vault;

/// Create the "Factory Reset" confirmation screen
pub fn create_factory_reset_screen(ui: &Rc<MainWindow>) {
    create_generic_question_screen(
        ui,
        "You are about to erase \\\\ all data on the device. \\\\ Are you sure you \\\\ want to proceed?",
        "Continue",
        "Cancel",
        || {
          navigate_to(Screen::FactoryResetPasscode)
        },
        || {
          go_back()
        }
    );
}

pub fn create_erase_screen(ui: &Rc<MainWindow>) {
    create_generic_question_screen(
        ui,
        "Enter your passcode to \\\\ confirm factory reset.",
        "Confirm",
        "Cancel",
        || {
          // Perform factory reset
          log_info!("Factory Reset confirmed - performing factory reset");
          if firmware().vault.lock().erase(true, true) {
            log_info!("Factory Reset: Vault erased successfully");
            // TODO: Restart device
          } else {
            log_info!("Factory Reset: Vault erase failed");
            // Navigate back to menu
            navigate_to(Screen::Menu);
          }
        },
        || {
          navigate_to(Screen::Menu);
        }
    );
}
