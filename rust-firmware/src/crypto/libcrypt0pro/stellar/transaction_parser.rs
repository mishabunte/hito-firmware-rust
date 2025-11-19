#![no_std]

extern crate alloc;
use alloc::{string::String, vec::Vec, format};

use stellar_xdr::curr::{
    TransactionEnvelope, TransactionV1Envelope, FeeBumpTransactionEnvelope,
    TransactionV0Envelope, Uint256,
    AssetCode,
    LiquidityPoolParameters,
    PublicKey, FeeBumpTransactionInnerTx,
    Operation, OperationBody, Asset, ChangeTrustAsset, AccountId, MuxedAccount,
    DecoratedSignature, TimeBounds, LedgerBounds, Preconditions, PreconditionsV2,
    Memo, Int64, Uint32, Uint64, ReadXdr, WriteXdr, SequenceNumber,
};

use crate::crypto::crypt0::bytes_to_hex;

use crate::printk;

//use base64ct::{Base64, Encoding};

// Configuration constants - adjust based on your needs
pub const MAX_OPERATIONS: usize = 100;
pub const MAX_PATH_ASSETS: usize = 5;
pub const MAX_SIGNATURES: usize = 20;
pub const MAX_CLAIMANTS: usize = 10;
pub const MAX_STRING_LEN: usize = 64;
pub const MAX_DATA_VALUE_LEN: usize = 64;
pub const MAX_ASSET_CODE_LEN: usize = 12;
pub const MAX_XDR_LEN: usize = 8192; // Maximum expected XDR length

#[derive(Debug, Clone)]
pub struct ParsedTransaction {
    pub source_account: String,
    pub sequence_number: i64,
    pub fee: i64,
    pub memo: Option<ParsedMemo>,
    pub time_bounds: Option<ParsedTimeBounds>,
    pub ledger_bounds: Option<ParsedLedgerBounds>,
    pub operations: Vec<ParsedOperation>,
    pub signatures: Vec<ParsedSignature>,
    pub envelope_type: TransactionEnvelopeType,
}

#[derive(Debug, Clone)]
pub enum TransactionEnvelopeType {
    TxV0,
    Tx,
    TxFeeBump,
}

#[derive(Debug, Clone)]
pub struct ParsedMemo {
    pub memo_type: String,
    pub value: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ParsedTimeBounds {
    pub min_time: Option<u64>,
    pub max_time: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct ParsedLedgerBounds {
    pub min_ledger: u32,
    pub max_ledger: u32,
}

#[derive(Debug, Clone)]
pub struct ParsedOperation {
    pub operation_type: String,
    pub source_account: Option<String>,
    pub details: OperationDetails,
}

#[derive(Debug, Clone)]
pub enum ParsedMuxedAccount {
    Ed25519 {
        account_id: String,
    },
    MuxedEd25519 {
        id: u64,
        account_id: String,
    },
}

#[derive(Debug, Clone)]
pub enum OperationDetails {
    CreateAccount {
        destination: String,
        starting_balance: i64,
    },
    Payment {
        destination: ParsedMuxedAccount,
        asset: ParsedAsset,
        amount: i64,
    },
    Other {
        operation_type: String,
        raw_data_len: usize, // Store only length in no-std
    },
    PathPaymentStrictSend {
        send_asset: ParsedAsset,
        send_amount: i64,
        destination: ParsedMuxedAccount,
        dest_asset: ParsedAsset,
        dest_min: i64,
        path: Vec<ParsedAsset>,
    },
    PathPaymentStrictReceive {
        send_asset: ParsedAsset,
        send_max: i64,
        destination: ParsedMuxedAccount,
        dest_asset: ParsedAsset,
        dest_amount: i64,
        path: Vec<ParsedAsset>,
    },
    ChangeTrust {
        asset: ParsedChangeTrustAsset,
        limit: i64,
    },
    AccountMerge {
        destination: String,
    },
    AllowTrust {
        trustor: String,
        asset_code: String,
        authorize: u32,
    },
    SetTrustLineFlags {
        trustor: String,
        asset: ParsedAsset,
        clear_flags: u32,
        set_flags: u32,
    },
}

#[derive(Debug, Clone)]
pub enum ParsedAsset {
    Native,
    CreditAlphanum4 {
        code: String,
        issuer: String,
    },
    CreditAlphanum12 {
        code: String,
        issuer: String,
    }, 
}

#[derive(Debug, Clone)]
pub enum ParsedChangeTrustAsset {
    Native,
    CreditAlphanum4 {
        code: String,
        issuer: String,
    },
    CreditAlphanum12 {
        code: String,
        issuer: String,
    }, 
    LiquidityPool {
        asset_a: ParsedAsset,
        asset_b: ParsedAsset,
        fee: i32,
    },
}

#[derive(Debug, Clone)]
pub struct ParsedSigner {
    pub key: String,
    pub weight: u32,
}

#[derive(Debug, Clone)]
pub struct ParsedSignature {
    pub hint: [u8; 4], // Signature hints are always 4 bytes
    pub signature: [u8; 64], // Ed25519 signatures are 64 bytes
}

#[derive(Debug)]
pub enum TransactionParseError {
    Base64Error,
    XdrError,
    InvalidEnvelopeType,
    UnsupportedOperation,
    StringTooLong,
    TooManyOperations,
    TooManySignatures,
    PayloadTooLarge,
    AccountIdError,
    MuxedAccountError,
    PayloadTooSmall,
}

impl core::fmt::Display for TransactionParseError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            TransactionParseError::Base64Error => write!(f, "Base64 decode error"),
            TransactionParseError::XdrError => write!(f, "XDR parsing error"),
            TransactionParseError::InvalidEnvelopeType => write!(f, "Invalid transaction envelope type"),
            TransactionParseError::UnsupportedOperation => write!(f, "Unsupported operation type"),
            TransactionParseError::StringTooLong => write!(f, "String too long"),
            TransactionParseError::TooManyOperations => write!(f, "Too many operations in transaction"),
            TransactionParseError::TooManySignatures => write!(f, "Too many signatures in transaction"),
            TransactionParseError::PayloadTooLarge => write!(f, "Payload too large"),
            TransactionParseError::AccountIdError => write!(f, "Failed to parse account ID"),
            TransactionParseError::MuxedAccountError => write!(f, "Failed to parse muxed account"),
            TransactionParseError::PayloadTooSmall => write!(f, "Payload too small"),
        }
    }
}

