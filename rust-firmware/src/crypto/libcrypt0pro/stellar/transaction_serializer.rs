#![no_std]

extern crate alloc;
use alloc::{vec::Vec, string::String};
use base64ct::{Base64, Encoding};

use crate::crypto::libcrypt0pro::stellar::{
    ParsedTransaction, ParsedMemo, ParsedTimeBounds, ParsedLedgerBounds,
    ParsedOperation, ParsedMuxedAccount, OperationDetails, ParsedAsset,
    ParsedChangeTrustAsset, ParsedSignature, TransactionEnvelopeType,
    TransactionParseError, BoundedString, MAX_XDR_LEN
};

pub enum TransactionSerializeError {
    Base64Error,
    XdrError,
    InvalidEnvelopeType,
    UnsupportedOperation,
    StringTooLong,
    TooManyOperations,
    TooManySignatures,
    DataTooLarge,
    AccountIdError,
    MuxedAccountError,
}

impl core::fmt::Display for TransactionSerializeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            TransactionSerializeError::Base64Error => write!(f, "Base64 decode error"),
            TransactionSerializeError::XdrError => write!(f, "XDR parsing error"),
            TransactionSerializeError::InvalidEnvelopeType => write!(f, "Invalid transaction envelope type"),
            TransactionSerializeError::UnsupportedOperation => write!(f, "Unsupported operation type"),
            TransactionSerializeError::StringTooLong => write!(f, "String too long for bounded container"),
            TransactionSerializeError::TooManyOperations => write!(f, "Too many operations in transaction"),
            TransactionSerializeError::TooManySignatures => write!(f, "Too many signatures in transaction"),
            TransactionSerializeError::DataTooLarge => write!(f, "Data too large for bounded container"),
            TransactionSerializeError::AccountIdError => write!(f, "Failed to parse account ID"),
            TransactionSerializeError::MuxedAccountError => write!(f, "Failed to parse muxed account"),
        }
    }
}

use crate::printk;

#[derive(Debug)]
pub struct StellarTransactionSerializer {
    buffer: Vec<u8>,
}

impl StellarTransactionSerializer {
    pub fn new() -> Self {
        Self {
            buffer: Vec::with_capacity(MAX_XDR_LEN),
        }
    }

    /// Serialize ParsedTransaction directly to XDR bytes
    pub fn serialize_to_xdr_bytes(&mut self, parsed_tx: &ParsedTransaction) -> Result<Vec<u8>, TransactionParseError> {
        self.buffer.clear();
        
        match parsed_tx.envelope_type {
            TransactionEnvelopeType::TxV0 => {
                self.write_u32(0)?; // ENVELOPE_TYPE_TX_V0
                self.serialize_v0_transaction(parsed_tx)?;
            },
            TransactionEnvelopeType::Tx => {
                self.write_u32(2)?; // ENVELOPE_TYPE_TX
                self.serialize_v1_transaction(parsed_tx)?;
            },
            TransactionEnvelopeType::TxFeeBump => {
                self.write_u32(5)?; // ENVELOPE_TYPE_TX_FEE_BUMP
                self.serialize_fee_bump_transaction(parsed_tx)?;
            }
        }
        
        Ok(self.buffer.clone())
    }

    /// Serialize to base64-encoded XDR string
    pub fn serialize_to_base64(&mut self, parsed_tx: &ParsedTransaction) -> Result<String, TransactionParseError> {
        let xdr_bytes = self.serialize_to_xdr_bytes(parsed_tx)?;
        self.encode_base64(&xdr_bytes)
    }

    fn serialize_v0_transaction(&mut self, parsed_tx: &ParsedTransaction) -> Result<(), TransactionParseError> {
        printk!("Serializing V0 transaction\n");
        
        // Source account (32 bytes)
        self.write_account_id(&parsed_tx.source_account)?;
        
        // Fee (4 bytes)
        self.write_u32(parsed_tx.fee as u32)?;
        
        // Sequence number (8 bytes)
        self.write_u64(parsed_tx.sequence_number as u64)?;
        
        // Time bounds (optional)
        if let Some(ref time_bounds) = parsed_tx.time_bounds {
            self.write_u32(1)?; // Present
            self.serialize_time_bounds(time_bounds)?;
        } else {
            self.write_u32(0)?; // Not present
        }
        
        // Memo
        self.serialize_memo(parsed_tx.memo.as_ref())?;
        
        // Operations
        self.serialize_operations(&parsed_tx.operations)?;
        
        // Extension (V0 = 0)
        self.write_u32(0)?;
        
        // Signatures
        self.serialize_signatures(&parsed_tx.signatures)?;
        
        Ok(())
    }

