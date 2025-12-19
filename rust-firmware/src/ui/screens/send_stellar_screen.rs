//! Screen creation functions for the firmware UI
//! 
//! This module contains functions to create and configure each screen in the application.
//! Each function sets up the UI items, header, and callbacks for its respective screen.

extern crate alloc;
use alloc::vec;
use alloc::rc::Rc;

use slint::ModelRc;
use slint::VecModel;

use crate::STATE;
use crate::ScreenItem;
use crate::slint_generatedMainWindow::{MainWindow, ScreenButton};
use crate::log_info;
use crate::ui::router::{navigate_to, go_back};

use super::Screen;

use crate::crypto::libcrypt0pro::stellar::StellarTransactionParser;

/// Create the main menu screen with navigation options
pub fn create_send_stellar_screen(ui: &Rc<MainWindow>) {
    ui.set_header_title(slint::SharedString::from("SEND STELLAR"));

    let button_x = 35.0;
    let y = 60.0;
    let y_gap = 15.0;
    let button_w = 320.0;
    let button_h = 35.0;

    if let Some(parsed_tx) = STATE.get().unwrap().lock().get_parsed_tx() {
      let amount = parsed_tx.operations.iter().find_map(|op| {
          match &op.details {
              crate::crypto::libcrypt0pro::stellar::OperationDetails::Payment { amount, .. } => {
                  Some(StellarTransactionParser::stroops_to_xlm_string(*amount)) // Convert stroops to XLM
              }
              crate::crypto::libcrypt0pro::stellar::OperationDetails::CreateAccount { starting_balance, .. } => {
                  Some(StellarTransactionParser::stroops_to_xlm_string(*starting_balance)) // Convert stroops to XLM
              }
              _ => None
          }
      });
      let amount_str = amount.clone().unwrap_or("0".into()) + " XLM";
      let items = ModelRc::new(VecModel::from(vec![
        ScreenItem { 
            text: "You send".into(), 
            width: button_w, 
            height: button_h, 
            x: button_x, 
            y: y,
        },
        ScreenItem { 
            text: amount_str.into(), 
            width: button_w, 
            height: button_h, 
            x: button_x, 
            y: y + y_gap,
        },
      ]));
      ui.set_items(items);
    } else {
        log_info!("Creating Send Stellar screen with no parsed transaction");
    }
  
}