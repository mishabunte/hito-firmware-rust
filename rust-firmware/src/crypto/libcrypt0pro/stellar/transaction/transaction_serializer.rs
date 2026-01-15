#![no_std]

extern crate alloc;
use alloc::{vec::Vec, string::String};
use base64ct::{Base64, Encoding};

use crate::crypto::crypt0::{bytes_to_hex, hex_to_bytes};
use crate::crypto::libcrypt0pro::stellar::StellarWallet;

use super::*;
use super::xdr::*;
use crate::log_info;

#[derive(Debug)]
pub struct StellarTransactionSerializer;

impl StellarTransactionSerializer {
    pub fn new() -> Self {
        Self {
        }
    }

    /// Serialize ParsedTransaction directly to XDR bytes
    pub fn serialize_to_xdr_bytes(envelope: &TransactionEnvelope, buffer: &mut Vec<u8>) -> Result<Vec<u8>, StellarTransactionError> {
        match &envelope {
          TransactionEnvelope::Transaction(tx) => {
              match tx.envelope_type {
                  TransactionEnvelopeType::TxV0 => {
                      Self::write_u32(ENVELOPE_TYPE_TX_V0 as u32, buffer)?; // ENVELOPE_TYPE_TX_V0
                      Self::serialize_v0_transaction(tx, buffer)?;
                  }
                  TransactionEnvelopeType::Tx => {
                      Self::write_u32(ENVELOPE_TYPE_TX as u32, buffer)?; // ENVELOPE_TYPE_TX
                      Self::serialize_v1_transaction(tx, buffer)?;
                  }
                  TransactionEnvelopeType::TxFeeBump => {
                      return Err(StellarTransactionError::InvalidEnvelopeType);
                  }
              }
          },
          TransactionEnvelope::FeeBump(fee_bump) => {
              Self::write_u32(ENVELOPE_TYPE_TX_FEE_BUMP as u32, buffer)?; // ENVELOPE_TYPE_TX_FEE_BUMP
              Self::serialize_fee_bump_transaction(fee_bump, buffer)?;
          }
        }
        
        Ok(buffer.clone())
    }

    /// Serialize to base64-encoded XDR string
    pub fn serialize_to_base64(parsed_tx: &TransactionEnvelope) -> Result<String, StellarTransactionError> {
        let mut buffer = Vec::with_capacity(MAX_XDR_LEN);
        let xdr_bytes = Self::serialize_to_xdr_bytes(parsed_tx, &mut buffer)?;
        Self::encode_base64(&xdr_bytes)
    }

    pub fn build_signature_base(
        envelope: &TransactionEnvelope,
    ) -> Result<Vec<u8>, StellarTransactionError> {
        let mut buffer = Vec::with_capacity(MAX_XDR_LEN);

        // Ok(buffer)
        //let network_id_bytes = &parsed_tx.network_hash.clone().unwrap().get_hash_bytes();
        match &envelope {
          TransactionEnvelope::Transaction(tx) => {
              match tx.envelope_type {
                  TransactionEnvelopeType::TxFeeBump => {
                      return Err(StellarTransactionError::InvalidEnvelopeType);
                  }
                  _ => {
                    let network_id_bytes = &tx.network_hash.clone().unwrap().get_hash_bytes();
                    buffer.extend_from_slice(network_id_bytes);

                    match tx.envelope_type {
                        TransactionEnvelopeType::TxV0 | TransactionEnvelopeType::Tx => {
                            // IMPORTANT:
                            // Backwards compatibility rule:
                            // We ALWAYS use ENVELOPE_TYPE_TX (2) for signing both TxV0 and Tx.
                            Self::write_u32(ENVELOPE_TYPE_TX as u32, &mut buffer)?; // ENVELOPE_TYPE_TX

                            // Then XDR for Transaction (no signatures!)
                            Self::serialize_tx_core_for_signature(tx, &mut buffer)?;
                            Ok(buffer)
                        }
                        TransactionEnvelopeType::TxFeeBump => {
                          return Err(StellarTransactionError::InvalidEnvelopeType);
                        }
                    }
                  }
              }
          },
          TransactionEnvelope::FeeBump(fee_bump) => {
              // Fee bump transaction
              let network_id_bytes = &fee_bump.inner_tx.network_hash.clone().unwrap().get_hash_bytes();
              buffer.extend_from_slice(network_id_bytes);
              Self::write_u32(5, &mut buffer)?; // ENVELOPE_TYPE_FEE_BUMP
              Self::serialize_fee_bump_core_for_signature(fee_bump, &mut buffer)?;
              Ok(buffer)
          }
        }
    }

