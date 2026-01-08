use crate::crypto::crypt0::bytes_to_hex;
use crate::crypto::libcrypt0pro::stellar::{ParsedSignature, ParsedTransaction, StellarKeypair, StellarTransactionSerializer, TransactionEnvelope};

use crate::crypto::ffi::crypt0_ed25519_sign;
use crate::log_info;

pub struct StellarTransactionSigner;


#[derive(Debug, PartialEq, Clone)]
pub enum StellarSignError {
    InvalidTransaction,
    SigningFailed,
    ErrorDerivingKeypair
}



impl StellarTransactionSigner {
    pub fn sign_transaction(envelope: &TransactionEnvelope, keypair: StellarKeypair) -> Result<TransactionEnvelope, StellarSignError> {
      // Build the signature base
      let signature_base = StellarTransactionSerializer::build_signature_base(envelope);
      if let Err(_) = signature_base {
          return Err(StellarSignError::InvalidTransaction);
      }
      let signature_base = signature_base.unwrap();
      log_info!("Signature base: {}", crate::crypto::crypt0::bytes_to_hex(&signature_base));
      let sha256_sig_base = unsafe {
          let mut hash = [0u8; 32];
          let res = crate::crypto::ffi::crypt0_sha256(
              signature_base.as_slice().as_ptr(),
              signature_base.as_slice().len(),
              hash.as_mut_ptr(),
              hash.len()
          );
          if !res {
              panic!("crypt0_sha256 failed");
          }
          hash
      };
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
      let mut envelope = envelope.clone();
      match &mut envelope {
          TransactionEnvelope::Transaction(tx_envelope) => {
              tx_envelope.signatures.push(ParsedSignature {
              hint: {
                let mut hint = [0u8; 4];
                hint.copy_from_slice(&keypair.public_key[28..32]);
                bytes_to_hex(&hint)
              },
              signature:bytes_to_hex(&signature),
              }
            );
          },
          TransactionEnvelope::FeeBump(fee_bump_envelope) => {
              fee_bump_envelope.signatures.push(ParsedSignature {
              hint: {
                let mut hint = [0u8; 4];
                hint.copy_from_slice(&keypair.public_key[28..32]);
                bytes_to_hex(&hint)
              },
              signature:bytes_to_hex(&signature),
              }
            );  
          },
      }
      Ok(envelope)
    }
}