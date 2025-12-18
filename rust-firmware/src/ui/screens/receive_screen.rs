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
use crate::drivers::Display;
use crate::drivers::QrCodeWrapper;
use crate::firmware;
use crate::slint_generatedMainWindow::{MainWindow, ScreenButton};
use crate::log_info;
use crate::ui::router::{navigate_to, go_back};
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

    let address_x = 65.0;
    let address_y = 200.0;
    let address_w = 320.0 - address_x * 2.0;
    let address_h = 35.0;

    let address = firmware().vault.lock().get_stellar_address().unwrap_or(String::from("Error retrieving address"));
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

    unsafe {
      let qr_data = format!("stellar:{}", address);
      QR_CODE = Some(QrCodeWrapper::new());
      QR_CODE.as_mut().unwrap().set_data(&qr_data);
      QR_CODE.as_mut().unwrap().set_coords(75, 35);
    }
}