    fn serialize_tx_core_for_signature(
        parsed_tx: &Transaction,
        buffer: &mut Vec<u8>,
    ) -> Result<(), StellarTransactionError> {
        match parsed_tx.envelope_type {
            TransactionEnvelopeType::TxV0 => {
                // Equivalent to Transaction with AccountID source, old-school timebounds.
                Self::write_account_id(&parsed_tx.source_account, buffer)?;
                Self::write_u32(parsed_tx.fee as u32, buffer)?;
                Self::write_u64(parsed_tx.sequence_number as u64, buffer)?;

                if let Some(ref time_bounds) = parsed_tx.time_bounds {
                    Self::write_u32(1, buffer)?; // Present
                    Self::serialize_time_bounds(time_bounds, buffer)?;
                } else {
                    Self::write_u32(0, buffer)?; // Not present
                }

                Self::serialize_memo(parsed_tx.memo.as_ref(), buffer)?;
                Self::serialize_operations(&parsed_tx.operations, buffer)?;
                Self::write_u32(0, buffer)?; // ext = 0
            }
            TransactionEnvelopeType::Tx => {
                // V1-style transaction (muxed account + preconditions)
                Self::write_muxed_account(
                    &MuxedAccount::Ed25519 {
                        account_id: parsed_tx.source_account.clone(),
                    },
                    buffer,
                )?;

                Self::write_u32(parsed_tx.fee as u32, buffer)?;
                Self::write_u64(parsed_tx.sequence_number as u64, buffer)?;

                Self::serialize_preconditions(
                    &parsed_tx.time_bounds,
                    &parsed_tx.ledger_bounds,
                    buffer,
                )?;

                Self::serialize_memo(parsed_tx.memo.as_ref(), buffer)?;
                Self::serialize_operations(&parsed_tx.operations, buffer)?;
                Self::write_u32(0, buffer)?; // ext = 0
            }
            TransactionEnvelopeType::TxFeeBump => {
                // Should not happen, handled by serialize_fee_bump_core_for_signature.
                return Err(StellarTransactionError::XdrError);
            }
        }

        Ok(())
    }

    fn serialize_fee_bump_core_for_signature(
        fee_bump: &FeeBumpTransaction,
        buffer: &mut Vec<u8>,
    ) -> Result<(), StellarTransactionError> {
        // Fee source (muxed)
        Self::write_muxed_account(
            &fee_bump.fee_source,
            buffer,
        )?;

        // Fee (int64)
        Self::write_i64(fee_bump.fee as i64, buffer)?;

        // Inner transaction type (currently always ENVELOPE_TYPE_TX = 2)
        Self::write_u32(2, buffer)?;

        // Inner transaction envelope (v1) WITHOUT the outer (fee bump) signatures.
        //
        // We reuse the same core layout as serialize_v1_transaction but skip
        // writing ParsedTransaction.signatures at the end.
        //
        //   struct TransactionV1Envelope {
        //     TransactionV1 tx;
        //     DecoratedSignature signatures<20>; // inner signatures (not fee bump)
        //   };
        //
        // Our ParsedTransaction currently does not distinguish between inner and
        // outer signatures, so if/when you add that, you'll want to adjust this
        // part to write the correct inner envelope.
        Self::serialize_v1_transaction(&fee_bump.inner_tx, buffer)?;

        // FeeBumpTransaction ext = 0
        Self::write_u32(0, buffer)?;

        Ok(())
    }

    pub fn compute_signature_hint(account_hex: &String) -> [u8; 4] {
        let pk_bytes = hex_to_bytes(account_hex).unwrap();
        let len = pk_bytes.len();
        let start = len.saturating_sub(4);
        let mut hint = [0u8; 4];
        hint.copy_from_slice(&pk_bytes[start..len]);
        hint
    }

