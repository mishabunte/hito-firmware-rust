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

use alloc::string::String;

const ADDRESS_SHORT_LENGTH: usize = 10;

pub struct SendScreenCallbackController;

impl CallbackController for SendScreenCallbackController {
    fn register_main_window_callbacks(&self, ui: &MainWindow, firmware: &mut HitoFirmware) {
        ui.global::<SendScreenState>().on_protocol_changed(move || {
          let s = STATE.get().unwrap().lock();
          s.mark_protocol_requested();
          log_info!("Protocol change requested from UI");
        });
    }
    fn handle_loop_events(&self, ui: &MainWindow, firmware: &mut HitoFirmware) {
        let s = STATE.get().unwrap().lock();
        // Device info request handling
        let send_screen_state = ui.global::<SendScreenState>();
        let router = ui.global::<Router>();
        if s.is_protocol_change_requested() {
            let protocol = send_screen_state.get_current_protocol();
            match protocol {
                Protocol::NFC => {
                    // Initialize socket protocol if not already initialized
                    #[cfg(feature = "minifb")]
                    if firmware.protocol.is_none() {
                        let mut socket_protocol = SocketProtocol::new();
                        if let Ok(_) = socket_protocol.init() {
                            firmware.protocol = Some(socket_protocol);
                            log_info!("Protocol set to NFC - Socket initialized");
                        } else {
                            log_info!("Protocol set to NFC - Socket initialization failed");
                        }
                    } else {
                        log_info!("Protocol set to NFC - Socket already initialized");
                    }

                    #[cfg(feature = "zephyr")]
                    log_info!("Protocol set to NFC");
                },
                Protocol::Bluetooth => {
                    // Close socket when switching to NFC
                    #[cfg(feature = "minifb")]
                    if let Some(mut socket) = firmware.protocol.take() {
                        socket.close();
                        log_info!("Protocol set to Bluetooth - Socket closed");
                    } else {
                        log_info!("Protocol set to Bluetooth");
                    }

                    #[cfg(feature = "zephyr")]
                    log_info!("Protocol set to Bluetooth");
                }
            }
            s.clear_protocol_change_requested();
          }
        if router.get_current() == ScreenEnum::Send {
            let protocol = send_screen_state.get_current_protocol();
            match protocol {
                Protocol::NFC => {
                  #[cfg(feature = "minifb")]
                  {
                      if firmware.protocol.is_none() {
                          let mut socket_protocol = SocketProtocol::new();
                          if let Ok(_) = socket_protocol.init() {
                              firmware.protocol = Some(socket_protocol);
                              log_info!("Protocol set to NFC - Socket initialized");
                          } else {
                              log_info!("Protocol set to NFC - Socket initialization failed");
                          }
                      }
                      if let Some(ref mut socket) = firmware.protocol {
                          // Try to accept new connections
                          match socket.accept_connection() {
                              Ok(true) => {
                                  log_info!("Linux Socket: New client connected");
                              },
                              Ok(false) => {
                                  // No new connection, check if we have data
                              },
                              Err(e) => {
                                  log_info!("Linux Socket: Accept error: {:?}", e);
                              }
                          }

                          // Check for incoming data
                          if socket.is_connected() {
                              match socket.receive() {
                                  Ok(Some(data)) => {
                                      log_info!("Linux Socket: Received {} bytes: {:?}", data.len(), data);
                                      // TODO: Process received data here
                                      let utf8_data = String::from_utf8_lossy(&data);
                                      log_info!("Linux Socket: Data as UTF-8: {}", utf8_data);
                                      if utf8_data.starts_with("stellar.sign:") {
                                        let xdr_base64 = &utf8_data["stellar.sign:".len()..];
                                        match StellarTransactionParser::parse_transaction(xdr_base64) {
                                            Ok(parsed_tx) => {
                                                use crate::crypto::crypt0::hex_to_bytes;

                                                log_info!("Parsed Stellar transaction: {:#?}", parsed_tx);

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
                                                    log_info!("No payment or create_account operation found in transaction");
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
                                                    log_info!("No destination address found in transaction");
                                                    send_screen_state.set_error_occurred(true);
                                                    send_screen_state.set_error_message(slint::SharedString::from("No destination address found"));
                                                    return;
                                                }

                                                let dest_hex = dest_hex.unwrap();
                                                let mut dest_bytes = [0u8; 32];
                                                dest_bytes.copy_from_slice(&hex_to_bytes(&dest_hex).unwrap());
                                                let destination = StellarWallet::encode_stellar_address(&dest_bytes);
                                                if destination.is_err() {
                                                    log_info!("Failed to encode destination address: {:?}", destination.err());
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
                                                use std::string::ToString;

                                                send_screen_state.set_error_occurred(true);
                                                send_screen_state.set_error_message(slint::SharedString::from(format!("Failed to parse Stellar transaction:\n{:?}", e.to_string())));
                                            }
                                        }
                                      } else {
                                        send_screen_state.set_error_occurred(true);
                                        send_screen_state.set_error_message(slint::SharedString::from("Unknown payload prefix"));
                                      }

                                      // Echo back for testing
                                      if let Err(e) = socket.send(&data) {
                                          log_info!("Linux Socket: Send error: {:?}", e);
                                      }
                                  },
                                  Ok(None) => {
                                      // No data available
                                  },
                                  Err(e) => {
                                      log_info!("Linux Socket: Receive error: {:?}", e);
                                  }
                              }
                          }
                      }
                  }

                  #[cfg(feature = "zephyr")]
                  log_info!("Polling NFC");
                },
                Protocol::Bluetooth => {
                  // Not implemented yet TODO: Implement Bluetooth handling for Minifb as well for completeness and testing purposes for Bluetooth functionality
                }
            }
        } else {
            // Not on Send screen, close socket if open
            #[cfg(feature = "minifb")]
            if let Some(mut socket) = firmware.protocol.take() {
                socket.close();
                send_screen_state.set_error_occurred(false);
                send_screen_state.set_error_message(slint::SharedString::from(""));
                log_info!("Not on Send screen - Socket closed");
            }
        }
    }
}