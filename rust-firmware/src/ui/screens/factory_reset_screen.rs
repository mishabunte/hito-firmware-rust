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

/// Create the "Factory Reset" confirmation screen
pub fn create_factory_reset_screen(ui: &Rc<MainWindow>) {
    let button_x = 65.0;
    let button_y = 80.0;
    let button_w = 320.0 - button_x * 2.0;
    let button_h = 35.0;

    let items = ModelRc::new(VecModel::from(vec![
        ScreenItem { 
            text: "Are you sure?".into(), 
            width: button_w, 
            height: button_h, 
            x: button_x, 
            y: 50.0 
        },
        ScreenItem { 
            text: "This will erase all data!".into(), 
            width: button_w, 
            height: 20.0, 
            x: button_x, 
            y: 75.0 
        },
    ]));

    let buttons = ModelRc::new(VecModel::from(vec![
        ScreenButton { 
            text: "Confirm Reset".into(),  
            width: button_w, 
            height: button_h,
            has_border: false, 
            x: button_x, 
            y: button_y + button_h * 2.0,
            inverted: false,
        },
        ScreenButton { 
            text: "Cancel".into(), 
            width: button_w, 
            height: button_h, 
            has_border: true, 
            x: button_x, 
            y: button_y + button_h * 3.5,
            inverted: false,
        },
    ]));
    
    ui.set_items(items);
    ui.set_buttons(buttons);
    ui.set_header_title(slint::SharedString::from("FACTORY RESET"));
    
    ui.on_pressed(|item| {
        log_info!("FactoryReset: Pressed item: {}", item.text);
        
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
    });
}