    fn serialize_v1_transaction(&mut self, parsed_tx: &ParsedTransaction) -> Result<(), TransactionParseError> {
        printk!("Serializing V1 transaction\n");
        
        // Source account (muxed)
        self.write_muxed_account(&ParsedMuxedAccount::Ed25519 {
            account_id: parsed_tx.source_account.clone()
        })?;
        
        // Fee (4 bytes)
        self.write_u32(parsed_tx.fee as u32)?;
        
        // Sequence number (8 bytes)
        self.write_u64(parsed_tx.sequence_number as u64)?;
        
        // Preconditions
        self.serialize_preconditions(&parsed_tx.time_bounds, &parsed_tx.ledger_bounds)?;
        
        // Memo
        self.serialize_memo(parsed_tx.memo.as_ref())?;
        
        // Operations
        self.serialize_operations(&parsed_tx.operations)?;
        
        // Extension (V0 = 0)
        self.write_u32(0)?;
        
        // Signatures
        self.serialize_signatures(&parsed_tx.signatures)?;
        
        Ok(())
    }

    fn serialize_fee_bump_transaction(&mut self, parsed_tx: &ParsedTransaction) -> Result<(), TransactionParseError> {
        printk!("Serializing Fee Bump transaction\n");
        
        // Fee source (muxed account)
        self.write_muxed_account(&ParsedMuxedAccount::Ed25519 {
            account_id: parsed_tx.source_account.clone()
        })?;
        
        // Fee
        self.write_u64(parsed_tx.fee as u64)?;
        
        // Inner transaction type (always TX = 2)
        self.write_u32(2)?;
        
        // Inner transaction (serialize as V1)
        self.serialize_v1_transaction(parsed_tx)?;
        
        // Extension (V0 = 0)
        self.write_u32(0)?;
        
        // Fee bump signatures (replace V1 signatures)
        self.serialize_signatures(&parsed_tx.signatures)?;
        
        Ok(())
    }

    fn serialize_memo(&mut self, memo: Option<&ParsedMemo>) -> Result<(), TransactionParseError> {
        match memo {
            None => {
                self.write_u32(0)?; // MEMO_NONE
            },
            Some(memo) => {
                match memo.memo_type.as_str() {
                    "none" => {
                        self.write_u32(0)?; // MEMO_NONE
                    },
                    "text" => {
                        self.write_u32(1)?; // MEMO_TEXT
                        if let Some(ref text) = memo.value {
                            self.write_string(text)?;
                        } else {
                            self.write_u32(0)?; // Empty string
                        }
                    },
                    "id" => {
                        self.write_u32(2)?; // MEMO_ID
                        if let Some(ref id_str) = memo.value {
                            let id = self.parse_u64_from_string(id_str)?;
                            self.write_u64(id)?;
                        } else {
                            self.write_u64(0)?;
                        }
                    },
                    "hash" => {
                        self.write_u32(3)?; // MEMO_HASH
                        if let Some(ref hash_str) = memo.value {
                            let hash_bytes = self.hex_string_to_bytes(hash_str)?;
                            self.write_fixed_bytes(&hash_bytes, 32)?;
                        } else {
                            self.write_fixed_bytes(&[0u8; 32], 32)?;
                        }
                    },
                    "return" => {
                        self.write_u32(4)?; // MEMO_RETURN
                        if let Some(ref return_str) = memo.value {
                            let return_bytes = self.hex_string_to_bytes(return_str)?;
                            self.write_fixed_bytes(&return_bytes, 32)?;
                        } else {
                            self.write_fixed_bytes(&[0u8; 32], 32)?;
                        }
                    },
                    _ => return Err(TransactionParseError::XdrError),
                }
            }
        }
        Ok(())
    }

    fn serialize_preconditions(
        &mut self,
        time_bounds: &Option<ParsedTimeBounds>,
        ledger_bounds: &Option<ParsedLedgerBounds>
    ) -> Result<(), TransactionParseError> {
        match (time_bounds, ledger_bounds) {
            (None, None) => {
                self.write_u32(0)?; // PRECOND_NONE
            },
            (Some(tb), None) => {
                self.write_u32(1)?; // PRECOND_TIME
                self.serialize_time_bounds(tb)?;
            },
            (time_bounds, ledger_bounds) => {
                self.write_u32(2)?; // PRECOND_V2
                
                // Time bounds (optional)
                if let Some(tb) = time_bounds {
                    self.write_u32(1)?; // Present
                    self.serialize_time_bounds(tb)?;
                } else {
                    self.write_u32(0)?; // Not present
                }
                
                // Ledger bounds (optional)
                if let Some(lb) = ledger_bounds {
                    self.write_u32(1)?; // Present
                    self.serialize_ledger_bounds(lb)?;
                } else {
                    self.write_u32(0)?; // Not present
                }
                
                // Min seq num (optional) - not present
                self.write_u32(0)?;
                
                // Min seq age
                self.write_u64(0)?;
                
                // Min seq ledger gap
                self.write_u32(0)?;
                
                // Extra signers (empty array)
                self.write_u32(0)?;
            }
        }
        Ok(())
    }