pub struct StellarTransactionParser;

impl StellarTransactionParser {
    pub fn new() -> Self {
        Self
    }

    /// Parse a base64-encoded transaction envelope into structured data
    pub fn parse_transaction(base64_data: &str) -> Result<ParsedTransaction, TransactionParseError> {
        if base64_data.len() > MAX_XDR_LEN {
            return Err(TransactionParseError::PayloadTooLarge);
        }
        if base64_data.len() < 20 {
            return Err(TransactionParseError::PayloadTooSmall);
        }
        // Decode base64 using no-std compatible base64 decoder
        //let xdr_bytes = Self::decode_base64(base64_data)?;
        let xdr_bytes = base64_data.as_bytes();
        
        // Parse XDR envelope
        let envelope = TransactionEnvelope::from_xdr_base64(&xdr_bytes, stellar_xdr::curr::Limits::none());

        if envelope.is_err() {
            printk!("XDR parsing error: {:?}\n", envelope.err());
            return Err(TransactionParseError::XdrError);
        }
        
        match envelope.unwrap() {
            TransactionEnvelope::TxV0(env) => StellarTransactionParser::parse_v0_transaction(env),
            TransactionEnvelope::Tx(env) => StellarTransactionParser::parse_v1_transaction(env),
            TransactionEnvelope::TxFeeBump(env) => StellarTransactionParser::parse_fee_bump_transaction(env),
        }
    }

    fn uint256_to_bounded_hex( v: &Uint256) -> Result<String, TransactionParseError> {
        let hex = bytes_to_hex(&v.0);
        if hex.len() > MAX_STRING_LEN {
            return Err(TransactionParseError::StringTooLong);
        }
        Ok(hex)
    }

    fn asset_code_to_string( code: &AssetCode) -> Result<String, TransactionParseError> {
        match code {
            AssetCode::CreditAlphanum4(c) => {
                let code_bytes = &c.0;
                let code_str = core::str::from_utf8(code_bytes).unwrap_or("invalid");
                let trimmed_code = code_str.trim_end_matches('\0');
                String::try_from(trimmed_code).map_err(|_| TransactionParseError::StringTooLong)
            },
            AssetCode::CreditAlphanum12(c) => {
                let code_bytes = &c.0;
                let code_str = core::str::from_utf8(code_bytes).unwrap_or("invalid");
                let trimmed_code = code_str.trim_end_matches('\0');
                String::try_from(trimmed_code).map_err(|_| TransactionParseError::StringTooLong)
            },
        }
    }

