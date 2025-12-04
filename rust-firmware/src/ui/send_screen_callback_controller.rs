use crate::crypto::crypt0::bytes_to_hex;
use crate::firmware_state::FirmwareState;
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
use slint::ToSharedString;


#[cfg(feature = "minifb")]
use crate::drivers::minifb::SocketProtocol;

#[cfg(feature = "zephyr")]
use crate::drivers::zephyr::{BLEProtocol, NFCProtocol};

use alloc::string::String;

const ADDRESS_SHORT_LENGTH: usize = 10;

#[cfg(feature = "zephyr")]
static mut NFC_HARDWARE_HANDLER: Option<NFCProtocol> = None;
#[cfg(feature = "zephyr")]
static mut BLE_HARDWARE_HANDLER: Option<BLEProtocol> = None;

#[cfg(feature = "minifb")]
static mut SOCKET_PROTOCOL_HANDLER: Option<SocketProtocol> = None;

pub struct SendScreenCallbackController;

static mut DATA: Option<alloc::vec::Vec<u8>> = None;

use crate::crypto::libcrypt0pro::stellar::{
    NETWORK_ID_MAINNET,
    NETWORK_ID_TESTNET,
    NETWORK_ID_FUTURENET,
};

enum CallType {
    Stellar,
    Unknown
}

fn shorten_address(address: &str) -> alloc::string::String {
    if address.len() <= 17 {
        return alloc::string::String::from(address);
    }
    let first_quarter = &address[0..4];

    let last_quarter = &address[address.len()-4..];
    let last = last_quarter.to_shared_string();
    alloc::format!("{}..{}", first_quarter, last)
}

fn str_to_call_type(s: &str) -> CallType {
    if s.starts_with("stellar.sign:") {
        CallType::Stellar
    } else {
        CallType::Unknown
    }
}