    fn serialize_time_bounds(&mut self, time_bounds: &ParsedTimeBounds) -> Result<(), TransactionParseError> {
        self.write_u64(time_bounds.min_time.unwrap_or(0))?;
        self.write_u64(time_bounds.max_time.unwrap_or(0))?;
        Ok(())
    }

    fn serialize_ledger_bounds(&mut self, ledger_bounds: &ParsedLedgerBounds) -> Result<(), TransactionParseError> {
        self.write_u32(ledger_bounds.min_ledger)?;
        self.write_u32(ledger_bounds.max_ledger)?;
        Ok(())
    }

    fn serialize_operations(&mut self, operations: &[ParsedOperation]) -> Result<(), TransactionParseError> {
        // Write operation count
        self.write_u32(operations.len() as u32)?;
        
        for operation in operations.iter() {
            self.serialize_operation(operation)?;
        }
        
        Ok(())
    }

    fn serialize_operation(&mut self, operation: &ParsedOperation) -> Result<(), TransactionParseError> {
        // Source account (optional)
        if let Some(ref source) = operation.source_account {
            self.write_u32(1)?; // Present
            self.write_muxed_account(&ParsedMuxedAccount::Ed25519 {
                account_id: source.clone()
            })?;
        } else {
            self.write_u32(0)?; // Not present
        }
        
        // Operation body
        match &operation.details {
            OperationDetails::CreateAccount { destination, starting_balance } => {
                self.write_i32(0)?; // CREATE_ACCOUNT
                self.write_account_id(destination)?;
                self.write_i64(*starting_balance)?;
            },
            OperationDetails::Payment { destination, asset, amount } => {
                self.write_u32(1)?; // PAYMENT
                self.write_muxed_account(destination)?;
                self.serialize_asset(asset)?;
                self.write_i64(*amount)?;
            },
            OperationDetails::PathPaymentStrictReceive { send_asset, send_max, destination, dest_asset, dest_amount, path } => {
                self.write_u32(2)?; // PATH_PAYMENT_STRICT_RECEIVE
                self.serialize_asset(send_asset)?;
                self.write_i64(*send_max)?;
                self.write_muxed_account(destination)?;
                self.serialize_asset(dest_asset)?;
                self.write_i64(*dest_amount)?;
                self.serialize_asset_path(path)?;
            },
            OperationDetails::PathPaymentStrictSend { send_asset, send_amount, destination, dest_asset, dest_min, path } => {
                self.write_u32(13)?; // PATH_PAYMENT_STRICT_SEND
                self.serialize_asset(send_asset)?;
                self.write_i64(*send_amount)?;
                self.write_muxed_account(destination)?;
                self.serialize_asset(dest_asset)?;
                self.write_i64(*dest_min)?;
                self.serialize_asset_path(path)?;
            },
            OperationDetails::ChangeTrust { asset, limit } => {
                self.write_u32(6)?; // CHANGE_TRUST
                self.serialize_change_trust_asset(asset)?;
                self.write_i64(*limit)?;
            },
            OperationDetails::AllowTrust { trustor, asset_code, authorize } => {
                self.write_u32(7)?; // ALLOW_TRUST
                self.write_account_id(trustor)?;
                self.serialize_asset_code(asset_code)?;
                self.write_u32(*authorize)?;
            },
            OperationDetails::AccountMerge { destination } => {
                self.write_u32(8)?; // ACCOUNT_MERGE
                self.write_muxed_account(&ParsedMuxedAccount::Ed25519 {
                    account_id: destination.clone()
                })?;
            },
            OperationDetails::SetTrustLineFlags { trustor, asset, clear_flags, set_flags } => {
                self.write_u32(21)?; // SET_TRUST_LINE_FLAGS
                self.write_account_id(trustor)?;
                self.serialize_asset(asset)?;
                self.write_u32(*clear_flags)?;
                self.write_u32(*set_flags)?;
            },
            OperationDetails::Other { .. } => {
                return Err(TransactionParseError::UnsupportedOperation);
            }
        }
        
        Ok(())
    }