    fn parse_v0_transaction(envelope: TransactionV0Envelope) -> Result<ParsedTransaction, TransactionParseError> {
        let tx = envelope.tx;
        
        let time_bounds = tx.time_bounds
            .as_ref()
            .map(|tb| Self::parse_time_bounds(tb))
            .transpose()?;
        let ledger_bounds = None; // V0 transactions do not have ledger bounds
        
        Ok(ParsedTransaction {
            source_account: { Self::uint256_to_bounded_hex(&tx.source_account_ed25519)? },
            sequence_number: tx.seq_num.0,
            fee: tx.fee as i64,
            memo: Some(Self::parse_memo(&tx.memo)?),
            time_bounds,
            ledger_bounds,
            operations: Self::parse_operations(&tx.operations)?,
            signatures: Self::parse_signatures(&envelope.signatures)?,
            envelope_type: TransactionEnvelopeType::TxV0,
        })
    }

    fn parse_v1_transaction( envelope: TransactionV1Envelope) -> Result<ParsedTransaction, TransactionParseError> {
        printk!("Parsing V1 transaction envelope...\n");
        let tx = envelope.tx;
        printk!("Parsing V1 transaction...\n");

        let (time_bounds, ledger_bounds) = Self::parse_preconditions(&tx.cond)?;
        
        Ok(ParsedTransaction {
            source_account: Self::muxed_account_to_string(&tx.source_account)?,
            sequence_number: tx.seq_num.0,
            fee: tx.fee as i64,
            memo: Some(Self::parse_memo(&tx.memo)?),
            time_bounds,
            ledger_bounds,
            operations: Self::parse_operations(&tx.operations)?,
            signatures: Self::parse_signatures(&envelope.signatures)?,
            envelope_type: TransactionEnvelopeType::Tx,
        })
    }

    fn parse_fee_bump_transaction( envelope: FeeBumpTransactionEnvelope) -> Result<ParsedTransaction, TransactionParseError> {
        let fee_bump = envelope.tx;
        printk!("Parsing Fee Bump transaction...\n");

        // Extract the inner transaction
        let inner_envelope = match fee_bump.inner_tx {
            FeeBumpTransactionInnerTx::Tx(env) => env,
        };

        let mut parsed = Self::parse_v1_transaction(inner_envelope)?;
        
        // Override with fee bump details
        parsed.source_account = Self::muxed_account_to_string(&fee_bump.fee_source)?;
        parsed.fee = fee_bump.fee;
        parsed.signatures = Self::parse_signatures(&envelope.signatures)?;
        parsed.envelope_type = TransactionEnvelopeType::TxFeeBump;
        
        Ok(parsed)
    }

    fn parse_memo( memo: &Memo) -> Result<ParsedMemo, TransactionParseError> {
        printk!("Parsing memo: {:?}\n", memo);
        let parsed_memo = match memo {
            Memo::None => ParsedMemo {
                memo_type: String::try_from("none").map_err(|_| TransactionParseError::StringTooLong)?,
                value: None,
            },
            Memo::Text(text) => {
                let text_str = core::str::from_utf8(&text).unwrap_or("invalid_utf8");
                ParsedMemo {
                    memo_type: String::try_from("text").map_err(|_| TransactionParseError::StringTooLong)?,
                    value: Some(String::try_from(text_str).map_err(|_| TransactionParseError::StringTooLong)?),
                }
            },
            Memo::Id(id) => ParsedMemo {
                memo_type: String::try_from("id").map_err(|_| TransactionParseError::StringTooLong)?,
                value: Some(Self::u64_to_string(id.clone())?),
            },
            Memo::Hash(hash) => ParsedMemo {
                memo_type: String::try_from("hash").map_err(|_| TransactionParseError::StringTooLong)?,
                value: Some(bytes_to_hex(&hash.0)),
            },
            Memo::Return(ret) => ParsedMemo {
                memo_type: String::try_from("return").map_err(|_| TransactionParseError::StringTooLong)?,
                value: Some(bytes_to_hex(&ret.0)),
            },
        };
        printk!("Parsed memo: {:?}\n", parsed_memo);
        
        Ok(parsed_memo)
    }