    fn serialize_v0_transaction(parsed_tx: &Transaction, buffer: &mut Vec<u8>) -> Result<(), StellarTransactionError> {
        log_info!("Serializing V0 transaction\n");
        
        // Source account (32 bytes)
        let source_account = StellarWallet::decode_stellar_address(&parsed_tx.source_account);
        if source_account.is_err() {
            return Err(StellarTransactionError::AccountIdError);
        }
        let source_account = source_account.unwrap();
        Self::write_account_id(&bytes_to_hex(&source_account), buffer)?;
        
        // Fee (4 bytes)
        Self::write_u32(parsed_tx.fee as u32, buffer)?;
        
        // Sequence number (8 bytes)
        Self::write_u64(parsed_tx.sequence_number as u64, buffer)?;
        
        // Time bounds (optional)
        if let Some(ref time_bounds) = parsed_tx.time_bounds {
            Self::write_u32(1, buffer)?; // Present
            Self::serialize_time_bounds(time_bounds, buffer)?;
        } else {
            Self::write_u32(0, buffer)?; // Not present
        }
        
        // Memo
        Self::serialize_memo(parsed_tx.memo.as_ref(), buffer)?;
        
        // Operations
        Self::serialize_operations(&parsed_tx.operations, buffer)?;
        
        // Extension (V0 = 0)
        Self::write_u32(0, buffer)?;
        
        // Signatures
        Self::serialize_signatures(&parsed_tx.signatures, buffer)?;
        
        Ok(())
    }

    fn serialize_v1_transaction(parsed_tx: &Transaction, buffer: &mut Vec<u8>) -> Result<(), StellarTransactionError> {
        log_info!("Serializing V1 transaction\n");
        
        // Source account (muxed)
        Self::write_muxed_account(&MuxedAccount::Ed25519 {
            account_id: parsed_tx.source_account.clone()
        }, buffer)?;
        
        // Fee (4 bytes)
        Self::write_u32(parsed_tx.fee as u32, buffer)?;
        
        // Sequence number (8 bytes)
        Self::write_u64(parsed_tx.sequence_number as u64, buffer)?;
        
        // Preconditions
        Self::serialize_preconditions(&parsed_tx.time_bounds, &parsed_tx.ledger_bounds, buffer)?;
        
        // Memo
        Self::serialize_memo(parsed_tx.memo.as_ref(), buffer)?;
        
        // Operations
        Self::serialize_operations(&parsed_tx.operations, buffer)?;
        
        // Extension (V0 = 0)
        Self::write_u32(0, buffer)?;
        
        // Signatures
        Self::serialize_signatures(&parsed_tx.signatures, buffer)?;
        
        Ok(())
    }

    fn serialize_fee_bump_transaction(fee_bump: &FeeBumpTransaction, buffer: &mut Vec<u8>) -> Result<(), StellarTransactionError> {
        log_info!("Serializing Fee Bump transaction\n");
        
        // Fee source (muxed account)
        Self::write_muxed_account(&fee_bump.fee_source, buffer)?;
        
        // Fee
        Self::write_u64(fee_bump.fee as u64, buffer)?;
        
        // Inner transaction type (always TX = 2)
        Self::write_u32(2, buffer)?;
        
        // Inner transaction (serialize as V1)
        Self::serialize_v1_transaction(&fee_bump.inner_tx, buffer)?;
        
        // Extension (V0 = 0)
        Self::write_u32(0, buffer)?;
        
        // Fee bump signatures (replace V1 signatures)
        Self::serialize_signatures(&fee_bump.signatures, buffer)?;
        
        Ok(())
    }

    fn serialize_memo(memo: Option<&Memo>, buffer: &mut Vec<u8>) -> Result<(), StellarTransactionError> {
        match memo {
            None => {
                Self::write_u32(0, buffer)?; // MEMO_NONE
            },
            Some(memo) => {
                match memo.memo_type.as_str() {
                    "none" => {
                        Self::write_u32(0, buffer)?; // MEMO_NONE
                    },
                    "text" => {
                        Self::write_u32(1, buffer)?; // MEMO_TEXT
                        if let Some(ref text) = memo.value {
                            Self::write_string(text, buffer)?;
                        } else {
                            Self::write_u32(0, buffer)?; // Empty string
                        }
                    },
                    "id" => {
                        Self::write_u32(2, buffer)?; // MEMO_ID
                        if let Some(ref id_str) = memo.value {
                            let id = Self::parse_u64_from_string(id_str)?;
                            Self::write_u64(id, buffer)?;
                        } else {
                            Self::write_u64(0, buffer)?;
                        }
                    },
                    "hash" => {
                        Self::write_u32(3, buffer)?; // MEMO_HASH
                        if let Some(ref hash_str) = memo.value {
                            let hash_bytes = hex_to_bytes(hash_str).unwrap();
                            Self::write_fixed_bytes(&hash_bytes.as_slice(), 32, buffer)?;
                        } else {
                            Self::write_fixed_bytes(&[0u8; 32], 32, buffer)?;
                        }
                    },
                    "return" => {
                        Self::write_u32(4, buffer)?; // MEMO_RETURN
                        if let Some(ref return_str) = memo.value {
                            let return_bytes = hex_to_bytes(return_str).unwrap();
                            Self::write_fixed_bytes(&return_bytes.as_slice(), 32, buffer)?;
                        } else {
                            Self::write_fixed_bytes(&[0u8; 32], 32, buffer)?;
                        }
                    },
                    _ => return Err(StellarTransactionError::XdrError),
                }
            }
        }
        Ok(())
    }

