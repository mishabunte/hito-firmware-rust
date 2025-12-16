//! Screen creation functions for the firmware UI
//! 
//! This module contains functions to create and configure each screen in the application.
//! Each function sets up the UI items, header, and callbacks for its respective screen.

extern crate alloc;
use alloc::vec;
use alloc::rc::Rc;

use slint::ModelRc;
use slint::VecModel;

use crate::slint_generatedMainWindow::{MainWindow, ScreenItem};
use crate::log_info;
use super::router::{navigate_to, go_back, Screen};

/// Create the main menu screen with navigation options
pub fn create_menu_screen(ui: &Rc<MainWindow>) {
    let button_x = 65.0;
    let button_y = 35.0;
    let button_w = 320.0 - button_x * 2.0;
    let button_h = 35.0;

    let items = ModelRc::new(VecModel::from(vec![
        ScreenItem { 
            text: "Pair with the app".into(), 
            is_button: true, 
            width: button_w, 
            height: button_h, 
            x: button_x, 
            y: button_y 
        },
        ScreenItem { 
            text: "Show seed phrase".into(), 
            is_button: true, 
            width: button_w, 
            height: button_h, 
            x: button_x, 
            y: button_y + button_h 
        },
        ScreenItem { 
            text: "Device info".into(), 
            is_button: true, 
            width: button_w, 
            height: button_h, 
            x: button_x, 
            y: button_y + button_h * 2.0 
        },
        ScreenItem { 
            text: "Change passcode".into(), 
            is_button: true, 
            width: button_w, 
            height: button_h, 
            x: button_x, 
            y: button_y + button_h * 3.5 
        },
        ScreenItem { 
            text: "Factory reset".into(), 
            is_button: true, 
            width: button_w, 
            height: button_h, 
            x: button_x, 
            y: button_y + button_h * 4.5 
        },
    ]));
    
    ui.set_items(items);
    ui.set_header_title(slint::SharedString::from("MENU"));
    
    ui.on_pressed(|item| {
        log_info!("Menu: Pressed item: {} (is_button: {})", item.text, item.is_button);
        
        if item.is_button {
            match item.text.as_str() {
                "Factory reset" => {
                    log_info!("Factory Reset button pressed - navigating to FactoryReset screen");
                    navigate_to(Screen::FactoryReset);
                }
                "Pair with the app" => {
                    log_info!("Pair with the app button pressed - navigating to PairWithApp screen");
                    navigate_to(Screen::PairWithApp);
                }
                "Show seed phrase" => {
                    log_info!("Show seed phrase button pressed");
                    // TODO: Navigate to ShowSeedPhrase screen when implemented
                }
                "Device info" => {
                    log_info!("Device info button pressed");
                    // TODO: Navigate to DeviceInfo screen when implemented
                }
                "Change passcode" => {
                    log_info!("Change passcode button pressed");
                    // TODO: Navigate to ChangePasscode screen when implemented
                }
                _ => {}
            }
        }
    });
}

/// Create the "Pair with App" screen
pub fn create_pair_with_app_screen(ui: &Rc<MainWindow>) {
    let button_x = 65.0;
    let button_y = 100.0;
    let button_w = 320.0 - button_x * 2.0;
    let button_h = 35.0;

    let items = ModelRc::new(VecModel::from(vec![
        ScreenItem { 
            text: "Pairing instructions here...".into(), 
            is_button: false, 
            width: button_w, 
            height: button_h, 
            x: button_x, 
            y: 50.0 
        },
        ScreenItem { 
            text: "Back".into(), 
            is_button: true, 
            width: button_w, 
            height: button_h, 
            x: button_x, 
            y: button_y + button_h * 2.0 
        },
    ]));
    
    ui.set_items(items);
    ui.set_header_title(slint::SharedString::from("PAIR WITH APP"));
    
    ui.on_pressed(|item| {
        log_info!("PairWithApp: Pressed item: {} (is_button: {})", item.text, item.is_button);
        
        if item.is_button && item.text == "Back" {
            log_info!("Back button pressed - navigating back to Menu");
            go_back();
        }
    });
}

/// Create the "Factory Reset" confirmation screen
pub fn create_factory_reset_screen(ui: &Rc<MainWindow>) {
    let button_x = 65.0;
    let button_y = 80.0;
    let button_w = 320.0 - button_x * 2.0;
    let button_h = 35.0;

    let items = ModelRc::new(VecModel::from(vec![
        ScreenItem { 
            text: "Are you sure?".into(), 
            is_button: false, 
            width: button_w, 
            height: button_h, 
            x: button_x, 
            y: 50.0 
        },
        ScreenItem { 
            text: "This will erase all data!".into(), 
            is_button: false, 
            width: button_w, 
            height: 20.0, 
            x: button_x, 
            y: 75.0 
        },
        ScreenItem { 
            text: "Confirm Reset".into(), 
            is_button: true, 
            width: button_w, 
            height: button_h, 
            x: button_x, 
            y: button_y + button_h * 2.0 
        },
        ScreenItem { 
            text: "Cancel".into(), 
            is_button: true, 
            width: button_w, 
            height: button_h, 
            x: button_x, 
            y: button_y + button_h * 3.5 
        },
    ]));
    
    ui.set_items(items);
    ui.set_header_title(slint::SharedString::from("FACTORY RESET"));
    
    ui.on_pressed(|item| {
        log_info!("FactoryReset: Pressed item: {} (is_button: {})", item.text, item.is_button);
        
        if item.is_button {
            match item.text.as_str() {
                "Confirm Reset" => {
                    log_info!("Confirm Reset button pressed - perform factory reset");
                    // TODO: Perform actual factory reset logic
                    // After reset, navigate back to menu
                    go_back();
                }
                "Cancel" => {
                    log_info!("Cancel button pressed - navigating back to Menu");
                    go_back();
                }
                _ => {}
            }
        }
    });
}