    fn parse_preconditions( preconditions: &Preconditions) -> Result<(Option<ParsedTimeBounds>, Option<ParsedLedgerBounds>), TransactionParseError> {
        match preconditions {
            Preconditions::None => Ok((None, None)),
            Preconditions::Time(time_bounds) => Ok((Some(Self::parse_time_bounds(time_bounds)?), None)),
            Preconditions::V2(preconditions_v2) => {
                let time_bounds = preconditions_v2.time_bounds
                    .as_ref()
                    .map(|tb| Self::parse_time_bounds(tb))
                    .transpose()?;
                let ledger_bounds = preconditions_v2.ledger_bounds
                    .as_ref()
                    .map(|lb| Self::parse_ledger_bounds(lb))
                    .transpose()?;
                Ok((time_bounds, ledger_bounds))
            }
        }
    }
    
    fn parse_time_bounds(time_bounds: &TimeBounds) -> Result<ParsedTimeBounds, TransactionParseError> {
        Ok(ParsedTimeBounds {
            min_time: Some(time_bounds.min_time.0),
            max_time: Some(time_bounds.max_time.0),
        })
    }

    fn parse_ledger_bounds(ledger_bounds: &LedgerBounds) -> Result<ParsedLedgerBounds, TransactionParseError> {
        Ok(ParsedLedgerBounds {
            min_ledger: ledger_bounds.min_ledger,
            max_ledger: ledger_bounds.max_ledger,
        })
    }

    fn parse_operations(operations: &[Operation]) -> Result<Vec<ParsedOperation>, TransactionParseError> {
        let mut parsed_ops = Vec::new();
        
        for operation in operations.iter() {
            let parsed_op = Self::parse_operation(operation)?;
            parsed_ops.push(parsed_op);
        }
        
        Ok(parsed_ops)
    }

    fn parse_operation(operation: &Operation) -> Result<ParsedOperation, TransactionParseError> {
        let source_account = match &operation.source_account {
            Some(account) => Some(Self::muxed_account_to_string(account)?),
            None => None,
        };
        
        let operation_type = match &operation.body {
            OperationBody::CreateAccount(create_account) => (
                String::try_from("create_account").map_err(|_| TransactionParseError::StringTooLong)?,
                OperationDetails::CreateAccount {
                    destination: Self::account_id_to_string(&create_account.destination)?,
                    starting_balance: create_account.starting_balance,
                }
            ),
            OperationBody::Payment(payment) => (
                String::try_from("payment").map_err(|_| TransactionParseError::StringTooLong)?,
                OperationDetails::Payment {
                    destination: Self::parse_muxed_account(&payment.destination)?,
                    asset: Self::parse_asset(&payment.asset)?,
                    amount: payment.amount,
                }
            ),
            OperationBody::PathPaymentStrictSend(ppss) => (
                String::try_from("path_payment_strict_send").map_err(|_| TransactionParseError::StringTooLong)?,
                OperationDetails::PathPaymentStrictSend {
                    send_asset: Self::parse_asset(&ppss.send_asset)?,
                    send_amount: ppss.send_amount,
                    destination: Self::parse_muxed_account(&ppss.destination)?,
                    dest_asset: Self::parse_asset(&ppss.dest_asset)?,
                    dest_min: ppss.dest_min,
                    path: Self::parse_assets_path(&ppss.path)?,
                }
            ),
            OperationBody::PathPaymentStrictReceive(ppss) => (
                String::try_from("path_payment_strict_send").map_err(|_| TransactionParseError::StringTooLong)?,
                OperationDetails::PathPaymentStrictReceive {
                    send_asset: Self::parse_asset(&ppss.send_asset)?,
                    send_max: ppss.send_max,
                    destination: Self::parse_muxed_account(&ppss.destination)?,
                    dest_asset: Self::parse_asset(&ppss.dest_asset)?,
                    dest_amount: ppss.dest_amount,
                    path: Self::parse_assets_path(&ppss.path)?,
                }
            ),
            OperationBody::ChangeTrust(change_trust) => (
                String::try_from("change_trust").map_err(|_| TransactionParseError::StringTooLong)?,
                OperationDetails::ChangeTrust {
                    asset: Self::parse_change_trust_asset(&change_trust.line)?,
                    limit: change_trust.limit,
                }
            ),
            OperationBody::AccountMerge(muxed_account) => (
                String::try_from("account_merge").map_err(|_| TransactionParseError::StringTooLong)?,
                OperationDetails::AccountMerge {
                    destination: Self::muxed_account_to_string(muxed_account)?,
                }
            ),
            OperationBody::AllowTrust(allow_trust) => (
                String::try_from("allow_trust").map_err(|_| TransactionParseError::StringTooLong)?,
                OperationDetails::AllowTrust {
                    trustor: Self::account_id_to_string(&allow_trust.trustor)?,
                    asset_code: Self::asset_code_to_string(&allow_trust.asset)?,
                    authorize: allow_trust.authorize,
                }
            ),
            OperationBody::SetTrustLineFlags(set_trust_line_flags) => (
                String::try_from("set_trust_line_flags").map_err(|_| TransactionParseError::StringTooLong)?,
                OperationDetails::SetTrustLineFlags {
                    trustor: Self::account_id_to_string(&set_trust_line_flags.trustor)?,
                    asset: Self::parse_asset(&set_trust_line_flags.asset)?,
                    clear_flags: set_trust_line_flags.clear_flags,
                    set_flags: set_trust_line_flags.set_flags,
                }
            ),
            _ => (
                String::try_from("other").map_err(|_| TransactionParseError::StringTooLong)?,
                OperationDetails::Other {
                    operation_type: String::try_from("unknown").map_err(|_| TransactionParseError::StringTooLong)?,
                    raw_data_len: 0, // In no-std, we avoid storing raw data
                }
            ),
        };
        
        Ok(ParsedOperation {
            operation_type: operation_type.0,
            source_account,
            details: operation_type.1,
        })
    }

