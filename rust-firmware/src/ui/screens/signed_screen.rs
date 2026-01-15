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
use crate::crypto::libcrypt0pro::stellar::StellarTransactionParser;
use crate::crypto::libcrypt0pro::stellar::StellarTransactionSerializer;
use crate::crypto::libcrypt0pro::stellar::StellarTransactionSigner;
use crate::crypto::libcrypt0pro::stellar::TransactionEnvelope;
use crate::drivers::Display;
use crate::drivers::QrCodeWrapper;
use crate::firmware;
use crate::slint_generatedMainWindow::{MainWindow, ScreenButton};
use crate::log_info;
use crate::state;
use crate::ui::router::set_back_screen;
use crate::ui::router::{navigate_to, go_back};
use crate::ui::screens::generic_question_screen::create_generic_question_screen;
use crate::ui::screens::show_alert;
use alloc::string::String;
use slint::ToSharedString;

use crate::common::ui_draw_qr;

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
pub fn create_signed_screen(ui: &Rc<MainWindow>) {
    ui.set_header_title(slint::SharedString::from("SCAN TRANSACTION"));

    let address_x = 40.0;
    let address_y = 200.0;
    let address_w = 320.0;
    let address_h = 35.0;

    set_back_screen(Screen::Home);

    ui.on_pressed(|_| {
      navigate_to(Screen::Home);
    });

    create_generic_question_screen(ui, "SCAN TRANSACTION", "", "DONE", None, ||{
      navigate_to(Screen::Home);
    }, ||{});

    if let Some(envelope) = state().lock().get_parsed_tx() {
      let keypair = firmware().vault.lock().get_stellar_keypair();
      if let Err(e) = keypair {
          show_alert(format!("{}", e).as_str().into());
          return;
      }
      let keypair = keypair.unwrap();

      let signed_tx = StellarTransactionSigner::sign_transaction(&envelope, keypair);
      if let Err(e) = signed_tx {
          log_info!("Error signing transaction: {:?}", e);
          show_alert(format!("{}", e).as_str().into());
          return;
      }
      let signed_tx = signed_tx.unwrap();

      let serialized_tx = StellarTransactionSerializer::serialize_to_base64(&signed_tx);
      if let Err(e) = serialized_tx {
          log_info!("Error serializing transaction: {:?}", e);
          show_alert(format!("{}", e).as_str().into());
          return;
      }
      let serialized_tx = serialized_tx.unwrap();

      let qr_data = format!("https://app.hito.dev/eth/tx/#!{}", serialized_tx);

      ui_draw_qr(&qr_data);
    } else {
        log_info!("No parsed transaction found in state");
    }
}