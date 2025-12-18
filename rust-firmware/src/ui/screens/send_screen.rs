//! Send Screen - handles protocol-based communication for sending crypto
//! 
//! This module contains the send screen creation and loop event handling.
//! For minifb: uses SocketProtocol
//! For zephyr: uses NFC or Bluetooth depending on the selected protocol

extern crate alloc;
use alloc::vec;
use alloc::vec::Vec;
use alloc::rc::Rc;
use alloc::format;
use alloc::string::String;

use slint::ModelRc;
use slint::VecModel;

use crate::slint_generatedMainWindow::Protocol;
use crate::slint_generatedMainWindow::{MainWindow, ScreenButton, ScreenItem};
use crate::log_info;
use crate::ui::router::{navigate_to, go_back};
use crate::firmware;
use crate::state;

use slint::{ComponentHandle, ToSharedString};

use super::Screen;

// Protocol handlers - stored globally for the main loop
#[cfg(feature = "minifb")]
use crate::drivers::minifb::SocketProtocol;

#[cfg(feature = "zephyr")]
use crate::drivers::zephyr::{NFCProtocol, BLEProtocol};

// Global protocol handler storage
#[cfg(feature = "minifb")]
static mut SOCKET_PROTOCOL_HANDLER: Option<SocketProtocol> = None;

#[cfg(feature = "zephyr")]
static mut NFC_HANDLER: Option<NFCProtocol> = None;

#[cfg(feature = "zephyr")]
static mut BLE_HANDLER: Option<BLEProtocol> = None;

// Global data buffer for received data
static mut RECEIVED_DATA: Option<Vec<u8>> = None;

use crate::crypto::libcrypt0pro::stellar::{
    NETWORK_ID_MAINNET,
    NETWORK_ID_TESTNET,
    NETWORK_ID_FUTURENET,
    StellarTransactionParser,
    StellarWallet,
};

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

fn shorten_address(address: &str) -> String {
    if address.len() <= 17 {
        return String::from(address);
    }
    let first_quarter = &address[0..4];
    let second_quarter = &address[4..8];
    let first = first_quarter.to_shared_string() + " " + second_quarter;

    let last_first_quarter = &address[address.len()-8..address.len()-4];
    let last_quarter = &address[address.len()-4..];
    let last = last_first_quarter.to_shared_string() + " " + last_quarter;
    format!("{}..{}", first, last)
}