fn handle_stellar_transaction(tx_data: &str, ui: &MainWindow, state: &FirmwareState) {
    let send_screen_state = ui.global::<SendScreenState>();
    match StellarTransactionParser::parse_transaction(tx_data) {
        Ok(parsed_tx) => {
            use crate::crypto::crypt0::hex_to_bytes;

            let send_stellar_state = ui.global::<SendStellarState>();
            let pk_vec = hex_to_bytes(&parsed_tx.source_account).expect("invalid hex for source_account");
            let pk: [u8; 32] = pk_vec
                                    .as_slice()
                                    .try_into()
                                    .expect("pubkey length != 32");
            let address = StellarWallet::encode_stellar_address(&pk).expect("failed to encode stellar address");
            if address != state.get_stellar_address().unwrap() {
                //log_info!("Source address mismatch: expected {}, got {}", send_stellar_state.get_source_short().to_string(), address);
                send_stellar_state.invoke_error_occurred();
                send_screen_state.set_error_occurred(true);
                send_screen_state.set_error_message(slint::SharedString::from("Wallet is not paired"));
                return;
            }

            let network_hash_hex = bytes_to_hex(&parsed_tx.network_hash).to_shared_string();
            match network_hash_hex.as_str() {
                NETWORK_ID_TESTNET => {
                    send_stellar_state.set_network("Testnet".to_shared_string());
                },
                NETWORK_ID_MAINNET => {
                    send_stellar_state.set_network("Mainnet".to_shared_string());
                },
                NETWORK_ID_FUTURENET => {
                    send_stellar_state.set_network("Futurenet".to_shared_string());
                },
                _ => {
                  send_stellar_state.invoke_error_occurred();
                  send_screen_state.set_error_occurred(true);
                  send_screen_state.set_error_message(slint::SharedString::from("Unknown network"));
                }
            }


            send_stellar_state.set_source_short(slint::SharedString::from(&shorten_address(&address)));
            send_stellar_state.set_sequence(slint::SharedString::from(format!("{}", parsed_tx.sequence_number)));

            let memo = parsed_tx.memo.clone().unwrap().value.unwrap_or(String::from("None"));
            send_stellar_state.set_memo_summary(slint::SharedString::from(&memo));

            send_stellar_state.set_fee(slint::SharedString::from(StellarTransactionParser::stroops_to_xlm_string(parsed_tx.fee)));

            let asset_code = if parsed_tx.operations.len() > 0 {
                match &parsed_tx.operations[0].details {
                    crate::crypto::libcrypt0pro::stellar::OperationDetails::Payment { asset, .. } => {
                        match asset {
                            crate::crypto::libcrypt0pro::stellar::ParsedAsset::Native => "XLM".to_shared_string(),
                            crate::crypto::libcrypt0pro::stellar::ParsedAsset::CreditAlphanum4 { code, .. } => code.clone().to_shared_string(),
                            crate::crypto::libcrypt0pro::stellar::ParsedAsset::CreditAlphanum12 { code, .. } => code.clone().to_shared_string(),
                        }
                    }
                    crate::crypto::libcrypt0pro::stellar::OperationDetails::CreateAccount { .. } => "XLM".to_shared_string(),
                    _ => "XLM".to_shared_string()
                }
            } else {
                "XLM".to_shared_string()
            };  
            send_stellar_state.set_asset_code(asset_code);
            send_stellar_state.set_op_count(parsed_tx.operations.len() as i32);
            let asset_issuer = if parsed_tx.operations.len() > 0 {
                match &parsed_tx.operations[0].details {
                    crate::crypto::libcrypt0pro::stellar::OperationDetails::Payment { asset, .. } => {
                        match asset {
                            crate::crypto::libcrypt0pro::stellar::ParsedAsset::Native => "Native".to_shared_string(),
                            crate::crypto::libcrypt0pro::stellar::ParsedAsset::CreditAlphanum4 { issuer, .. } => issuer.clone().to_shared_string(),
                            crate::crypto::libcrypt0pro::stellar::ParsedAsset::CreditAlphanum12 { issuer, .. } => issuer.clone().to_shared_string(),
                        }
                    }
                    crate::crypto::libcrypt0pro::stellar::OperationDetails::CreateAccount { .. } => "Native".to_shared_string(),
                    _ => "".to_shared_string()
                }
            } else {
                "".to_shared_string()
            };  
            send_stellar_state.set_asset_issuer_short(asset_issuer);


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
                //log_info!("No payment or create_account operation found in transaction");
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
                // //log_info!("No destination address found in transaction");
                send_screen_state.set_error_occurred(true);
                send_screen_state.set_error_message(slint::SharedString::from("No destination address found"));
                return;
            }

            let dest_hex = dest_hex.unwrap();
            let mut dest_bytes = [0u8; 32];
            dest_bytes.copy_from_slice(&hex_to_bytes(&dest_hex).unwrap());
            let destination = StellarWallet::encode_stellar_address(&dest_bytes);
            if destination.is_err() {
                // //log_info!("Failed to encode destination address: {:?}", destination.err());
                send_screen_state.set_error_occurred(true);
                send_screen_state.set_error_message(slint::SharedString::from("Failed to encode destination address"));
                return;
            }
            let destination_short = shorten_address(&destination.as_ref().unwrap());

            // Update UI with transaction details
            send_stellar_state.set_amount(slint::SharedString::from(&amount));
            send_stellar_state.set_destination_short(slint::SharedString::from(&destination_short));
            send_screen_state.invoke_send_requested();
            //log_info!("Stellar transaction parsed successfully: amount={}, destination={}", amount, destination.as_ref().unwrap());
            state.set_parsed_tx(parsed_tx);
        }
        Err(e) => {
            send_screen_state.set_error_occurred(true);
            send_screen_state.set_error_message(slint::SharedString::from(format!("Failed to parse Stellar transaction:\n{:?}", e.to_shared_string().replace('"', ""))));
        }
    }
}

