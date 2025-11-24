use crate::{STATE, ui::CallbackController, hito_firmware::HitoFirmware};
use crate::slint_generatedMainWindow::SendScreenState;
use crate::slint_generatedMainWindow::SendStellarState;
use crate::slint_generatedMainWindow::MainWindow;
use crate::slint_generatedMainWindow::Protocol;
use crate::slint_generatedMainWindow::Router;
use crate::slint_generatedMainWindow::ScreenEnum;
use crate::log_info;
use slint::ComponentHandle;
use crate::crypto::libcrypt0pro::stellar::StellarTransactionParser;
use crate::crypto::libcrypt0pro::stellar::StellarWallet;
extern crate alloc;
use alloc::format;


#[cfg(feature = "minifb")]
use crate::drivers::minifb::SocketProtocol;

#[cfg(feature = "zephyr")]
use crate::drivers::zephyr::{NFCProtocol, BLEProtocol};

use alloc::string::String;

const ADDRESS_SHORT_LENGTH: usize = 10;

#[cfg(feature = "zephyr")]
static NFC_HARDWARE_HANDLER: NFCProtocol = NFCProtocol::new();
#[cfg(feature = "zephyr")]
static BLE_HARDWARE_HANDLER: BLEProtocol = BLEProtocol::new();
#[cfg(feature = "minifb")]
static mut SOCKET_PROTOCOL_HANDLER: Option<SocketProtocol> = None;

pub struct SendScreenCallbackController;

static mut DATA: Option<alloc::vec::Vec<u8>> = None;

enum CallType {
    Stellar,
    Unknown
}

fn str_to_call_type(s: &str) -> CallType {
    if s.starts_with("stellar.sign:") {
        CallType::Stellar
    } else {
        CallType::Unknown
    }
}

fn handle_stellar_transaction(tx_data: &str, ui: &MainWindow) {
    let send_screen_state = ui.global::<SendScreenState>();
    match StellarTransactionParser::parse_transaction(tx_data) {
        Ok(parsed_tx) => {
            use crate::crypto::crypt0::hex_to_bytes;

            // log_info!("Parsed Stellar transaction: {:#?}", parsed_tx);

            // Extract amount from the first payment or create_account operation
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

            if amount.is_none() {
                // log_info!("No payment or create_account operation found in transaction");
                send_screen_state.set_error_occurred(true);
                send_screen_state.set_error_message(slint::SharedString::from("No payment or create_account operation found"));
                return;
            }

            let amount = amount.unwrap();

            // Extract destination hex address
            let dest_hex = if let Some(first_op) = parsed_tx.operations.first() {
                match &first_op.details {
                    crate::crypto::libcrypt0pro::stellar::OperationDetails::Payment { destination, .. } => {
                        match destination {
                            crate::crypto::libcrypt0pro::stellar::ParsedMuxedAccount::Ed25519 {account_id} => Some(account_id.clone()),
                            crate::crypto::libcrypt0pro::stellar::ParsedMuxedAccount::MuxedEd25519 { id: _, account_id } => Some(account_id.clone()),
                        }
                    }
                    crate::crypto::libcrypt0pro::stellar::OperationDetails::CreateAccount { destination, .. } => {
                        Some(destination.clone())
                    }
                    _ => None
                }
            } else {
                None
            };

            if dest_hex.is_none() {
                // log_info!("No destination address found in transaction");
                send_screen_state.set_error_occurred(true);
                send_screen_state.set_error_message(slint::SharedString::from("No destination address found"));
                return;
            }

            let dest_hex = dest_hex.unwrap();
            let mut dest_bytes = [0u8; 32];
            dest_bytes.copy_from_slice(&hex_to_bytes(&dest_hex).unwrap());
            let destination = StellarWallet::encode_stellar_address(&dest_bytes);
            if destination.is_err() {
                // log_info!("Failed to encode destination address: {:?}", destination.err());
                send_screen_state.set_error_occurred(true);
                send_screen_state.set_error_message(slint::SharedString::from("Failed to encode destination address"));
                return;
            }
            let destination_short = format!("{}...\n{}", &destination.as_ref().unwrap()[0..ADDRESS_SHORT_LENGTH/2], &destination.as_ref().unwrap()[destination.as_ref().unwrap().len()-ADDRESS_SHORT_LENGTH/2..]);

            // Update UI with transaction details
            let send_stellar_state = ui.global::<SendStellarState>();
            send_stellar_state.set_amount(amount.parse::<f32>().unwrap_or(0.0));
            send_stellar_state.set_destination_short(slint::SharedString::from(destination_short));
            send_screen_state.invoke_send_requested();
        }
        Err(e) => {
            send_screen_state.set_error_occurred(true);
            send_screen_state.set_error_message(slint::SharedString::from(format!("Failed to parse Stellar transaction:\n{:?}", e)));
        }
    }
}