/// Handle a Stellar transaction - parse, validate, store in state, and navigate to SendStellar screen
fn handle_stellar_transaction(tx_data: &str, ui: &MainWindow) {
    use crate::crypto::crypt0::hex_to_bytes;

    let vault_arc = firmware().vault.clone();
    let vault = vault_arc.lock();
    
    match StellarTransactionParser::parse_transaction(tx_data) {
        Ok(parsed_tx) => {
            let s = state().lock();
            
            // Validate source account matches our wallet
            let pk_vec = hex_to_bytes(&parsed_tx.source_account).expect("invalid hex for source_account");
            let pk: [u8; 32] = pk_vec
                .as_slice()
                .try_into()
                .expect("pubkey length != 32");
            let address = StellarWallet::encode_stellar_address(&pk).expect("failed to encode stellar address");
            log_info!("Address: {}", address);
            
            if let Ok(our_address) = vault.get_stellar_address() {
                if address != our_address {
                    drop(s);
                    show_error(ui, "Wallet is not paired");
                    return;
                }
            } else {
                log_info!("Address: {}", s.get_stellar_address().unwrap_or("None".into()));
                drop(s);
                show_error(ui, "No wallet configured");
                return;
            }

            // Validate network
            let network_hash_hex = parsed_tx
                .network_hash
                .as_ref()
                .map(|h| h.get_hash_hex());
            
            let network_valid = match network_hash_hex {
                Some(NETWORK_ID_TESTNET) | Some(NETWORK_ID_MAINNET) | Some(NETWORK_ID_FUTURENET) => true,
                _ => false,
            };
            
            if !network_valid {
                drop(s);
                show_error(ui, "Unknown network");
                return;
            }

            // Validate we have a payment or create_account operation
            let has_valid_op = parsed_tx.operations.iter().any(|op| {
                matches!(&op.details,
                    crate::crypto::libcrypt0pro::stellar::OperationDetails::Payment { .. } |
                    crate::crypto::libcrypt0pro::stellar::OperationDetails::CreateAccount { .. }
                )
            });

            if !has_valid_op {
                drop(s);
                show_error(ui, "No payment or create_account operation found");
                return;
            }

            // Extract and validate destination
            let dest_hex = if let Some(first_op) = parsed_tx.operations.first() {
                match &first_op.details {
                    crate::crypto::libcrypt0pro::stellar::OperationDetails::Payment { destination, .. } => {
                        match destination {
                            crate::crypto::libcrypt0pro::stellar::ParsedMuxedAccount::Ed25519 { account_id } => Some(account_id.clone()),
                            crate::crypto::libcrypt0pro::stellar::ParsedMuxedAccount::MuxedEd25519 { account_id, .. } => Some(account_id.clone()),
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
                drop(s);
                show_error(ui, "No destination address found");
                return;
            }

            let dest_hex = dest_hex.unwrap();
            let dest_bytes_vec = match hex_to_bytes(&dest_hex) {
                Some(bytes) if bytes.len() == 32 => bytes,
                _ => {
                    drop(s);
                    show_error(ui, "Invalid destination address");
                    return;
                }
            };
            
            let mut dest_bytes = [0u8; 32];
            dest_bytes.copy_from_slice(&dest_bytes_vec);
            let destination = StellarWallet::encode_stellar_address(&dest_bytes);
            if destination.is_err() {
                drop(s);
                show_error(ui, "Failed to encode destination address");
                return;
            }

            // All validation passed - store the parsed transaction and navigate
            log_info!("Stellar transaction parsed successfully");
            s.set_parsed_tx(parsed_tx);
            drop(s);
            
            // Navigate to SendStellar screen to display transaction details
            navigate_to(Screen::SendStellar);
        }
        Err(e) => {
            let error_msg = format!("Failed to parse transaction:\n{:?}", e);
            show_error(ui, &error_msg);
        }
    }
}

/// Show an error message on the current screen
fn show_error(ui: &MainWindow, message: &str) {
    ui.invoke_clear_screen();
    cleanup_send_screen();
    ui.set_header_title(slint::SharedString::from("ERROR"));
    let items = ModelRc::new(VecModel::from(vec![
        ScreenItem { 
            text: message.into(), 
            width: 320.0 - 40.0, 
            height: 80.0, 
            x: 20.0, 
            y: 80.0,
        },
    ]));
    ui.set_items(items);
    let buttons = ModelRc::new(VecModel::from(vec![
        ScreenButton { 
            text: "Back".into(), 
            width: 320.0, 
            height: 40.0, 
            x: 0.0, 
            has_border: false,
            inverted: false,
            y: 190.0,
        },
    ]));
    ui.set_buttons(buttons);
    let ui_weak = ui.as_weak();
    ui.on_pressed(move |item| {
        if item.text == "Back" {
          crate::ui::navigate_to(Screen::Send);
        }
    });
}


/// Process received data from protocol handlers
fn parse_received_data(data: &[u8], ui: &MainWindow) {
    log_info!("Received {} bytes of data", data.len());
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
            log_info!("Call type identified as Stellar transaction");
            // Extract transaction data after "stellar.sign:"
            let tx_data = &utf8_data["stellar.sign:".len()..];
            handle_stellar_transaction(tx_data, ui);
        },
        CallType::Unknown => {
            show_error(ui, "Unknown call type received");
            cleanup_send_screen();
        }
    }
}

/// Create the send screen with protocol selection UI
pub fn create_send_screen(ui: &Rc<MainWindow>) {
    ui.set_header_title(slint::SharedString::from("SEND CRYPTO"));
    ui.set_subheader_title(slint::SharedString::from("NFC is active"));
    ui.set_protocol_image_shown(true);
    ui.set_current_protocol(Protocol::NFC);

    let buttons = ModelRc::new(VecModel::from(vec![
        ScreenButton { 
            text: "Switch to Bluetooth".into(), 
            width: 320.0, 
            height: 40.0, 
            x: 0.0, 
            has_border: false,
            inverted: false,
            y: 190.0,
        },
    ]));
    ui.set_buttons(buttons);

    // Initialize the protocol on screen creation
    #[cfg(feature = "minifb")]
    {
        init_socket_protocol();
    }
    
    #[cfg(feature = "zephyr")]
    {
        init_nfc_protocol();
    }

    let ui_weak = Rc::downgrade(ui);
    ui.on_pressed(move |item| {
        let Some(ui) = ui_weak.upgrade() else { return };
        
        if item.text == "Switch to Bluetooth" {
            ui.set_current_protocol(Protocol::Bluetooth);
            ui.set_subheader_title(slint::SharedString::from("Bluetooth is active"));
            
            let buttons = ModelRc::new(VecModel::from(vec![
                ScreenButton { 
                    text: "Switch to NFC".into(), 
                    width: 320.0, 
                    height: 40.0, 
                    x: 0.0, 
                    has_border: false,
                    inverted: false,
                    y: 190.0,
                },
            ]));
            ui.set_buttons(buttons);
            
            // Mark protocol change in state (will be handled in loop)
            state().lock().mark_protocol_requested();
        }
        
        if item.text == "Switch to NFC" {
            ui.set_current_protocol(Protocol::NFC);
            ui.set_subheader_title(slint::SharedString::from("NFC is active"));
            
            let buttons = ModelRc::new(VecModel::from(vec![
                ScreenButton { 
                    text: "Switch to Bluetooth".into(), 
                    width: 320.0, 
                    height: 40.0, 
                    x: 0.0, 
                    has_border: false,
                    inverted: false,
                    y: 190.0,
                },
            ]));
            ui.set_buttons(buttons);
            
            // Mark protocol change in state (will be handled in loop)
            state().lock().mark_protocol_requested();
        }
    });
}

// ============================================================================
// Protocol initialization functions
// ============================================================================

#[cfg(feature = "minifb")]
fn init_socket_protocol() {
    unsafe {
        if SOCKET_PROTOCOL_HANDLER.is_none() {
            let mut socket = SocketProtocol::new();
            match socket.init() {
                Ok(_) => {
                    SOCKET_PROTOCOL_HANDLER = Some(socket);
                    log_info!("Socket protocol initialized for Send screen");
                },
                Err(e) => {
                    log_info!("Failed to initialize socket protocol: {:?}", e);
                }
            }
        }
    }
}

#[cfg(feature = "minifb")]
fn close_socket_protocol() {
    unsafe {
        if let Some(mut socket) = SOCKET_PROTOCOL_HANDLER.take() {
            socket.close();
            log_info!("Socket protocol closed");
        }
    }
}

#[cfg(feature = "zephyr")]
fn init_nfc_protocol() {
    unsafe {
        // Stop BLE if running
        if let Some(ref ble) = BLE_HANDLER {
            ble.stop();
        }
        BLE_HANDLER = None;
        
        // Initialize NFC
        let nfc = NFCProtocol::new();
        let address = firmware().vault.lock().get_stellar_address().unwrap_or_default();
        let nfc_message = format!("app.hito.dev/#eth/send/!from={}", address);
        if nfc.init(&nfc_message).is_ok() {
            NFC_HANDLER = Some(nfc);
            log_info!("NFC protocol initialized");
        } else {
            log_info!("Failed to initialize NFC protocol");
        }
    }
}

#[cfg(feature = "zephyr")]
fn init_ble_protocol() {
    unsafe {
        // Stop NFC if running
        if let Some(ref nfc) = NFC_HANDLER {
            nfc.stop();
        }
        NFC_HANDLER = None;
        
        // Initialize BLE
        let ble = BLEProtocol::new();
        if ble.init().is_ok() {
            BLE_HANDLER = Some(ble);
            log_info!("BLE protocol initialized");
        } else {
            log_info!("Failed to initialize BLE protocol");
        }
    }
}

#[cfg(feature = "zephyr")]
fn stop_all_protocols() {
    unsafe {
        if let Some(ref nfc) = NFC_HANDLER {
            nfc.stop();
        }
        NFC_HANDLER = None;
        
        if let Some(ref ble) = BLE_HANDLER {
            ble.stop();
        }
        BLE_HANDLER = None;
    }
}

// ============================================================================
// Loop event handling - called from main loop
// ============================================================================

/// Handle Send screen loop events
/// This should be called from the main loop when on the Send screen
pub fn handle_send_screen_loop(ui: &MainWindow) {
    let s = state().lock();
    
    // Handle protocol change request
    if s.is_protocol_change_requested() {
        #[cfg(feature = "zephyr")]
        {
            let protocol = ui.get_current_protocol();
            match protocol {
                Protocol::NFC => {
                    drop(s); // Release lock before init
                    init_nfc_protocol();
                    let s = state().lock();
                    s.clear_protocol_change_requested();
                },
                Protocol::Bluetooth => {
                    drop(s); // Release lock before init
                    init_ble_protocol();
                    let s = state().lock();
                    s.clear_protocol_change_requested();
                }
            }
        }
        
        #[cfg(feature = "minifb")]
        {
            // For minifb, socket protocol doesn't change based on UI selection
            // It's always the same socket, so just clear the flag
            s.clear_protocol_change_requested();
        }
    } else {
        drop(s); // Release lock if no protocol change
    }
    
    // Handle incoming data based on platform and protocol
    #[cfg(feature = "minifb")]
    {
        handle_socket_data(ui);
    }
    
    #[cfg(feature = "zephyr")]
    {
        handle_zephyr_protocol_data(ui);
    }
}

#[cfg(feature = "minifb")]
fn handle_socket_data(ui: &MainWindow) {
    unsafe {
        if let Some(ref mut socket) = SOCKET_PROTOCOL_HANDLER {
            // Try to accept new connections
            let _ = socket.accept_connection();
            
            // Check for incoming data if connected
            if socket.is_connected() {
                match socket.receive() {
                    Ok(Some(data)) => {
                        RECEIVED_DATA = Some(data.clone());
                        parse_received_data(&data, ui);
                    },
                    Ok(None) => {
                        // No data available
                    },
                    Err(e) => {
                        log_info!("Socket receive error: {:?}", e);
                    }
                }
            }
        }
    }
}

#[cfg(feature = "zephyr")]
fn handle_zephyr_protocol_data(ui: &MainWindow) {
    let protocol = ui.get_current_protocol();
    
    unsafe {
        let data = match protocol {
            Protocol::NFC => {
                if let Some(ref nfc) = NFC_HANDLER {
                    nfc.get_data()
                } else {
                    None
                }
            },
            Protocol::Bluetooth => {
                if let Some(ref ble) = BLE_HANDLER {
                    ble.get_data()
                } else {
                    None
                }
            }
        };
        
        if let Some(data) = data {
            RECEIVED_DATA = Some(data.clone());
            parse_received_data(&data, ui);
        }
    }
}

/// Called when leaving the Send screen - cleanup protocols
pub fn cleanup_send_screen() {
    #[cfg(feature = "minifb")]
    {
        close_socket_protocol();
    }
    
    #[cfg(feature = "zephyr")]
    {
        stop_all_protocols();
    }
    
    unsafe {
        RECEIVED_DATA = None;
    }
}