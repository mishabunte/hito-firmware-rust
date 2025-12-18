//! Screen creation functions for the firmware UI
//! 
//! This module contains functions to create and configure each screen in the application.
//! Each function sets up the UI items, header, and callbacks for its respective screen.

extern crate alloc;
use alloc::vec;
use alloc::rc::Rc;

use slint::ModelRc;
use slint::VecModel;

use crate::slint_generatedMainWindow::{MainWindow, ScreenItem, ScreenButton};
use crate::log_info;
use crate::ui::router::{navigate_to, go_back};

use super::Screen;

/// Create the "Pair with App" screen
pub fn create_pair_with_app_screen(ui: &Rc<MainWindow>) {
    let button_x = 65.0;
    let button_y = 100.0;
    let button_w = 320.0 - button_x * 2.0;
    let button_h = 35.0;

    let items = ModelRc::new(VecModel::from(vec![
        ScreenItem { 
            text: "Pairing instructions here...".into(), 
            width: button_w, 
            height: button_h, 
            x: button_x, 
            y: 50.0 
        },
    ]));
    
    let buttons = ModelRc::new(VecModel::from(vec![
        ScreenButton { 
            text: "Back".into(),  
            width: button_w, 
            height: button_h, 
            has_border: true,
            x: button_x, 
            y: button_y + button_h * 2.0,
            inverted: false,
        },
    ]));
    ui.set_items(items);
    ui.set_buttons(buttons);
    ui.set_header_title(slint::SharedString::from("PAIR WITH APP"));
    
    ui.on_pressed(|item| {
        log_info!("PairWithApp: Pressed item: {}", item.text);
        
        if item.text == "Back" {
            log_info!("Back button pressed - navigating back to Menu");
            go_back();
        }
    });
}