fn parse_data(data: &[u8], send_screen_state: &SendScreenState, ui: &MainWindow) {
    let mut start_from = 0;
    #[cfg(feature = "zephyr")]
    {
    if data[0] == 0x02 || (data[0] == 'e' as u8 && data[1] == 'n' as u8) {
        start_from = 3;
      }
    }
    log_info!("Data received: {:?}", String::from_utf8_lossy(&data[start_from..]));
    let utf8_data = String::from_utf8_lossy(&data[start_from..]);

    match str_to_call_type(&utf8_data) {
        CallType::Stellar => {
            handle_stellar_transaction(&utf8_data["stellar.sign:".len()..], ui);
        },
        CallType::Unknown => {
            send_screen_state.set_error_occurred(true);
            send_screen_state.set_error_message(slint::SharedString::from("Unknown call type received"));
        }
    }
}

impl CallbackController for SendScreenCallbackController {
    fn register_main_window_callbacks(&self, ui: &MainWindow, firmware: &mut HitoFirmware) {
        ui.global::<SendScreenState>().on_protocol_changed(move || {
          let s = STATE.get().unwrap().lock();
          s.mark_protocol_requested();
          // log_info!("Protocol change requested from UI");
        });
    }
    fn handle_loop_events(&self, ui: &MainWindow, firmware: &mut HitoFirmware) {
        let s = STATE.get().unwrap().lock();
        // Device info request handling
        let send_screen_state = ui.global::<SendScreenState>();
        let router = ui.global::<Router>();
        if s.is_protocol_change_requested() {
            #[cfg(feature = "zephyr")]
            {
              let protocol = send_screen_state.get_current_protocol();
              match protocol {
                  Protocol::NFC => {
                    NFC_HARDWARE_HANDLER.init("Thou shall not use std!");
                    // log_info!("Protocol set to NFC - NFC handler initialized");
                    BLE_HARDWARE_HANDLER.stop();
                  },
                  Protocol::Bluetooth => {
                    BLE_HARDWARE_HANDLER.init();
                    // log_info!("Protocol set to Bluetooth - BLE handler initialized");
                    NFC_HARDWARE_HANDLER.stop();
                  },
              }
            }
            s.clear_protocol_change_requested();
          }
        if router.get_current() == ScreenEnum::Send {
            #[cfg(feature = "minifb")]
            { unsafe {
                if SOCKET_PROTOCOL_HANDLER.is_none() {
                    let mut socket = SocketProtocol::new();
                    match socket.init() {
                        Ok(_) => {
                            unsafe {
                                SOCKET_PROTOCOL_HANDLER = Some(socket);
                            }
                            // log_info!("Socket protocol initialized for Send screen");
                        },
                        Err(e) => {
                            // log_info!("Failed to initialize socket protocol: {:?}", e);
                            send_screen_state.set_error_occurred(true);
                            send_screen_state.set_error_message(slint::SharedString::from("Failed to initialize socket protocol"));
                        }
                    }
                }
                if let Some(ref mut socket) = SOCKET_PROTOCOL_HANDLER {
                  let _ = socket.accept_connection();

                  // Check for incoming data
                  if socket.is_connected() {
                      match socket.receive() {
                          Ok(Some(data)) => {
                            unsafe {
                              DATA = Some(data);
                              if let Some(ref data) = DATA {
                                parse_data(&data, &send_screen_state, ui);
                              }
                            }
                          },
                          Ok(None) => {
                              // No data available
                          },
                          Err(e) => {
                              // log_info!("Linux Socket: Receive error: {:?}", e);
                      }
                    }
                  }
                }
              }
            }
            #[cfg(feature = "zephyr")]
            {
              let protocol = send_screen_state.get_current_protocol();
              match protocol {
                  Protocol::NFC => {
                    unsafe {
                      DATA = NFC_HARDWARE_HANDLER.get_data();
                    }
                  },
                  Protocol::Bluetooth => {
                    unsafe {
                      DATA = BLE_HARDWARE_HANDLER.get_data();
                    }
                  }
              }
              unsafe {
                if let Some(ref data) = DATA {
                  parse_data(&data, &send_screen_state, ui);
                }
              }
            }
        } else {
            // Not on Send screen, close socket if open
            #[cfg(feature = "minifb")]
            {
              unsafe {
                if let Some(mut socket) = SOCKET_PROTOCOL_HANDLER.take() {
                    socket.close();
                    send_screen_state.set_error_occurred(false);
                    send_screen_state.set_error_message(slint::SharedString::from(""));
                    // log_info!("Not on Send screen - Socket closed");
                }
              }
            }
        }
    }
}