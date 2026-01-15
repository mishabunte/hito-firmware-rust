//! Screen creation functions for the firmware UI
//! 
//! This module contains functions to create and configure each screen in the application.
//! Each function sets up the UI items, header, and callbacks for its respective screen.

extern crate alloc;
use alloc::vec;
use alloc::rc::Rc;

use slint::Model;
use slint::ModelRc;
use slint::SharedString;
use slint::VecModel;
use slint::format;

use crate::STATE;
use crate::ScreenItem;
use crate::crypto::crypt0::hex_to_bytes;
use crate::crypto::libcrypt0pro::stellar::OperationDetails;
use crate::crypto::libcrypt0pro::stellar::Asset;
use crate::crypto::libcrypt0pro::stellar::MuxedAccount;
use crate::crypto::libcrypt0pro::stellar::Transaction;
use crate::crypto::libcrypt0pro::stellar::StellarWallet;
use crate::crypto::libcrypt0pro::stellar::TransactionEnvelope;
use crate::firmware;
use crate::slint_generatedMainWindow::{MainWindow, ScreenButton};
use crate::log_info;
use crate::state;
use crate::ui;
use crate::ui::router::set_back_screen;
use crate::ui::router::{navigate_to, go_back};
use crate::ui::screens::generic_question_screen::create_generic_question_screen;
use crate::ui::screens::show_alert;
use slint::ToSharedString;
use alloc::vec::Vec;

use crate::crypto::libcrypt0pro::stellar::{
    NETWORK_ID_MAINNET,
    NETWORK_ID_TESTNET,
    NETWORK_ID_FUTURENET,
};

static mut CURRENT_OPERATION_INDEX: Option<usize> = None;

const NEXT_OP_TEXT: &str = "next >";
const PREV_OP_TEXT: &str = "< prev";
const SIGN_TEXT: &str = "SIGN";
const DETAILS_TEXT: &str = "DETAILS";
const OPERATIONS_TEXT: &str = "OPERATIONS";

use super::Screen;

use crate::crypto::libcrypt0pro::stellar::StellarTransactionParser;

fn shorten_address(address: &str) -> alloc::string::String {
    if address.len() <= 17 {
        return alloc::string::String::from(address);
    }
    let first_quarter = &address[0..4];

    let last_quarter = &address[address.len()-4..];
    let last = last_quarter.to_shared_string();
    alloc::format!("{}..{}", first_quarter, last)
}

