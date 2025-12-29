//! Screen creation functions for the firmware UI
//! 
//! This module contains functions to create and configure each screen in the application.
//! Each function sets up the UI items, header, and callbacks for its respective screen.

extern crate alloc;
use alloc::rc::Rc;

use crate::slint_generatedMainWindow::MainWindow;
use crate::log_info;
use crate::ui::router::{navigate_to, go_back};

use crate::drivers::Battery;

use super::Screen;
use crate::ui::screens::generic_question_screen::create_generic_question_screen;

use crate::firmware;

/// Create the "Factory Reset" confirmation screen
pub fn create_factory_reset_screen(ui: &Rc<MainWindow>) {
    create_generic_question_screen(
        ui,
        "FACTORY RESET",
        "You are about to erase \\\\ all data on the device. \\\\ Are you sure you \\\\ want to proceed?",
        "Continue",
        Some("Cancel"),
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
        "FACTORY RESET",
        "Enter your passcode to \\\\ confirm factory reset.",
        "Confirm",
        Some("Cancel"),
        || {
          // Perform factory reset
          log_info!("Factory Reset confirmed - performing factory reset");
          if firmware().vault.lock().erase(true, true) {
            log_info!("Factory Reset: Vault erased successfully");
            firmware().battery.lock().reboot();
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