    fn serialize_preconditions(
        time_bounds: &Option<TimeBounds>,
        ledger_bounds: &Option<LedgerBounds>,
        buffer: &mut Vec<u8>
    ) -> Result<(), StellarTransactionError> {
        match (time_bounds, ledger_bounds) {
            (None, None) => {
                Self::write_u32(0, buffer)?; // PRECOND_NONE
            },
            (Some(tb), None) => {
                Self::write_u32(1, buffer)?; // PRECOND_TIME
                Self::serialize_time_bounds(tb, buffer)?;
            },
            (time_bounds, ledger_bounds) => {
                Self::write_u32(2, buffer)?; // PRECOND_V2
                
                // Time bounds (optional)
                if let Some(tb) = time_bounds {
                    Self::write_u32(1, buffer)?; // Present
                    Self::serialize_time_bounds(tb, buffer)?;
                } else {
                    Self::write_u32(0, buffer)?; // Not present
                }
                
                // Ledger bounds (optional)
                if let Some(lb) = ledger_bounds {
                    Self::write_u32(1, buffer)?; // Present
                    Self::serialize_ledger_bounds(lb, buffer)?;
                } else {
                    Self::write_u32(0, buffer)?; // Not present
                }
                
                // Min seq num (optional) - not present
                Self::write_u32(0, buffer)?;
                
                // Min seq age
                Self::write_u64(0, buffer)?;
                
                // Min seq ledger gap
                Self::write_u32(0, buffer)?;
                
                // Extra signers (empty array)
                Self::write_u32(0, buffer)?;
            }
        }
        Ok(())
    }

    fn serialize_time_bounds(time_bounds: &TimeBounds, buffer: &mut Vec<u8>) -> Result<(), StellarTransactionError> {
        Self::write_u64(time_bounds.min_time.unwrap_or(0), buffer)?;
        Self::write_u64(time_bounds.max_time.unwrap_or(0), buffer)?;
        Ok(())
    }

    fn serialize_ledger_bounds(ledger_bounds: &LedgerBounds, buffer: &mut Vec<u8>) -> Result<(), StellarTransactionError> {
        Self::write_u32(ledger_bounds.min_ledger, buffer)?;
        Self::write_u32(ledger_bounds.max_ledger, buffer)?;
        Ok(())
    }

    fn serialize_operations(operations: &[Operation], buffer: &mut Vec<u8>) -> Result<(), StellarTransactionError> {
        // Write operation count
        Self::write_u32(operations.len() as u32, buffer)?;
        
        for operation in operations.iter() {
            Self::serialize_operation(operation, buffer)?;
        }
        
        Ok(())
    }