    fn serialize_asset(&mut self, asset: &ParsedAsset) -> Result<(), TransactionParseError> {
        match asset {
            ParsedAsset::Native => {
                self.write_u32(0)?; // ASSET_TYPE_NATIVE
            },
            ParsedAsset::CreditAlphanum4 { code, issuer } => {
                self.write_u32(1)?; // ASSET_TYPE_CREDIT_ALPHANUM4
                self.write_asset_code4(code)?;
                self.write_account_id(issuer)?;
            },
            ParsedAsset::CreditAlphanum12 { code, issuer } => {
                self.write_u32(2)?; // ASSET_TYPE_CREDIT_ALPHANUM12
                self.write_asset_code12(code)?;
                self.write_account_id(issuer)?;
            }
        }
        Ok(())
    }

    fn serialize_change_trust_asset(&mut self, asset: &ParsedChangeTrustAsset) -> Result<(), TransactionParseError> {
        match asset {
            ParsedChangeTrustAsset::Native => {
                self.write_u32(0)?; // ASSET_TYPE_NATIVE
            },
            ParsedChangeTrustAsset::CreditAlphanum4 { code, issuer } => {
                self.write_u32(1)?; // ASSET_TYPE_CREDIT_ALPHANUM4
                self.write_asset_code4(code)?;
                self.write_account_id(issuer)?;
            },
            ParsedChangeTrustAsset::CreditAlphanum12 { code, issuer } => {
                self.write_u32(2)?; // ASSET_TYPE_CREDIT_ALPHANUM12
                self.write_asset_code12(code)?;
                self.write_account_id(issuer)?;
            },
            ParsedChangeTrustAsset::LiquidityPool { asset_a, asset_b, fee } => {
                self.write_u32(3)?; // ASSET_TYPE_POOL_SHARE
                self.write_u32(0)?; // LIQUIDITY_POOL_CONSTANT_PRODUCT
                self.serialize_asset(asset_a)?;
                self.serialize_asset(asset_b)?;
                self.write_u32(*fee as u32)?;
            }
        }
        Ok(())
    }

    fn serialize_asset_path(&mut self, path: &[ParsedAsset]) -> Result<(), TransactionParseError> {
        self.write_u32(path.len() as u32)?;
        for asset in path.iter() {
            self.serialize_asset(asset)?;
        }
        Ok(())
    }

    fn serialize_asset_code(&mut self, code: &BoundedString) -> Result<(), TransactionParseError> {
        if code.len() <= 4 {
            self.write_u32(1)?; // ASSET_TYPE_CREDIT_ALPHANUM4
            self.write_asset_code4(code)?;
        } else {
            self.write_u32(2)?; // ASSET_TYPE_CREDIT_ALPHANUM12
            self.write_asset_code12(code)?;
        }
        Ok(())
    }

    fn serialize_signatures(&mut self, signatures: &[ParsedSignature]) -> Result<(), TransactionParseError> {
        self.write_u32(signatures.len() as u32)?;
        
        for signature in signatures.iter() {
            // Signature hint (4 bytes)
            self.write_fixed_bytes(&signature.hint, 4)?;
            
            // Signature (variable length)
            self.write_opaque_bytes(&signature.signature)?;
        }
        
        Ok(())
    }

    // Low-level writing functions
    fn write_u32(&mut self, value: u32) -> Result<(), TransactionParseError> {
        let bytes = value.to_be_bytes();
        self.buffer.extend_from_slice(&bytes);
        Ok(())
    }

    fn write_u64(&mut self, value: u64) -> Result<(), TransactionParseError> {
        let bytes = value.to_be_bytes();
        self.buffer.extend_from_slice(&bytes);
        Ok(())
    }

    fn write_i64(&mut self, value: i64) -> Result<(), TransactionParseError> {
        let bytes = value.to_be_bytes();
        self.buffer.extend_from_slice(&bytes);
        Ok(())
    }

    fn write_i32(&mut self, value: i32) -> Result<(), TransactionParseError> {
        let bytes = value.to_be_bytes();
        self.buffer.extend_from_slice(&bytes);
        Ok(())
    }

    fn write_fixed_bytes(&mut self, bytes: &[u8], expected_len: usize) -> Result<(), TransactionParseError> {
        if bytes.len() > expected_len {
            return Err(TransactionParseError::DataTooLarge);
        }
        
        self.buffer.extend_from_slice(bytes);
        
        // Pad with zeros if needed
        for _ in bytes.len()..expected_len {
            self.buffer.push(0);
        }
        
        Ok(())
    }