fn create_details_page(ui: &Rc<MainWindow>) {
    if let Some(envelope) = state().lock().get_parsed_tx() {
      match &envelope {
        TransactionEnvelope::Transaction(parsed_tx) => {
          let top_x = 20.0;
          let top_y = 45.0;
          let y_gap = 40.0;
          let x_gap = 160.0;
          let coords = vec![
              (top_x, top_y),
              (top_x, top_y + y_gap),
              (top_x, top_y + 2.0 * y_gap),
              (top_x + x_gap, top_y),
              (top_x + x_gap, top_y + y_gap),
              (top_x + x_gap, top_y + 2.0 * y_gap),
          ];
          let address = parsed_tx.source_account.clone();
          if let Ok(expected) = firmware().vault.lock().get_stellar_address() {
              if address != expected {
                  show_alert("Wallet is not paired");
                  return;
              }
          } else {
              show_alert("Failed to get source account");
              return;
          }
          let source_short = shorten_address(&address);

          let sequence = parsed_tx.sequence_number;

          let fee = StellarTransactionParser::stroops_to_xlm_string(parsed_tx.fee);

          let op_count = parsed_tx.operations.len();

          let network_hash_hex = parsed_tx
              .network_hash
              .as_ref()
              .unwrap()
              .get_hash_hex();
          let network: Result<SharedString, ()> = match network_hash_hex {
              NETWORK_ID_TESTNET => {
                  Ok("Testnet".to_shared_string())
              },
              NETWORK_ID_MAINNET => {
                  Ok("Mainnet".to_shared_string())
              },
              NETWORK_ID_FUTURENET => {
                  Ok("Futurenet".to_shared_string())
              },
              _ => {
                  Err(())
              }
          };

          if let Err(_) = network {
              show_alert("Unknown network");
              return;
          }

          let network = network.unwrap();

          let memo = match &parsed_tx.memo {
              Some(memo) => match memo.value {
                Some(ref text) => {
                  text.to_shared_string()
                }
                None => {
                  "".to_shared_string()
                }
              }
              None => "".to_shared_string(),
          };

          let max_memo_len_line = 18;
          // Split memo into multiple lines if too long. Is space found, split there
          let memo = if memo.len() > max_memo_len_line {
              let mut memo_lines: Vec<SharedString> = vec![];
              let mut start = 0;
              while start < memo.len() {
                  let end = if start + max_memo_len_line >= memo.len() {
                      memo.len()
                  } else {
                      // Look for last space within next max_memo_len_line chars
                      match memo.as_str()[start..start + max_memo_len_line].rfind(' ') {
                          Some(space_index) => start + space_index,
                          None => start + max_memo_len_line,
                      }
                  };
                  let segment = memo.as_str()[start..end].trim();
                  memo_lines.push(segment.to_shared_string());
                  start = end;
              }
              memo_lines.join("\n").to_shared_string()
          } else {
              memo
          };

          let details_lines = vec![
              alloc::format!("Source:\n{}", source_short),
              alloc::format!("Sequence:\n{}", sequence),
              alloc::format!("Fee:\n{} XLM", fee),
              alloc::format!("Network:\n{}", network),
              alloc::format!("Operations:\n{} op(s)", op_count),
              alloc::format!("Memo:\n{}", memo),
          ];

          let small_text_items: Vec<ScreenItem> = details_lines.iter().map(|line| {
              ScreenItem {
                  text: line.to_shared_string(),
                  width: 240.0,
                  height: 60.0,
                  x: coords[details_lines.iter().position(|x| x == line).unwrap()].0,
                  y: coords[details_lines.iter().position(|x| x == line).unwrap()].1,
              }
          }).collect();  
          ui.set_small_text_items(ModelRc::new(VecModel::from(small_text_items)));

          let button_w = 130.0;
          let button_h = 40.0;
          let cancel_x = 20.0;
          let cancel_y = 190.0;
          let confirm_x = 170.0;
          let confirm_y = 190.0;

          let buttons_vec = vec![
            ScreenButton {
                text: SIGN_TEXT.into(),
                width: button_w,
                height: button_h,
                has_border: false,
                x: confirm_x,
                y: confirm_y,
                inverted: false,
            },
            ScreenButton {
                text: OPERATIONS_TEXT.into(),
                width: button_w,
                height: button_h,
                has_border: false,
                x: cancel_x,
                y: cancel_y,
                inverted: false,
            },
            // ScreenButton {
            //     text: "more info".into(),
            //     width: button_w + 10.0,
            //     height: button_h,
            //     has_border: true,
            //     x: confirm_x,
            //     y: confirm_y - button_h,
            //     inverted: true,
            // },
          ];

          let buttons = ModelRc::new(VecModel::from(buttons_vec));
          ui.set_buttons(buttons);
        
        },
        TransactionEnvelope::FeeBump(_) => {
          drop(envelope);
          show_alert("Fee bump transactions not supported");
          return;
        }
      }
    }
}

