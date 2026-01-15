//! Screen creation functions for the firmware UI
//! 
//! This module contains functions to create and configure each screen in the application.
//! Each function sets up the UI items, header, and callbacks for its respective screen.

extern crate alloc;
use alloc::vec;
use alloc::rc::Rc;

use slint::ModelRc;
use slint::VecModel;
use alloc::format;

use crate::ScreenItem;
use crate::common::ui_draw_qr;
use crate::drivers::Display;
use crate::drivers::QrCodeWrapper;
use crate::firmware;
use crate::slint_generatedMainWindow::{MainWindow, ScreenButton};
use crate::log_info;
use crate::ui::router::{navigate_to, go_back};
use crate::ui::screens::show_alert;
use alloc::string::String;
use slint::ToSharedString;

use crate::common::QR_CODE;

use super::Screen;

fn shorten_address(address: &str) -> alloc::string::String {
    if address.len() <= 17 {
        return alloc::string::String::from(address);
    }
    let first_quarter = &address[0..4];
    let second_quarter = &address[4..8];
    let first = first_quarter.to_shared_string() + " " + second_quarter;

    let last_first_quarter = &address[address.len()-8..address.len()-4];
    let last_quarter = &address[address.len()-4..];
    let last = last_first_quarter.to_shared_string() + " " + last_quarter;
    alloc::format!("{}..{}", first, last)
}

/// Create the main menu screen with navigation options
pub fn create_receive_screen(ui: &Rc<MainWindow>) {
    ui.set_header_title(slint::SharedString::from("MENU"));

    let address_x = 40.0;
    let address_y = 200.0;
    let address_w = 320.0;
    let address_h = 35.0;

    if let Ok(address) = firmware().vault.lock().get_stellar_address() {
        let address_short = shorten_address(&address);

        let items = ModelRc::new(VecModel::from(vec![
          ScreenItem { 
              text: address_short.into(), 
              width: address_w, 
              height: address_h, 
              x: address_x, 
              y: address_y,
          },
        ]));

        ui.set_header_title(slint::SharedString::from("YOUR ADDRESS"));
        
        ui.set_items(items);

        let qr_data = format!("stellar:{}", address);
        ui_draw_qr(&qr_data);
    } else if let Err(e) = firmware().vault.lock().get_stellar_address() {
        show_alert(format!("Failed to get address: {}", e).as_str());
        return;
    }
}