    fn parse_assets_path( path: &[Asset]) -> Result<Vec<ParsedAsset>, TransactionParseError> {
        let mut parsed_assets = Vec::new();
        
        for asset in path.iter() {
            let parsed_asset = Self::parse_asset(asset)?;
            parsed_assets.push(parsed_asset);
        }
        
        Ok(parsed_assets)
    }

    fn parse_muxed_account( muxed_account: &MuxedAccount) -> Result<ParsedMuxedAccount, TransactionParseError> {
        match muxed_account {
            MuxedAccount::Ed25519(account_id) => Ok(ParsedMuxedAccount::Ed25519 {
                account_id: bytes_to_hex(&account_id.0),
            }),
            MuxedAccount::MuxedEd25519(muxed) => Ok(ParsedMuxedAccount::MuxedEd25519 {
                id: muxed.id,
                account_id: bytes_to_hex(&muxed.ed25519.0),
            }),
        }
    }

    fn parse_asset( asset: &Asset) -> Result<ParsedAsset, TransactionParseError> {
        let parsed_asset = match asset {
            Asset::Native => ParsedAsset::Native {
                // asset_type: String::try_from("native").map_err(|_| TransactionParseError::StringTooLong)?,
                // asset_code: None,
                // issuer: None,
            },
            Asset::CreditAlphanum4(alpha4) => {
                let code_bytes = &alpha4.asset_code.0;
                let code_str = core::str::from_utf8(code_bytes).unwrap_or("invalid");
                let trimmed_code = code_str.trim_end_matches('\0');
                
                ParsedAsset::CreditAlphanum4 {
                    code: String::from(trimmed_code),
                    issuer: Self::account_id_to_string(&alpha4.issuer)?,
                }
            },
            Asset::CreditAlphanum12(alpha12) => {
                let code_bytes = &alpha12.asset_code.0;
                let code_str = core::str::from_utf8(code_bytes).unwrap_or("invalid");
                let trimmed_code = code_str.trim_end_matches('\0');

                ParsedAsset::CreditAlphanum12 {
                    code: String::from(trimmed_code),
                    issuer: Self::account_id_to_string(&alpha12.issuer)?,
                }
            },
        };
        
        Ok(parsed_asset)
    }

    fn parse_change_trust_asset( asset: &ChangeTrustAsset) -> Result<ParsedChangeTrustAsset, TransactionParseError> {
        let parsed_asset = match asset {
            ChangeTrustAsset::Native => ParsedChangeTrustAsset::Native,
            ChangeTrustAsset::CreditAlphanum4(alpha4) => {
                let code_bytes = &alpha4.asset_code.0;
                let code_str = core::str::from_utf8(code_bytes).unwrap_or("invalid");
                let trimmed_code = code_str.trim_end_matches('\0');
                
                ParsedChangeTrustAsset::CreditAlphanum4 {
                    code: String::from(trimmed_code),
                    issuer: Self::account_id_to_string(&alpha4.issuer)?,
                }
            },
            ChangeTrustAsset::CreditAlphanum12(alpha12) => {
                let code_bytes = &alpha12.asset_code.0;
                let code_str = core::str::from_utf8(code_bytes).unwrap_or("invalid");
                let trimmed_code = code_str.trim_end_matches('\0');

                ParsedChangeTrustAsset::CreditAlphanum12 {
                    code: String::from(trimmed_code),
                    issuer: Self::account_id_to_string(&alpha12.issuer)?,
                }
            },
            ChangeTrustAsset::PoolShare(lp) => {
                match lp {
                    LiquidityPoolParameters::LiquidityPoolConstantProduct(cp) => {
                        let asset_a = Self::parse_asset(&cp.asset_a)?;
                        let asset_b = Self::parse_asset(&cp.asset_b)?;
                        ParsedChangeTrustAsset::LiquidityPool {
                            asset_a,
                            asset_b,
                            fee: cp.fee,
                        }
                    },
                }
            },
        };
        
        Ok(parsed_asset)
    }