    fn serialize_operation(operation: &Operation, buffer: &mut Vec<u8>) -> Result<(), StellarTransactionError> {
        // Source account (optional)
        if let Some(ref source) = operation.source_account {
            Self::write_u32(1, buffer)?; // Present
            Self::write_muxed_account(&MuxedAccount::Ed25519 {
                account_id: source.clone()
            }, buffer)?;
        } else {
            Self::write_u32(0, buffer)?; // Not present
        }
        
        // Operation body
        match &operation.details {
            OperationDetails::CreateAccount { destination, starting_balance } => {
                Self::write_i32(0, buffer)?; // CREATE_ACCOUNT
                Self::write_account_id(destination, buffer)?;
                Self::write_i64(*starting_balance, buffer)?;
            },
            OperationDetails::Payment { destination, asset, amount } => {
                Self::write_u32(1, buffer)?; // PAYMENT
                Self::write_muxed_account(destination, buffer)?;
                Self::serialize_asset(asset, buffer)?;
                Self::write_i64(*amount, buffer)?;
            },
            OperationDetails::PathPaymentStrictReceive { send_asset, send_max, destination, dest_asset, dest_amount, path } => {
                Self::write_u32(2, buffer)?; // PATH_PAYMENT_STRICT_RECEIVE
                Self::serialize_asset(send_asset, buffer)?;
                Self::write_i64(*send_max, buffer)?;
                Self::write_muxed_account(destination, buffer)?;
                Self::serialize_asset(dest_asset, buffer)?;
                Self::write_i64(*dest_amount, buffer)?;
                Self::serialize_asset_path(path, buffer)?;
            },
            OperationDetails::PathPaymentStrictSend { send_asset, send_amount, destination, dest_asset, dest_min, path } => {
                Self::write_u32(13, buffer)?; // PATH_PAYMENT_STRICT_SEND
                Self::serialize_asset(send_asset, buffer)?;
                Self::write_i64(*send_amount, buffer)?;
                Self::write_muxed_account(destination, buffer)?;
                Self::serialize_asset(dest_asset, buffer)?;
                Self::write_i64(*dest_min, buffer)?;
                Self::serialize_asset_path(path, buffer)?;
            },
            OperationDetails::ChangeTrust { asset, limit } => {
                Self::write_u32(6, buffer)?; // CHANGE_TRUST
                Self::serialize_change_trust_asset(asset, buffer)?;
                Self::write_i64(*limit, buffer)?;
            },
            OperationDetails::AllowTrust { trustor, asset_code, authorize } => {
                Self::write_u32(7, buffer)?; // ALLOW_TRUST
                Self::write_account_id(trustor, buffer)?;
                Self::serialize_asset_code(asset_code, buffer)?;
                Self::write_u32(*authorize, buffer)?;
            },
            OperationDetails::AccountMerge { destination } => {
                Self::write_u32(8, buffer)?; // ACCOUNT_MERGE
                Self::write_muxed_account(&MuxedAccount::Ed25519 {
                    account_id: destination.clone()
                }, buffer)?;
            },
            OperationDetails::SetTrustLineFlags { trustor, asset, clear_flags, set_flags } => {
                Self::write_u32(21, buffer)?; // SET_TRUST_LINE_FLAGS
                Self::write_account_id(trustor, buffer)?;
                Self::serialize_asset(asset, buffer)?;
                Self::write_u32(*clear_flags, buffer)?;
                Self::write_u32(*set_flags, buffer)?;
            },
            OperationDetails::Other { .. } => {
                return Err(StellarTransactionError::UnsupportedOperation);
            }
        }
        
        Ok(())
    }

    fn serialize_asset(asset: &Asset, buffer: &mut Vec<u8>) -> Result<(), StellarTransactionError> {
        match asset {
            Asset::Native => {
                Self::write_u32(0, buffer)?; // ASSET_TYPE_NATIVE
            },
            Asset::CreditAlphanum4 { code, issuer } => {
                Self::write_u32(1, buffer)?; // ASSET_TYPE_CREDIT_ALPHANUM4
                Self::write_asset_code4(code, buffer)?;
                Self::write_account_id(issuer, buffer)?;
            },
            Asset::CreditAlphanum12 { code, issuer } => {
                Self::write_u32(2, buffer)?; // ASSET_TYPE_CREDIT_ALPHANUM12
                Self::write_asset_code12(code, buffer)?;
                Self::write_account_id(issuer, buffer)?;
            }
        }
        Ok(())
    }

