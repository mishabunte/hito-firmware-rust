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

pub fn create_generic_question_screen(ui: &Rc<MainWindow>, question: &str, confirm_text: &str, cancel_text: &str, on_confirm: impl Fn() + 'static, on_cancel: impl Fn() + 'static) {
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
    let question_lines: Vec<&str> = question.split("\\\\").collect();

    let lines = question_lines.len();
    log_info!("GenericQuestion: Question has {} lines", lines);

    for (i, line) in question_lines.iter().enumerate() {
        log_info!("Question line {}: {}", i, line);
        if line.len() > 25 {
            log_info!("Warning: line {} is too long ({} characters)", i, line.len());
            // fail here
            panic!("Line {} is too long ({} characters)", i, line.len());
        }
    }

    // Create screen items based on number of lines
    let mut items_vec = vec![];
    for (i, line) in question_lines.iter().enumerate() {
        items_vec.push(ScreenItem { 
            text: (*line).into(), 
            width: 320.0,
            height: 20.0, 
            x: button_x, 
            // add a 10.0 gap between 2. and 3. line
            y: if i > 1 { button_y + (i as f32) * button_gap + 10.0 } else { button_y + (i as f32) * button_gap },
        });
    }
    
    let items = ModelRc::new(VecModel::from(items_vec));

    let buttons = ModelRc::new(VecModel::from(vec![
        ScreenButton { 
            text: confirm_text.into(),  
            width: button_w, 
            height: button_h,
            has_border: false, 
            x: confirm_x, 
            y: confirm_y,
            inverted: false,
        },
        ScreenButton { 
            text: cancel_text.into(), 
            width: button_w, 
            height: button_h, 
            has_border: false, 
            x: cancel_x, 
            y: cancel_y,
            inverted: false,
        },
    ]));
    
    ui.set_items(items);
    ui.set_buttons(buttons);
    ui.set_header_title(slint::SharedString::from("FACTORY RESET"));
    
    let confirm_text = confirm_text.to_string();
    let cancel_text = cancel_text.to_string();
    
    ui.on_pressed(move |item| {
        log_info!("GenericQuestion: Pressed item: {}", item.text);
        
        let text = item.text.as_str();
        if text == confirm_text {
            log_info!("Confirm button pressed");
            on_confirm();
        } else if text == cancel_text {
            log_info!("Cancel button pressed");
            on_cancel();
        }
    });
}
