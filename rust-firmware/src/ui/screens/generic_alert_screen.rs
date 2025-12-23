//! Screen creation functions for the firmware UI
//! 
//! This module contains functions to create and configure each screen in the application.
//! Each function sets up the UI items, header, and callbacks for its respective screen.

extern crate alloc;
use alloc::vec::Vec;
use alloc::vec;
use alloc::rc::Rc;
use alloc::string::ToString;

use slint::ModelRc;
use slint::VecModel;

use crate::slint_generatedMainWindow::{MainWindow, ScreenItem, ScreenButton, ScreenImage};
use crate::log_info;
use crate::ui::router::{navigate_to, go_back};

use super::Screen;

pub fn show_alert(ui: &Rc<MainWindow>, alert: &str) {
    let button_x = 0.0;
    let button_y = 62.0;
    let button_gap = 28.0;
    let button_w = 130.0;
    let button_h = 40.0;

    let cancel_x = 20.0;
    let cancel_y = 190.0;
    let confirm_x = 170.0;
    let confirm_y = 190.0;
    
    ui.set_center_text(true);
    // parse question into lines if too long, \\ is line break
    let alert_lines: Vec<&str> = alert.split("\\\\").collect();

    let lines = alert_lines.len();
    log_info!("GenericQuestion: Question has {} lines", lines);

    for (i, line) in alert_lines.iter().enumerate() {
        log_info!("Question line {}: {}", i, line);
        if line.len() > 25 {
            log_info!("Warning: line {} is too long ({} characters)", i, line.len());
            // fail here
            panic!("Line {} is too long ({} characters)", i, line.len());
        }
    }

    // Create screen items based on number of lines
    let mut items_vec = vec![];
    for (i, line) in alert_lines.iter().enumerate() {
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
    ui.set_header_title(slint::SharedString::from("ALERT"));
}