fn parse_data(data: &[u8], send_screen_state: &SendScreenState, ui: &MainWindow, state: &FirmwareState) {
    let mut start_from = 0;
    #[cfg(feature = "zephyr")]
    {
    if data[0] == 0x02 || (data[0] == 'e' as u8 && data[1] == 'n' as u8) {
        start_from = 3;
      }
    }
    let utf8_data = String::from_utf8_lossy(&data[start_from..]);

    match str_to_call_type(&utf8_data) {
        CallType::Stellar => {
            //log_info!("Call type identified as Stellar transaction");
            handle_stellar_transaction(&utf8_data["stellar.sign:".len()..], ui, state);
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
          // //log_info!("Protocol change requested from UI");
        });
    }
    fn handle_loop_events(&self, ui: &MainWindow, firmware: &mut HitoFirmware) {
        let s = STATE.get().unwrap().lock();
        // Device info request handling
        let send_screen_state = ui.global::<SendScreenState>();
        let router = ui.global::<Router>();
        if s.is_protocol_change_requested() {
            s.set_stellar_address(firmware.vault.get_stellar_address().unwrap());
            #[cfg(feature = "zephyr")]
            {
              let protocol = send_screen_state.get_current_protocol();
              match protocol {
                  Protocol::NFC => {
                    unsafe {
                      NFC_HARDWARE_HANDLER = Some(NFCProtocol::new());
                      if let Some(ref mut handler) = NFC_HARDWARE_HANDLER {
                        handler.init(format!("app.hito.dev/#eth/send/!from={}", s.get_stellar_address().unwrap()).as_str());
                      }
                      // //log_info!("Protocol set to NFC - NFC handler initialized");
                      if let Some(ref mut handler) = BLE_HARDWARE_HANDLER {
                        handler.stop();
                      }
                      BLE_HARDWARE_HANDLER = None;
                    }
                  },
                  Protocol::Bluetooth => {
                    unsafe {
                      BLE_HARDWARE_HANDLER = Some(BLEProtocol::new());
                      if let Some(ref mut handler) = BLE_HARDWARE_HANDLER {
                        handler.init();
                      }
                      // //log_info!("Protocol set to Bluetooth - BLE handler initialized");
                      if let Some(ref mut handler) = NFC_HARDWARE_HANDLER {
                        handler.stop();
                      }
                      NFC_HARDWARE_HANDLER = None;
                    }
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
                            // //log_info!("Socket protocol initialized for Send screen");
                        },
                        Err(e) => {
                            // //log_info!("Failed to initialize socket protocol: {:?}", e);
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
                                parse_data(&data, &send_screen_state, ui, &s);
                              }
                            }
                          },
                          Ok(None) => {
                              // No data available
                          },
                          Err(e) => {
                              // //log_info!("Linux Socket: Receive error: {:?}", e);
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
                      if let Some(ref mut nfc_handler) = NFC_HARDWARE_HANDLER {
                        DATA = nfc_handler.get_data();
                      }
                    }
                  },
                  Protocol::Bluetooth => {
                    unsafe {
                      if let Some(ref mut ble_handler) = BLE_HARDWARE_HANDLER {
                        DATA = ble_handler.get_data();
                      }
                    }
                  }
              }
              unsafe {
                if let Some(ref data) = DATA {
                  parse_data(&data, &send_screen_state, ui, &s);
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
                    // //log_info!("Not on Send screen - Socket closed");
                }
              }
            }
            #[cfg(feature = "zephyr")]
            {
              unsafe {
                if let Some(ref mut nfc_handler) = NFC_HARDWARE_HANDLER {
                    nfc_handler.stop();
                }
                if let Some(ref mut ble_handler) = BLE_HARDWARE_HANDLER {
                    ble_handler.stop();
                }
                NFC_HARDWARE_HANDLER = None;
                BLE_HARDWARE_HANDLER = None;
              }
            }
        }
    }
}