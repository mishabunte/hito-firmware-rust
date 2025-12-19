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
pub fn create_home_screen(ui: &Rc<MainWindow>) {

    let buttons = ModelRc::new(VecModel::from(vec![
        ScreenButton {
            text: "RECEIVE".into(),  
            width: 130.0, 
            height: 40.0,
            has_border: true, 
            x: 20.0, 
            y: 180.0,
            inverted: true,
        },
        ScreenButton { 
            text: "SEND".into(),  
            width: 130.0, 
            height: 40.0,
            has_border: true, 
            x: 170.0, 
            y: 180.0,
            inverted: true,
        },
    ]));

    ui.set_setting_shown(true);
    ui.set_back_shown(false);

    ui.set_buttons(buttons);
    
    ui.on_pressed(move |item| {
        if item.text == "RECEIVE" {
            log_info!("Navigate to RECEIVE screen");
            navigate_to(Screen::Receive);
        } else if item.text == "SEND" {
            log_info!("Navigate to SEND screen");
            navigate_to(Screen::Send);
        } else if item.text == "SETTINGS" {
            log_info!("Navigate to SETTINGS screen");
            navigate_to(Screen::Menu);
        }
    });
}