    fn parse_signatures(signatures: &[DecoratedSignature]) -> Result<Vec<ParsedSignature>, TransactionParseError> {
        let mut parsed_sigs = Vec::new();
        
        for sig in signatures.iter() {
            let mut hint = [0u8; 4];
            let mut signature = [0u8; 64];
            
            // Copy hint (always 4 bytes)
            for (i, &byte) in sig.hint.0.iter().enumerate() {
                hint[i] = byte;
            }
            
            // Copy signature (up to 64 bytes for Ed25519)
            for (i, &byte) in sig.signature.0.iter().enumerate() {
                signature[i] = byte;
            }
            
            let parsed_sig = ParsedSignature { hint, signature };
            parsed_sigs.push(parsed_sig);
        }
        
        Ok(parsed_sigs)
    }

    fn account_id_to_string(
        
        account_id: &AccountId,
    ) -> Result<String, TransactionParseError> {
        match &account_id.0 {
            PublicKey::PublicKeyTypeEd25519(ed) => Ok(bytes_to_hex(&ed.0)),
        }
    }

    fn muxed_account_to_string(muxed_account: &MuxedAccount) -> Result<String, TransactionParseError> {
        printk!("Parsing muxed account: {:?}\n", muxed_account);
        match muxed_account {
            MuxedAccount::Ed25519(account_id) => Ok(bytes_to_hex(&account_id.0)),
            MuxedAccount::MuxedEd25519(muxed) => {
                // For muxed accounts, we'd normally encode both the account and ID
                // For simplicity, just return the account part as hex
                Ok(bytes_to_hex(&muxed.ed25519.0))
            }
        }
    }

    /// Convert stroops to XLM string representation with bounded string
    fn stroops_to_xlm_bounded(stroops: i64) -> Result<String, TransactionParseError> {
        // Simple conversion avoiding floating point in no-std
        let whole_xlm = stroops / 10_000_000;
        let fractional = stroops % 10_000_000;
        
        if fractional == 0 {
            Self::i64_to_string(whole_xlm)
        } else {
            // For simplicity, just return the stroops value as string in no-std
            Self::i64_to_string(stroops)
        }
    }

    fn u64_to_string(value: u64) -> Result<String, TransactionParseError> {
        let mut buffer = [0u8; 20]; // u64 max is 20 digits
        let mut i = buffer.len();
        let mut n = value;
        
        if n == 0 {
            return String::try_from("0").map_err(|_| TransactionParseError::StringTooLong);
        }
        
        while n > 0 {
            i -= 1;
            buffer[i] = b'0' + (n % 10) as u8;
            n /= 10;
        }
        
        let str_slice = core::str::from_utf8(&buffer[i..]).map_err(|_| TransactionParseError::StringTooLong)?;
        String::try_from(str_slice).map_err(|_| TransactionParseError::StringTooLong)
    }

    fn i64_to_string(value: i64) -> Result<String, TransactionParseError> {
        if value >= 0 {
            Self::u64_to_string(value as u64)
        } else {
            let pos_str = Self::u64_to_string((-value) as u64)?;
            let mut result = String::new();
            result.push('-');
            result.push_str(&pos_str);
            Ok(result)
        }
    }

    pub fn stroops_to_xlm_string(stroops: i64) -> String {
        let whole_xlm = stroops / 10_000_000;
        let fractional = stroops % 10_000_000;
        
        if fractional == 0 {
            format!("{}", whole_xlm)
        } else {
            format!("{}.{}", whole_xlm, fractional)
        }
    }
}

impl Default for StellarTransactionParser {
    fn default() -> Self {
        Self::new()
    }
}