    fn write_opaque_bytes(&mut self, bytes: &[u8]) -> Result<(), TransactionParseError> {
        // Write length
        self.write_u32(bytes.len() as u32)?;
        
        // Write data
        self.buffer.extend_from_slice(bytes);
        
        // Add padding to 4-byte boundary
        let padding = (4 - (bytes.len() % 4)) % 4;
        for _ in 0..padding {
            self.buffer.push(0);
        }
        
        Ok(())
    }

    fn write_string(&mut self, s: &BoundedString) -> Result<(), TransactionParseError> {
        let bytes = s.as_bytes();
        self.write_opaque_bytes(bytes)
    }

    fn write_account_id(&mut self, account_hex: &BoundedString) -> Result<(), TransactionParseError> {
        self.write_u32(0)?;
        let bytes = self.hex_string_to_bytes(account_hex)?;
        self.write_fixed_bytes(&bytes, 32)
    }

    fn write_muxed_account(&mut self, account: &ParsedMuxedAccount) -> Result<(), TransactionParseError> {
        match account {
            ParsedMuxedAccount::Ed25519 { account_id } => {
                self.write_u32(0)?; // KEY_TYPE_ED25519
                let bytes = self.hex_string_to_bytes(account_id)?;
                self.write_fixed_bytes(&bytes, 32)?;
            },
            ParsedMuxedAccount::MuxedEd25519 { id, account_id } => {
                self.write_u32(256)?; // KEY_TYPE_MUXED_ED25519
                self.write_u64(*id)?;
                let bytes = self.hex_string_to_bytes(account_id)?;
                self.write_fixed_bytes(&bytes, 32)?;
            }
        }
        Ok(())
    }

    fn write_asset_code4(&mut self, code: &BoundedString) -> Result<(), TransactionParseError> {
        let mut code_bytes = [0u8; 4];
        let bytes = code.as_bytes();
        let len = bytes.len().min(4);
        code_bytes[..len].copy_from_slice(&bytes[..len]);
        self.buffer.extend_from_slice(&code_bytes);
        Ok(())
    }

    fn write_asset_code12(&mut self, code: &BoundedString) -> Result<(), TransactionParseError> {
        let mut code_bytes = [0u8; 12];
        let bytes = code.as_bytes();
        let len = bytes.len().min(12);
        code_bytes[..len].copy_from_slice(&bytes[..len]);
        self.buffer.extend_from_slice(&code_bytes);
        Ok(())
    }

    // Helper functions
    fn parse_u64_from_string(&self, s: &BoundedString) -> Result<u64, TransactionParseError> {
        let bytes = s.as_bytes();
        let mut result = 0u64;
        
        for &byte in bytes {
            match byte {
                b'0'..=b'9' => {
                    let digit = (byte - b'0') as u64;
                    result = result.checked_mul(10)
                        .and_then(|r| r.checked_add(digit))
                        .ok_or(TransactionParseError::XdrError)?;
                },
                _ => return Err(TransactionParseError::XdrError),
            }
        }
        
        Ok(result)
    }

    fn hex_string_to_bytes(&self, hex_str: &BoundedString) -> Result<Vec<u8>, TransactionParseError> {
        let hex_chars = hex_str.as_bytes();
        if hex_chars.len() % 2 != 0 {
            return Err(TransactionParseError::XdrError);
        }

        let mut bytes = Vec::new();
        let mut i = 0;
        while i < hex_chars.len() {
            let high = self.hex_char_to_u8(hex_chars[i])?;
            let low = self.hex_char_to_u8(hex_chars[i + 1])?;
            bytes.push((high << 4) | low);
            i += 2;
        }
        Ok(bytes)
    }

    fn hex_char_to_u8(&self, c: u8) -> Result<u8, TransactionParseError> {
        match c {
            b'0'..=b'9' => Ok(c - b'0'),
            b'a'..=b'f' => Ok(c - b'a' + 10),
            b'A'..=b'F' => Ok(c - b'A' + 10),
            _ => Err(TransactionParseError::XdrError),
        }
    }

    fn encode_base64(&self, data: &[u8]) -> Result<String, TransactionParseError> {
        let mut buf = [0u8; MAX_XDR_LEN * 4 / 3 + 4]; // Base64 expansion ratio
        let encoded = Base64::encode(data, &mut buf)
            .map_err(|_| TransactionParseError::Base64Error)?;
        Ok(String::from(encoded))
    }
}

impl Default for StellarTransactionSerializer {
    fn default() -> Self {
        Self::new()
    }
}