fn create_operations_page(ui: &Rc<MainWindow>) {
  if let Some(envelope) = state().lock().get_parsed_tx() {
    match &envelope {
      TransactionEnvelope::Transaction(parsed_tx) => {
        let operations_count = parsed_tx.operations.len();
        let mut current_index = unsafe { CURRENT_OPERATION_INDEX };
        if current_index.is_none() {
            current_index = Some(0);
            unsafe { CURRENT_OPERATION_INDEX = current_index; }
        }
        let current_index = current_index.unwrap();
        if current_index >= operations_count {
            unsafe { CURRENT_OPERATION_INDEX = Some(0); }
            return;
        }
        let button_w = 130.0;
        let button_h = 40.0;
        let left_x = 20.0;
        let cancel_x = 20.0;
        let confirm_x = 170.0;
        let confirm_y = 190.0;

        let memo_header = ScreenItem {
            text: format!("Operation #{}:", current_index + 1).into(),
            width: 280.0,
            height: 25.0,
            x: left_x,
            y: 45.0,
        };

        ui.set_text_wrapping(true);

        ui.set_items(ModelRc::new(VecModel::from(vec![memo_header])));

        let mut buttons_vec = vec![
        ];
        let next_text = if current_index < operations_count - 1 {
          NEXT_OP_TEXT
        } else {
          SIGN_TEXT
        };
        buttons_vec.push(ScreenButton {
          text: next_text.into(),
          width: button_w,
          height: button_h,
          has_border: false,
          x: confirm_x,
          y: confirm_y,
          inverted: false,
        });
        let prev_text = if current_index > 0 {
            PREV_OP_TEXT
        } else {
            DETAILS_TEXT
        };
        buttons_vec.push(ScreenButton {
          text: prev_text.into(),
          width: button_w,
          height: button_h,
          has_border: false,
          x: cancel_x,
          y: confirm_y,
          inverted: false,
        });
        let current_operation = &parsed_tx.operations[current_index];
        let mut operation_lines: Vec<SharedString> = vec![];
        operation_lines.push(format!("Type: {:?}", current_operation.operation_type));
        let source_account = if let Some(source) = &current_operation.source_account {
            source.clone().to_shared_string()
        } else {
            "None".to_shared_string()
        };
        if source_account != "None" {
          operation_lines.push(format!("Source account: {}", shorten_address(&source_account)).into());
        }
        match &current_operation.details {
            OperationDetails::Payment { destination, amount, asset } => {
                let dest_address = match destination {
                    MuxedAccount::Ed25519 { account_id } => {
                      account_id.clone()
                    },
                    MuxedAccount::MuxedEd25519 { id: _, account_id } => {
                        account_id.clone()
                    },
                };
                let asset_issuer = match asset {
                    Asset::Native => "Native".to_shared_string(),
                    Asset::CreditAlphanum4 { issuer, .. } => {
                      shorten_address(issuer).to_shared_string()
                    },
                    Asset::CreditAlphanum12 { issuer, .. } => {
                      shorten_address(issuer).to_shared_string()
                    },
                };
                operation_lines.push(format!("Asset issuer: {}", asset_issuer));
                let dest_short = shorten_address(&dest_address);
                operation_lines.push(format!("Destination: {}", dest_short));
                let amount_str = StellarTransactionParser::stroops_to_xlm_string(*amount);
                let asset_str = match asset {
                    Asset::Native => "XLM".to_shared_string(),
                    Asset::CreditAlphanum4 { code, .. } => code.clone().to_shared_string(),
                    Asset::CreditAlphanum12 { code, .. } => code.clone().to_shared_string(),
                };
                operation_lines.push(format!("Amount: {} {}", amount_str, asset_str));
            },
            OperationDetails::CreateAccount { destination, starting_balance } => {
                let dest_short = shorten_address(&destination).to_shared_string();
                operation_lines.push(format!("Destination: {}", dest_short));
                let amount_str = StellarTransactionParser::stroops_to_xlm_string(*starting_balance);
                operation_lines.push(format!("Starting balance: {} XLM", amount_str));
            },

            OperationDetails::PathPaymentStrictSend { send_asset, send_amount, destination, dest_asset, dest_min, path } => {
                let send_asset_issuer = match send_asset {
                    Asset::Native => "Native".to_shared_string(),
                    Asset::CreditAlphanum4 { issuer, .. } => {
                      shorten_address(issuer).to_shared_string()
                    },
                    Asset::CreditAlphanum12 { issuer, .. } => {
                      shorten_address(issuer).to_shared_string()
                    },
                };
                let send_asset_code = match send_asset {
                    Asset::Native => "XLM".to_shared_string(),
                    Asset::CreditAlphanum4 { code, .. } => code.clone().to_shared_string(),
                    Asset::CreditAlphanum12 { code, .. } => code.clone().to_shared_string(),
                };
                let send_amount_str = StellarTransactionParser::stroops_to_xlm_string(*send_amount);
                
                
                let dest_address = match destination {
                    MuxedAccount::Ed25519 { account_id } => {
                      account_id.clone()
                    },
                    MuxedAccount::MuxedEd25519 { id: _, account_id } => {
                        account_id.clone()
                    },
                };
                let dest_short = shorten_address(&dest_address).to_shared_string();

                let dest_asset_issuer = match dest_asset {
                    Asset::Native => "Native".to_shared_string(),
                    Asset::CreditAlphanum4 { issuer, .. } => {
                      shorten_address(issuer).to_shared_string()
                    },
                    Asset::CreditAlphanum12 { issuer, .. } => {
                      shorten_address(issuer).to_shared_string()
                    },
                };
                let dest_asset_code = match dest_asset {
                    Asset::Native => "XLM".to_shared_string(),
                    Asset::CreditAlphanum4 { code, .. } => code.clone().to_shared_string(),
                    Asset::CreditAlphanum12 { code, .. } => code.clone().to_shared_string(),
                };
                let dest_min_str = StellarTransactionParser::stroops_to_xlm_string(*dest_min);

                operation_lines.push(format!("Send: {} {}", send_amount_str, send_asset_code));
                operation_lines.push(format!("Send asset issuer: {}", send_asset_issuer));
                operation_lines.push(format!("Destination: {}", dest_short));
                
                operation_lines.push(format!("Dest asset issuer: {}", dest_asset_issuer));
                operation_lines.push(format!("Dest min: {} {}", dest_min_str, dest_asset_code));
            },
            _ => {
                operation_lines.push("Details: (not implemented)".to_shared_string());
            }
        }
        let operation_items: Vec<ScreenItem> = operation_lines.iter().enumerate().map(|(i, line)| {
            ScreenItem {
                text: line.to_shared_string(),
                width: 280.0,
                height: 25.0,
                x: left_x,
                y: 80.0 + (i as f32) * 20.0,
            }
        }).collect();
        ui.set_small_text_items(ModelRc::new(VecModel::from(operation_items)));
        let buttons = ModelRc::new(VecModel::from(buttons_vec));
        ui.set_buttons(buttons);
      },
      TransactionEnvelope::FeeBump(_) => {
        drop(envelope);
        show_alert("Fee bump transactions not supported");
        return;
      }
    }
  }
}