    fn serialize_change_trust_asset(asset: &ChangeTrustAsset, buffer: &mut Vec<u8>) -> Result<(), StellarTransactionError> {
        match asset {
            ChangeTrustAsset::Native => {
                Self::write_u32(0, buffer)?; // ASSET_TYPE_NATIVE
            },
            ChangeTrustAsset::CreditAlphanum4 { code, issuer } => {
                Self::write_u32(1, buffer)?; // ASSET_TYPE_CREDIT_ALPHANUM4
                Self::write_asset_code4(code, buffer)?;
                Self::write_account_id(issuer, buffer)?;
            },
            ChangeTrustAsset::CreditAlphanum12 { code, issuer } => {
                Self::write_u32(2, buffer)?; // ASSET_TYPE_CREDIT_ALPHANUM12
                Self::write_asset_code12(code, buffer)?;
                Self::write_account_id(issuer, buffer)?;
            },
            ChangeTrustAsset::LiquidityPool { asset_a, asset_b, fee } => {
                Self::write_u32(3, buffer)?; // ASSET_TYPE_POOL_SHARE
                Self::write_u32(0, buffer)?; // LIQUIDITY_POOL_CONSTANT_PRODUCT
                Self::serialize_asset(asset_a, buffer)?;
                Self::serialize_asset(asset_b, buffer)?;
                Self::write_u32(*fee as u32, buffer)?;
            }
        }
        Ok(())
    }

    fn serialize_asset_path(path: &[Asset], buffer: &mut Vec<u8>) -> Result<(), StellarTransactionError> {
        Self::write_u32(path.len() as u32, buffer)?;
        for asset in path.iter() {
            Self::serialize_asset(asset, buffer)?;
        }
        Ok(())
    }

    fn serialize_asset_code(code: &String, buffer: &mut Vec<u8>) -> Result<(), StellarTransactionError> {
        if code.len() <= 4 {
            Self::write_u32(1, buffer)?; // ASSET_TYPE_CREDIT_ALPHANUM4
            Self::write_asset_code4(code, buffer)?;
        } else {
            Self::write_u32(2, buffer)?; // ASSET_TYPE_CREDIT_ALPHANUM12
            Self::write_asset_code12(code, buffer)?;
        }
        Ok(())
    }

    fn serialize_signatures(signatures: &[Signature], buffer: &mut Vec<u8>) -> Result<(), StellarTransactionError> {
        Self::write_u32(signatures.len() as u32, buffer)?;
        
        for signature in signatures.iter() {
            // Signature hint (4 bytes)
            let signature_hint_hex = hex_to_bytes(&signature.hint);
            if signature_hint_hex.is_err() {
                return Err(StellarTransactionError::XdrError);
            }
            let signature_hint_hex = signature_hint_hex.unwrap();
            Self::write_fixed_bytes(&signature_hint_hex, 4, buffer)?;
            
            // Signature (variable length)
            let signature_bytes = hex_to_bytes(&signature.signature);
            if signature_bytes.is_err() {
                return Err(StellarTransactionError::XdrError);
            }
            let signature_bytes = signature_bytes.unwrap();
            Self::write_opaque_bytes(&signature_bytes, buffer)?;
        }
        
        Ok(())
    }

    // Low-level writing functions
    fn write_u32(value: u32, buffer: &mut Vec<u8>) -> Result<(), StellarTransactionError> {
        let bytes = value.to_be_bytes();
        buffer.extend_from_slice(&bytes);
        Ok(())
    }

    fn write_u64(value: u64, buffer: &mut Vec<u8>) -> Result<(), StellarTransactionError> {
        let bytes = value.to_be_bytes();
        buffer.extend_from_slice(&bytes);
        Ok(())
    }

    fn write_i64(value: i64, buffer: &mut Vec<u8>) -> Result<(), StellarTransactionError> {
        let bytes = value.to_be_bytes();
        buffer.extend_from_slice(&bytes);
        Ok(())
    }

    fn write_i32(value: i32, buffer: &mut Vec<u8>) -> Result<(), StellarTransactionError> {
        let bytes = value.to_be_bytes();
        buffer.extend_from_slice(&bytes);
        Ok(())
    }

    fn write_fixed_bytes(bytes: &[u8], expected_len: usize, buffer: &mut Vec<u8>) -> Result<(), StellarTransactionError> {
        if bytes.len() > expected_len {
            return Err(StellarTransactionError::XdrError);
        }
        
        buffer.extend_from_slice(bytes);
        
        // Pad with zeros if needed
        for _ in bytes.len()..expected_len {
            buffer.push(0);
        }
        
        Ok(())
    }

    fn write_opaque_bytes(bytes: &[u8], buffer: &mut Vec<u8>) -> Result<(), StellarTransactionError> {
        // Write length
        Self::write_u32(bytes.len() as u32, buffer)?;
        
        // Write data
        buffer.extend_from_slice(bytes);
        
        // Add padding to 4-byte boundary
        let padding = (4 - (bytes.len() % 4)) % 4;
        for _ in 0..padding {
            buffer.push(0);
        }
        
        Ok(())
    }

