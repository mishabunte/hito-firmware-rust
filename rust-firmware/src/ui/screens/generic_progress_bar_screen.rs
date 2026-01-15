//! Screen creation functions for the firmware UI
//! 
//! This module contains functions to create and configure each screen in the application.
//! Each function sets up the UI items, header, and callbacks for its respective screen.

extern crate alloc;
use alloc::vec::Vec;
use alloc::vec;
use alloc::rc::Rc;
use alloc::string::ToString;
use alloc::format;

use slint::ModelRc;
use slint::VecModel;

use crate::common::ui_set_progress_bar_properties;
use crate::firmware;
use crate::slint_generatedMainWindow::{MainWindow, ScreenItem, ScreenButton, ScreenImage};
use crate::log_info;
use crate::state;
use crate::ui::router::{navigate_to, go_back};
use crate::ui::screens::show_alert;
use crate::ui_report_progress;

use super::Screen;

pub fn create_generic_progress_bar_screen(ui: &Rc<MainWindow>, header_title: &str, message: &str, initialize: bool) {
    let button_x = 0.0;
    let button_y = 62.0;
    let button_gap = 28.0;
    
    ui.set_center_text(true);
    // parse message into lines if too long, \\ is line break
    let message_lines: Vec<&str> = message.split("\\\\").collect();

    let lines = message_lines.len();
    log_info!("GenericProgressBar: Message has {} lines", lines);
    for (i, line) in message_lines.iter().enumerate() {
        log_info!("Message line {}: {}", i, line);
        if line.len() > 25 {
            log_info!("Warning: line {} is too long ({} characters)", i, line.len());
            // fail here
            panic!("Line {} is too long ({} characters)", i, line.len());
        }
    }

    // Create screen items based on number of lines
    let mut items_vec = vec![];
    for (i, line) in message_lines.iter().enumerate() {
        items_vec.push(ScreenItem { 
            text: (*line).into(), 
            width: 320.0,
            height: 25.0,
            x: button_x, 
            // add a 10.0 gap between 2. and 3. line
            y: if i > 1 { button_y + (i as f32) * button_gap + 10.0 } else { button_y + (i as f32) * button_gap },
        });
    }
    
    let items = ModelRc::new(VecModel::from(items_vec));
    
    ui.set_items(items);
    ui.set_header_title(slint::SharedString::from(header_title));

    let pin = state().lock().get_pin();

    let buttons = ModelRc::new(VecModel::from(vec![
        ScreenButton {
            text: "ENCRYPT".into(),  
            has_border: true, 
            x: 84.0, 
            y: 158.0,
            width: 152.0, 
            height: 25.0,
            inverted: true,
        },
    ]));
    
    ui.set_buttons(buttons);
    ui_set_progress_bar_properties(84, 158, 152, 25, 1);

    ui.on_pressed(move |item| {
        if item.text == "ENCRYPT" {
          match firmware().vault.lock().set_passcode(pin.as_bytes(), Some(ui_report_progress)) {
              Ok(()) => {
                navigate_to(Screen::FinalizePasscodeChange);
              },
              Err(e) => {
                show_alert(format!("Failed to set passcode: {}", e).as_str());
              }
          }
        }
    });
}
