//! Screen creation functions for the firmware UI
//! 
//! This module contains functions to create and configure each screen in the application.
//! Each function sets up the UI items, header, and callbacks for its respective screen.

extern crate alloc;
use alloc::vec;
use alloc::rc::Rc;

use slint::ModelRc;
use slint::VecModel;

use crate::slint_generatedMainWindow::{MainWindow, ScreenButton};
use crate::log_info;
use crate::ui::router::{navigate_to, go_back};

use super::Screen;

/// Create the main menu screen with navigation options
pub fn create_menu_screen(ui: &Rc<MainWindow>) {
    ui.set_header_title(slint::SharedString::from("MENU"));

    let button_x = 65.0;
    let button_y = 42.0;
    let button_w = 320.0 - button_x * 2.0;
    let button_h = 35.0;

    let items = ModelRc::new(VecModel::from(vec![
        ScreenButton { 
            text: "Pair with the app".into(), 
            has_border: false,
            width: button_w, 
            height: button_h, 
            x: button_x, 
            y: button_y,
            inverted: false,
        },
        ScreenButton { 
            text: "Show seed phrase".into(), 
            has_border: false,
            width: button_w, 
            height: button_h, 
            x: button_x, 
            y: button_y + button_h,
            inverted: false,
        },
        ScreenButton { 
            text: "Device info".into(), 
            has_border: false,
            width: button_w, 
            height: button_h, 
            x: button_x, 
            y: button_y + button_h * 2.0,
            inverted: false,
        },
        ScreenButton { 
            text: "Change passcode".into(), 
            has_border: false,
            width: button_w, 
            height: button_h, 
            x: button_x, 
            y: button_y + button_h * 3.0,
            inverted: false, 
        },
        ScreenButton { 
            text: "Factory reset".into(), 
            has_border: false,
            width: button_w, 
            height: button_h, 
            x: button_x, 
            y: button_y + button_h * 4.3,
            inverted: false,
        },
    ]));
    
    ui.set_buttons(items);
    
    ui.on_pressed(|item| {
        log_info!("Menu: Pressed item: {}", item.text);
        
        match item.text.as_str() {
            "Factory reset" => {
                log_info!("Factory Reset button pressed - navigating to FactoryReset screen");
                navigate_to(Screen::FactoryReset);
            }
            "Pair with the app" => {
                log_info!("Pair with the app button pressed - navigating to PairWithApp screen");
                navigate_to(Screen::Receive);
            }
            "Show seed phrase" => {
                log_info!("Show seed phrase button pressed");
                navigate_to(Screen::ShowSeed);
            }
            "Device info" => {
                log_info!("Device info button pressed");
                // TODO: Navigate to DeviceInfo screen when implemented
            }
            "Change passcode" => {
                log_info!("Change passcode button pressed");
                navigate_to(Screen::ChangePasscodePasscode);
            }
            _ => {}
        }
    });
}