//! Screen creation functions for the firmware UI
//! 
//! This module contains functions to create and configure each screen in the application.
//! Each function sets up the UI items, header, and callbacks for its respective screen.

extern crate alloc;
use alloc::vec;
use alloc::rc::Rc;

use slint::ModelRc;
use slint::VecModel;

use crate::ScreenItem;
use crate::slint_generatedMainWindow::{MainWindow, ScreenButton};
use crate::log_info;
use crate::ui::router::{navigate_to, go_back};

use super::Screen;

/// Create the main menu screen with navigation options
pub fn create_send_stellar_screen(ui: &Rc<MainWindow>) {
    ui.set_header_title(slint::SharedString::from("MENU"));

    let button_x = 65.0;
    let button_y = 42.0;
    let button_w = 320.0 - button_x * 2.0;
    let button_h = 35.0;

    let items = ModelRc::new(VecModel::from(vec![
      ScreenItem { 
          text: "This is a Stellar screen".into(), 
          width: button_w, 
          height: button_h, 
          x: button_x, 
          y: button_y,
      },
    ]));
    
    ui.set_items(items);
}