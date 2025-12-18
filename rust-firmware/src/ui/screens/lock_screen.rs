//! Screen creation functions for the firmware UI
//! 
//! This module contains functions to create and configure each screen in the application.
//! Each function sets up the UI items, header, and callbacks for its respective screen.

extern crate alloc;
use alloc::string::String;

use alloc::vec;
use alloc::rc::Rc;

use slint::ModelRc;
use slint::VecModel;

use crate::slint_generatedMainWindow::{MainWindow, ScreenItem, ScreenButton, ScreenImage};
use crate::log_info;
use crate::ui::router::{navigate_to, go_back};

use super::Screen;

/// Create the "Enter Passcode" screen
pub fn create_lock_screen(ui: &Rc<MainWindow>) {
    ui.set_back_shown(false);
    ui.set_is_lockscreen(true);
}