    fn write_string(s: &String, buffer: &mut Vec<u8>) -> Result<(), StellarTransactionError> {
        let bytes = s.as_bytes();
        Self::write_opaque_bytes(bytes, buffer)
    }

    fn write_account_id(account_hex: &String, buffer: &mut Vec<u8>) -> Result<(), StellarTransactionError> {
        Self::write_u32(0, buffer)?;
        let account_bytes = StellarWallet::decode_stellar_address(account_hex);
        if account_bytes.is_err() {
            return Err(StellarTransactionError::AccountIdError);
        }
        let bytes = account_bytes.unwrap();
        Self::write_fixed_bytes(&bytes.as_slice(), 32, buffer)
    }

    fn write_muxed_account(account: &MuxedAccount, buffer: &mut Vec<u8>) -> Result<(), StellarTransactionError> {
        match account {
            MuxedAccount::Ed25519 { account_id } => {
                Self::write_u32(0, buffer)?; // KEY_TYPE_ED25519
                let account_bytes = StellarWallet::decode_stellar_address(account_id);
                if account_bytes.is_err() {
                    return Err(StellarTransactionError::MuxedAccountError);
                }
                let bytes = account_bytes.unwrap();
                Self::write_fixed_bytes(&bytes.as_slice(), 32, buffer)?;
            },
            MuxedAccount::MuxedEd25519 { id, account_id } => {
                Self::write_u32(256, buffer)?; // KEY_TYPE_MUXED_ED25519
                Self::write_u64(*id, buffer)?;
                
                let account_bytes = StellarWallet::decode_stellar_address(account_id);
                if account_bytes.is_err() {
                    return Err(StellarTransactionError::MuxedAccountError);
                }
                let bytes = account_bytes.unwrap();
                Self::write_fixed_bytes(&bytes.as_slice(), 32, buffer)?;
            }
        }
        Ok(())
    }

    fn write_asset_code4(code: &String, buffer: &mut Vec<u8>) -> Result<(), StellarTransactionError> {
        let mut code_bytes = [0u8; 4];
        let bytes = code.as_bytes();
        let len = bytes.len().min(4);
        code_bytes[..len].copy_from_slice(&bytes[..len]);
        buffer.extend_from_slice(&code_bytes);
        Ok(())
    }

    fn write_asset_code12(code: &String, buffer: &mut Vec<u8>) -> Result<(), StellarTransactionError> {
        let mut code_bytes = [0u8; 12];
        let bytes = code.as_bytes();
        let len = bytes.len().min(12);
        code_bytes[..len].copy_from_slice(&bytes[..len]);
        buffer.extend_from_slice(&code_bytes);
        Ok(())
    }

    // Helper functions
    fn parse_u64_from_string(s: &String) -> Result<u64, StellarTransactionError> {
        let bytes = s.as_bytes();
        let mut result = 0u64;
        
        for &byte in bytes {
            match byte {
                b'0'..=b'9' => {
                    let digit = (byte - b'0') as u64;
                    result = result.checked_mul(10)
                        .and_then(|r| r.checked_add(digit))
                        .ok_or(StellarTransactionError::XdrError)?;
                },
                _ => return Err(StellarTransactionError::XdrError),
            }
        }
        
        Ok(result)
    }

    fn hex_char_to_u8(c: u8) -> Result<u8, StellarTransactionError> {
        match c {
            b'0'..=b'9' => Ok(c - b'0'),
            b'a'..=b'f' => Ok(c - b'a' + 10),
            b'A'..=b'F' => Ok(c - b'A' + 10),
            _ => Err(StellarTransactionError::XdrError),
        }
    }

    fn encode_base64(data: &[u8]) -> Result<String, StellarTransactionError> {
        let mut buf = [0u8; MAX_XDR_LEN * 4 / 3 + 4]; // Base64 expansion ratio
        let encoded = Base64::encode(data, &mut buf)
            .map_err(|_| StellarTransactionError::Base64Error)?;
        Ok(String::from(encoded))
    }
}

impl Default for StellarTransactionSerializer {
    fn default() -> Self {
        Self::new()
    }
}