/// Create the main send stellar screen (generic info page)
pub fn create_send_stellar_operations_screen(ui: &Rc<MainWindow>) {
    ui.set_header_title("SEND STELLAR".into());
    set_back_screen(Screen::Home);
    let ui_weak = Rc::downgrade(ui);
    ui.on_pressed(move |item| {
      if let Some(ui) = ui_weak.upgrade() {
        let text = item.text.as_str();
        if text == SIGN_TEXT {
            unsafe { CURRENT_OPERATION_INDEX = None; }
            navigate_to(Screen::Signed);
        }
        if text == DETAILS_TEXT {
            unsafe { CURRENT_OPERATION_INDEX = None; }
            navigate_to(Screen::SendStellarGeneralInfo);
        }
        if text == NEXT_OP_TEXT {
            let mut current_index = unsafe { CURRENT_OPERATION_INDEX };
            if current_index.is_none() {
                current_index = Some(0);
            }
            let current_index = current_index.unwrap();
            unsafe { CURRENT_OPERATION_INDEX = Some(current_index + 1); }
            create_operations_page(&ui);
        }
        if text == PREV_OP_TEXT {
            let mut current_index = unsafe { CURRENT_OPERATION_INDEX };
            if current_index.is_none() || current_index.unwrap() == 0 {
                unsafe { CURRENT_OPERATION_INDEX = Some(0); }
            } else {
                let current_index = current_index.unwrap();
                unsafe { CURRENT_OPERATION_INDEX = Some(current_index - 1); }
            }
            create_operations_page(&ui);
        }
      }
    });
    create_operations_page(ui);
}

/// Create the send stellar memo screen
pub fn create_send_stellar_general_info_screen(ui: &Rc<MainWindow>) {
    ui.set_header_title("SEND STELLAR".into());
    set_back_screen(Screen::Home);
    ui.on_pressed(move|item| {
        let text = item.text.as_str();
        if text == SIGN_TEXT {
            unsafe { CURRENT_OPERATION_INDEX = None; }
            navigate_to(Screen::Signed);
        }
        if text == OPERATIONS_TEXT {
            navigate_to(Screen::SendStellarOperations);
        }
    });
    create_details_page(ui);
}