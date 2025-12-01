use crate::drivers::{Display};
use crate::log_info;
use crate::{STATE, ui::CallbackController, hito_firmware::HitoFirmware};
use crate::slint_generatedMainWindow::SignedState;
use crate::slint_generatedMainWindow::BrightnessController;
use crate::slint_generatedMainWindow::MainWindow;
use slint::{ComponentHandle};
use crate::slint_generatedMainWindow::Router;
use crate::slint_generatedMainWindow::ScreenEnum;
use crate::crypto::libcrypt0pro::stellar::{StellarTransactionParser, StellarTransactionSerializer};
use crate::crypto::ffi::crypt0_ed25519_sign;
use alloc::format;

pub struct SignedDataCallbackController;

const NETWORK_ID: &str = "Test SDF Network ; September 2015";

impl CallbackController for SignedDataCallbackController {
    fn register_main_window_callbacks(&self, ui: &MainWindow, firmware: &mut HitoFirmware) {
      ui.global::<SignedState>().on_request_data(move || {
          //log_info!("Signed data request callback triggered");
          let s = STATE.get().unwrap().lock();
          //log_info!("Signed data requested");
          s.mark_qr_data_requested();
      });
      ui.global::<BrightnessController>().on_scale_shown_changed(move || {
          let s = STATE.get().unwrap().lock();
          s.mark_scale_shown_changed();
      });
    }
    fn handle_loop_events(&self, ui: &MainWindow, firmware: &mut HitoFirmware) {
      let s = STATE.get().unwrap().lock();
      let router = ui.global::<Router>();

      if s.is_qr_data_requested() || s.is_scale_shown_changed() {
        if router.get_current() == ScreenEnum::Signed {
          let parsed_tx = s.get_parsed_tx().unwrap();
          let base = StellarTransactionSerializer::build_signature_base(&parsed_tx, NETWORK_ID).unwrap();
          let sha256_sig_base = unsafe {
              let mut hash = [0u8; 32];
              let res = crate::crypto::ffi::crypt0_sha256(
                  base.as_slice().as_ptr(),
                  base.as_slice().len(),
                  hash.as_mut_ptr(),
                  hash.len()
              );
              if !res {
                  panic!("crypt0_sha256 failed");
              }
              hash
          };
          let keypair = firmware.vault.get_stellar_keypair().unwrap();
          let signature = unsafe {
              let mut sig = [0u8; 64];
              let privkey_bytes = keypair.secret_key;
              let pubkey_bytes = keypair.public_key;
              let res = crypt0_ed25519_sign(
                  sha256_sig_base.as_ptr(),
                  sha256_sig_base.len(),
                  privkey_bytes.as_ptr(),
                  privkey_bytes.len(),
                  pubkey_bytes.as_ptr(),
                  pubkey_bytes.len(),
                  sig.as_mut_ptr(),
                  sig.len()
              );
              if res != 0 {
                  panic!("crypt0_ed25519_sign failed");
              }
              sig
          };
          let mut parsed_tx = parsed_tx.clone();
          parsed_tx.signatures.push(crate::crypto::libcrypt0pro::stellar::ParsedSignature {
              hint: {
                  let mut hint = [0u8; 4];
                  hint.copy_from_slice(&keypair.public_key[28..32]);
                  hint
              },
              signature,
          });
          let qr_data = format!("https://app.hito.dev/eth/tx/#!{}", StellarTransactionSerializer::serialize_to_base64(&parsed_tx).unwrap());
          firmware.display.draw_qr(95, 50, &qr_data);
          s.mark_qr_data_success();
          s.clear_scale_shown_changed();
        }
      }
    }
}