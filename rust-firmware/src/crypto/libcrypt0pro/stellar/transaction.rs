#![no_std]

extern crate alloc;
use alloc::{string::String, vec::Vec, format};
use stellar_strkey::Contract;
use core::str::{Bytes, FromStr};

use stellar_xdr::curr::{
    TransactionEnvelope, TransactionV1Envelope, FeeBumpTransactionEnvelope,
    TransactionV0Envelope, Uint256,
    Transaction, TransactionV0, FeeBumpTransaction,
    ScVal, LedgerKey,
    AssetCode,
    LiquidityPoolParameters,
    PublicKey, FeeBumpTransactionInnerTx,
    Operation, OperationBody, Asset, ChangeTrustAsset, AccountId, MuxedAccount,
    DecoratedSignature, TimeBounds, LedgerBounds, Preconditions, PreconditionsV2,
    Memo, Int64, Uint32, Uint64, ReadXdr, WriteXdr, SequenceNumber,
};
use heapless::{Vec as HeaplessVec, String as HeaplessString};

use crate::printk;

use base64ct::{Base64, Encoding};

// Configuration constants - adjust based on your needs
pub const MAX_OPERATIONS: usize = 100;
pub const MAX_PATH_ASSETS: usize = 5;
pub const MAX_SIGNATURES: usize = 20;
pub const MAX_CLAIMANTS: usize = 10;
pub const MAX_STRING_LEN: usize = 64;
pub const MAX_DATA_VALUE_LEN: usize = 64;
pub const MAX_ASSET_CODE_LEN: usize = 12;
const MAX_XDR_LEN: usize = 8192; // Maximum expected XDR length

pub type BoundedString = HeaplessString<MAX_STRING_LEN>;
pub type BoundedVec<T> = HeaplessVec<T, MAX_OPERATIONS>;
pub type BoundedAssetVec = HeaplessVec<ParsedAsset, MAX_PATH_ASSETS>;
pub type BytesM<const N: usize> = HeaplessVec<u8, N>;
pub type BoundedSignatureVec = HeaplessVec<ParsedSignature, MAX_SIGNATURES>;
pub type BoundedClaimantVec = HeaplessVec<BoundedString, MAX_CLAIMANTS>;

#[derive(Debug, Clone)]
pub struct ParsedTransaction {
    pub source_account: BoundedString,
    pub sequence_number: i64,
    pub fee: i64,
    pub memo: Option<ParsedMemo>,
    pub time_bounds: Option<ParsedTimeBounds>,
    pub ledger_bounds: Option<ParsedLedgerBounds>,
    pub operations: BoundedVec<ParsedOperation>,
    pub signatures: BoundedSignatureVec,
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
    pub memo_type: BoundedString,
    pub value: Option<BoundedString>,
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
    pub operation_type: BoundedString,
    pub source_account: Option<BoundedString>,
    pub details: OperationDetails,
}

#[derive(Debug, Clone)]
pub enum ParsedMuxedAccount {
    Ed25519 {
        account_id: BoundedString,
    },
    MuxedEd25519 {
        id: u64,
        account_id: BoundedString,
    },
}

#[derive(Debug, Clone)]
pub enum OperationDetails {
    CreateAccount {
        destination: BoundedString,
        starting_balance: i64,
    },
    Payment {
        destination: ParsedMuxedAccount,
        asset: ParsedAsset,
        amount: i64,
    },
    Other {
        operation_type: BoundedString,
        raw_data_len: usize, // Store only length in no-std
    },
    PathPaymentStrictSend {
        send_asset: ParsedAsset,
        send_amount: i64,
        destination: ParsedMuxedAccount,
        dest_asset: ParsedAsset,
        dest_min: i64,
        path: BoundedAssetVec,
    },
    PathPaymentStrictReceive {
        send_asset: ParsedAsset,
        send_max: i64,
        destination: ParsedMuxedAccount,
        dest_asset: ParsedAsset,
        dest_amount: i64,
        path: BoundedAssetVec,
    },
    ChangeTrust {
        asset: ParsedChangeTrustAsset,
        limit: i64,
    },
    AccountMerge {
        destination: BoundedString,
    },
    AllowTrust {
        trustor: BoundedString,
        asset_code: BoundedString,
        authorize: u32,
    },
    SetTrustLineFlags {
        trustor: BoundedString,
        asset: ParsedAsset,
        clear_flags: u32,
        set_flags: u32,
    },
}

#[derive(Debug, Clone)]
pub enum ParsedAsset {
    Native,
    CreditAlphanum4 {
        code: BoundedString,
        issuer: BoundedString,
    },
    CreditAlphanum12 {
        code: BoundedString,
        issuer: BoundedString,
    }, 
}

#[derive(Debug, Clone)]
pub enum ParsedChangeTrustAsset {
    Native,
    CreditAlphanum4 {
        code: BoundedString,
        issuer: BoundedString,
    },
    CreditAlphanum12 {
        code: BoundedString,
        issuer: BoundedString,
    }, 
    LiquidityPool {
        asset_a: ParsedAsset,
        asset_b: ParsedAsset,
        fee: i32,
    },
}

#[derive(Debug, Clone)]
pub struct ParsedSigner {
    pub key: BoundedString,
    pub weight: u32,
}

#[derive(Debug, Clone)]
pub struct ParsedSignature {
    pub hint: HeaplessVec<u8, 4>, // Signature hints are always 4 bytes
    pub signature: HeaplessVec<u8, 64>, // Ed25519 signatures are 64 bytes
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
    DataTooLarge,
    AccountIdError,
    MuxedAccountError,
}

impl core::fmt::Display for TransactionParseError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            TransactionParseError::Base64Error => write!(f, "Base64 decode error"),
            TransactionParseError::XdrError => write!(f, "XDR parsing error"),
            TransactionParseError::InvalidEnvelopeType => write!(f, "Invalid transaction envelope type"),
            TransactionParseError::UnsupportedOperation => write!(f, "Unsupported operation type"),
            TransactionParseError::StringTooLong => write!(f, "String too long for bounded container"),
            TransactionParseError::TooManyOperations => write!(f, "Too many operations in transaction"),
            TransactionParseError::TooManySignatures => write!(f, "Too many signatures in transaction"),
            TransactionParseError::DataTooLarge => write!(f, "Data too large for bounded container"),
            TransactionParseError::AccountIdError => write!(f, "Failed to parse account ID"),
            TransactionParseError::MuxedAccountError => write!(f, "Failed to parse muxed account"),
        }
    }
}

pub struct StellarTransactionParser;

impl StellarTransactionParser {
    pub fn new() -> Self {
        Self
    }

    /// Parse a base64-encoded transaction envelope into structured data
    pub fn parse_transaction(&self, base64_data: &str) -> Result<ParsedTransaction, TransactionParseError> {
        // Decode base64 using no-std compatible base64 decoder
        let xdr_bytes = self.decode_base64(base64_data)?;
        
        // Parse XDR envelope
        let envelope = TransactionEnvelope::from_xdr(&xdr_bytes, stellar_xdr::curr::Limits::none())
            .map_err(|_| TransactionParseError::XdrError)?;
        
        match envelope {
            TransactionEnvelope::TxV0(env) => self.parse_v0_transaction(env),
            TransactionEnvelope::Tx(env) => self.parse_v1_transaction(env),
            TransactionEnvelope::TxFeeBump(env) => self.parse_fee_bump_transaction(env),
        }
    }

    fn decode_base64(&self, input: &str) -> Result<Vec<u8>, TransactionParseError> {
        // Simple base64 decoder for no-std
        let mut dec_buf = [0u8; MAX_XDR_LEN]; // Allocate enough space
        let decoded = Base64::decode(input, &mut dec_buf).unwrap();
        printk!("Decoded base64 length: {}\n", decoded.len());
        Ok(decoded.to_vec())
    }

    fn uint256_to_bounded_hex(&self, v: &Uint256) -> Result<BoundedString, TransactionParseError> {
        self.bytes_to_hex_string(&v.0)
    }

    fn asset_code_to_bounded_string(&self, code: &AssetCode) -> Result<BoundedString, TransactionParseError> {
        match code {
            AssetCode::CreditAlphanum4(c) => {
                let code_bytes = &c.0;
                let code_str = core::str::from_utf8(code_bytes).unwrap_or("invalid");
                let trimmed_code = code_str.trim_end_matches('\0');
                BoundedString::try_from(trimmed_code).map_err(|_| TransactionParseError::StringTooLong)
            },
            AssetCode::CreditAlphanum12(c) => {
                let code_bytes = &c.0;
                let code_str = core::str::from_utf8(code_bytes).unwrap_or("invalid");
                let trimmed_code = code_str.trim_end_matches('\0');
                BoundedString::try_from(trimmed_code).map_err(|_| TransactionParseError::StringTooLong)
            },
        }
    }

    fn parse_v0_transaction(&self, envelope: TransactionV0Envelope) -> Result<ParsedTransaction, TransactionParseError> {
        let tx = envelope.tx;
        
        let time_bounds = tx.time_bounds
            .as_ref()
            .map(|tb| self.parse_time_bounds(tb))
            .transpose()?;
        let ledger_bounds = None; // V0 transactions do not have ledger bounds
        
        Ok(ParsedTransaction {
            source_account: { self.uint256_to_bounded_hex(&tx.source_account_ed25519)? },
            sequence_number: tx.seq_num.0,
            fee: tx.fee as i64,
            memo: Some(self.parse_memo(&tx.memo)?),
            time_bounds,
            ledger_bounds,
            operations: self.parse_operations(&tx.operations)?,
            signatures: self.parse_signatures(&envelope.signatures)?,
            envelope_type: TransactionEnvelopeType::TxV0,
        })
    }

    fn parse_v1_transaction(&self, envelope: TransactionV1Envelope) -> Result<ParsedTransaction, TransactionParseError> {
        printk!("Parsing V1 transaction envelope...\n");
        let tx = envelope.tx;
        printk!("Parsing V1 transaction...\n");

        let (time_bounds, ledger_bounds) = self.parse_preconditions(&tx.cond)?;
        
        Ok(ParsedTransaction {
            source_account: self.muxed_account_to_bounded_string(&tx.source_account)?,
            sequence_number: tx.seq_num.0,
            fee: tx.fee as i64,
            memo: Some(self.parse_memo(&tx.memo)?),
            time_bounds,
            ledger_bounds,
            operations: self.parse_operations(&tx.operations)?,
            signatures: self.parse_signatures(&envelope.signatures)?,
            envelope_type: TransactionEnvelopeType::Tx,
        })
    }

    fn parse_fee_bump_transaction(&self, envelope: FeeBumpTransactionEnvelope) -> Result<ParsedTransaction, TransactionParseError> {
        let fee_bump = envelope.tx;
        printk!("Parsing Fee Bump transaction...\n");

        // Extract the inner transaction
        let inner_envelope = match fee_bump.inner_tx {
            FeeBumpTransactionInnerTx::Tx(env) => env,
        };

        let mut parsed = self.parse_v1_transaction(inner_envelope)?;
        
        // Override with fee bump details
        parsed.source_account = self.muxed_account_to_bounded_string(&fee_bump.fee_source)?;
        parsed.fee = fee_bump.fee;
        parsed.signatures = self.parse_signatures(&envelope.signatures)?;
        parsed.envelope_type = TransactionEnvelopeType::TxFeeBump;
        
        Ok(parsed)
    }

    fn parse_memo(&self, memo: &Memo) -> Result<ParsedMemo, TransactionParseError> {
        printk!("Parsing memo: {:?}\n", memo);
        let parsed_memo = match memo {
            Memo::None => ParsedMemo {
                memo_type: BoundedString::try_from("none").map_err(|_| TransactionParseError::StringTooLong)?,
                value: None,
            },
            Memo::Text(text) => {
                let text_str = core::str::from_utf8(&text).unwrap_or("invalid_utf8");
                ParsedMemo {
                    memo_type: BoundedString::try_from("text").map_err(|_| TransactionParseError::StringTooLong)?,
                    value: Some(BoundedString::try_from(text_str).map_err(|_| TransactionParseError::StringTooLong)?),
                }
            },
            Memo::Id(id) => ParsedMemo {
                memo_type: BoundedString::try_from("id").map_err(|_| TransactionParseError::StringTooLong)?,
                value: Some(self.u64_to_bounded_string(id.clone())?),
            },
            Memo::Hash(hash) => ParsedMemo {
                memo_type: BoundedString::try_from("hash").map_err(|_| TransactionParseError::StringTooLong)?,
                value: Some(self.bytes_to_hex_string(&hash.0)?),
            },
            Memo::Return(ret) => ParsedMemo {
                memo_type: BoundedString::try_from("return").map_err(|_| TransactionParseError::StringTooLong)?,
                value: Some(self.bytes_to_hex_string(&ret.0)?),
            },
        };
        printk!("Parsed memo: {:?}\n", parsed_memo);
        
        Ok(parsed_memo)
    }

    fn parse_preconditions(&self, preconditions: &Preconditions) -> Result<(Option<ParsedTimeBounds>, Option<ParsedLedgerBounds>), TransactionParseError> {
        match preconditions {
            Preconditions::None => Ok((None, None)),
            Preconditions::Time(time_bounds) => Ok((Some(self.parse_time_bounds(time_bounds)?), None)),
            Preconditions::V2(preconditions_v2) => {
                let time_bounds = preconditions_v2.time_bounds
                    .as_ref()
                    .map(|tb| self.parse_time_bounds(tb))
                    .transpose()?;
                let ledger_bounds = preconditions_v2.ledger_bounds
                    .as_ref()
                    .map(|lb| self.parse_ledger_bounds(lb))
                    .transpose()?;
                Ok((time_bounds, ledger_bounds))
            }
        }
    }
    
    fn parse_time_bounds(&self, time_bounds: &TimeBounds) -> Result<ParsedTimeBounds, TransactionParseError> {
        Ok(ParsedTimeBounds {
            min_time: Some(time_bounds.min_time.0),
            max_time: Some(time_bounds.max_time.0),
        })
    }

    fn parse_ledger_bounds(&self, ledger_bounds: &LedgerBounds) -> Result<ParsedLedgerBounds, TransactionParseError> {
        Ok(ParsedLedgerBounds {
            min_ledger: ledger_bounds.min_ledger,
            max_ledger: ledger_bounds.max_ledger,
        })
    }

    fn parse_operations(&self, operations: &[Operation]) -> Result<BoundedVec<ParsedOperation>, TransactionParseError> {
        let mut parsed_ops = BoundedVec::new();
        
        for operation in operations.iter() {
            let parsed_op = self.parse_operation(operation)?;
            parsed_ops.push(parsed_op).map_err(|_| TransactionParseError::TooManyOperations)?;
        }
        
        Ok(parsed_ops)
    }

    fn parse_operation(&self, operation: &Operation) -> Result<ParsedOperation, TransactionParseError> {
        let source_account = match &operation.source_account {
            Some(account) => Some(self.muxed_account_to_bounded_string(account)?),
            None => None,
        };
        
        let operation_type = match &operation.body {
            OperationBody::CreateAccount(create_account) => (
                BoundedString::try_from("create_account").map_err(|_| TransactionParseError::StringTooLong)?,
                OperationDetails::CreateAccount {
                    destination: self.account_id_to_bounded_string(&create_account.destination)?,
                    starting_balance: create_account.starting_balance,
                }
            ),
            OperationBody::Payment(payment) => (
                BoundedString::try_from("payment").map_err(|_| TransactionParseError::StringTooLong)?,
                OperationDetails::Payment {
                    destination: self.parse_muxed_account(&payment.destination)?,
                    asset: self.parse_asset(&payment.asset)?,
                    amount: payment.amount,
                }
            ),
            OperationBody::PathPaymentStrictSend(ppss) => (
                BoundedString::try_from("path_payment_strict_send").map_err(|_| TransactionParseError::StringTooLong)?,
                OperationDetails::PathPaymentStrictSend {
                    send_asset: self.parse_asset(&ppss.send_asset)?,
                    send_amount: ppss.send_amount,
                    destination: self.parse_muxed_account(&ppss.destination)?,
                    dest_asset: self.parse_asset(&ppss.dest_asset)?,
                    dest_min: ppss.dest_min,
                    path: self.parse_assets_path(&ppss.path)?,
                }
            ),
            OperationBody::PathPaymentStrictReceive(ppss) => (
                BoundedString::try_from("path_payment_strict_send").map_err(|_| TransactionParseError::StringTooLong)?,
                OperationDetails::PathPaymentStrictReceive {
                    send_asset: self.parse_asset(&ppss.send_asset)?,
                    send_max: ppss.send_max,
                    destination: self.parse_muxed_account(&ppss.destination)?,
                    dest_asset: self.parse_asset(&ppss.dest_asset)?,
                    dest_amount: ppss.dest_amount,
                    path: self.parse_assets_path(&ppss.path)?,
                }
            ),
            OperationBody::ChangeTrust(change_trust) => (
                BoundedString::try_from("change_trust").map_err(|_| TransactionParseError::StringTooLong)?,
                OperationDetails::ChangeTrust {
                    asset: self.parse_change_trust_asset(&change_trust.line)?,
                    limit: change_trust.limit,
                }
            ),
            OperationBody::AccountMerge(muxed_account) => (
                BoundedString::try_from("account_merge").map_err(|_| TransactionParseError::StringTooLong)?,
                OperationDetails::AccountMerge {
                    destination: self.muxed_account_to_bounded_string(muxed_account)?,
                }
            ),
            OperationBody::AllowTrust(allow_trust) => (
                BoundedString::try_from("allow_trust").map_err(|_| TransactionParseError::StringTooLong)?,
                OperationDetails::AllowTrust {
                    trustor: self.account_id_to_bounded_string(&allow_trust.trustor)?,
                    asset_code: self.asset_code_to_bounded_string(&allow_trust.asset)?,
                    authorize: allow_trust.authorize,
                }
            ),
            OperationBody::SetTrustLineFlags(set_trust_line_flags) => (
                BoundedString::try_from("set_trust_line_flags").map_err(|_| TransactionParseError::StringTooLong)?,
                OperationDetails::SetTrustLineFlags {
                    trustor: self.account_id_to_bounded_string(&set_trust_line_flags.trustor)?,
                    asset: self.parse_asset(&set_trust_line_flags.asset)?,
                    clear_flags: set_trust_line_flags.clear_flags,
                    set_flags: set_trust_line_flags.set_flags,
                }
            ),
            _ => (
                BoundedString::try_from("other").map_err(|_| TransactionParseError::StringTooLong)?,
                OperationDetails::Other {
                    operation_type: BoundedString::try_from("unknown").map_err(|_| TransactionParseError::StringTooLong)?,
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

    fn parse_assets_path(&self, path: &[Asset]) -> Result<BoundedAssetVec, TransactionParseError> {
        let mut parsed_assets = BoundedAssetVec::new();
        
        for asset in path.iter() {
            let parsed_asset = self.parse_asset(asset)?;
            parsed_assets.push(parsed_asset).map_err(|_| TransactionParseError::TooManyOperations)?;
        }
        
        Ok(parsed_assets)
    }

    fn parse_muxed_account(&self, muxed_account: &MuxedAccount) -> Result<ParsedMuxedAccount, TransactionParseError> {
        match muxed_account {
            MuxedAccount::Ed25519(account_id) => Ok(ParsedMuxedAccount::Ed25519 {
                account_id: self.bytes_to_hex_string(&account_id.0)?,
            }),
            MuxedAccount::MuxedEd25519(muxed) => Ok(ParsedMuxedAccount::MuxedEd25519 {
                id: muxed.id,
                account_id: self.bytes_to_hex_string(&muxed.ed25519.0)?,
            }),
        }
    }

    fn parse_asset(&self, asset: &Asset) -> Result<ParsedAsset, TransactionParseError> {
        let parsed_asset = match asset {
            Asset::Native => ParsedAsset::Native {
                // asset_type: BoundedString::try_from("native").map_err(|_| TransactionParseError::StringTooLong)?,
                // asset_code: None,
                // issuer: None,
            },
            Asset::CreditAlphanum4(alpha4) => {
                let code_bytes = &alpha4.asset_code.0;
                let code_str = core::str::from_utf8(code_bytes).unwrap_or("invalid");
                let trimmed_code = code_str.trim_end_matches('\0');
                
                ParsedAsset::CreditAlphanum4 {
                    code: HeaplessString::try_from(trimmed_code).map_err(|_| TransactionParseError::StringTooLong)?,
                    issuer: self.account_id_to_bounded_string(&alpha4.issuer)?,
                }
            },
            Asset::CreditAlphanum12(alpha12) => {
                let code_bytes = &alpha12.asset_code.0;
                let code_str = core::str::from_utf8(code_bytes).unwrap_or("invalid");
                let trimmed_code = code_str.trim_end_matches('\0');

                ParsedAsset::CreditAlphanum12 {
                    code: HeaplessString::try_from(trimmed_code).map_err(|_| TransactionParseError::StringTooLong)?,
                    issuer: self.account_id_to_bounded_string(&alpha12.issuer)?,
                }
            },
        };
        
        Ok(parsed_asset)
    }

    fn parse_change_trust_asset(&self, asset: &ChangeTrustAsset) -> Result<ParsedChangeTrustAsset, TransactionParseError> {
        let parsed_asset = match asset {
            ChangeTrustAsset::Native => ParsedChangeTrustAsset::Native,
            ChangeTrustAsset::CreditAlphanum4(alpha4) => {
                let code_bytes = &alpha4.asset_code.0;
                let code_str = core::str::from_utf8(code_bytes).unwrap_or("invalid");
                let trimmed_code = code_str.trim_end_matches('\0');
                
                ParsedChangeTrustAsset::CreditAlphanum4 {
                    code: HeaplessString::try_from(trimmed_code).map_err(|_| TransactionParseError::StringTooLong)?,
                    issuer: self.account_id_to_bounded_string(&alpha4.issuer)?,
                }
            },
            ChangeTrustAsset::CreditAlphanum12(alpha12) => {
                let code_bytes = &alpha12.asset_code.0;
                let code_str = core::str::from_utf8(code_bytes).unwrap_or("invalid");
                let trimmed_code = code_str.trim_end_matches('\0');

                ParsedChangeTrustAsset::CreditAlphanum12 {
                    code: HeaplessString::try_from(trimmed_code).map_err(|_| TransactionParseError::StringTooLong)?,
                    issuer: self.account_id_to_bounded_string(&alpha12.issuer)?,
                }
            },
            ChangeTrustAsset::PoolShare(lp) => {
                match lp {
                    LiquidityPoolParameters::LiquidityPoolConstantProduct(cp) => {
                        let asset_a = self.parse_asset(&cp.asset_a)?;
                        let asset_b = self.parse_asset(&cp.asset_b)?;
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

    fn parse_signatures(&self, signatures: &[DecoratedSignature]) -> Result<BoundedSignatureVec, TransactionParseError> {
        let mut parsed_sigs = BoundedSignatureVec::new();
        
        for sig in signatures.iter() {
            let mut hint = HeaplessVec::new();
            let mut signature = HeaplessVec::new();
            
            // Copy hint (always 4 bytes)
            for &byte in sig.hint.0.iter() {
                hint.push(byte).map_err(|_| TransactionParseError::DataTooLarge)?;
            }
            
            // Copy signature (up to 64 bytes for Ed25519)
            for &byte in sig.signature.0.iter() {
                signature.push(byte).map_err(|_| TransactionParseError::DataTooLarge)?;
            }
            
            let parsed_sig = ParsedSignature { hint, signature };
            parsed_sigs.push(parsed_sig).map_err(|_| TransactionParseError::TooManySignatures)?;
        }
        
        Ok(parsed_sigs)
    }

    fn account_id_to_bounded_string(
        &self,
        account_id: &AccountId,
    ) -> Result<BoundedString, TransactionParseError> {
        match &account_id.0 {
            PublicKey::PublicKeyTypeEd25519(ed) => self.bytes_to_hex_string(&ed.0),
        }
    }

    fn muxed_account_to_bounded_string(&self, muxed_account: &MuxedAccount) -> Result<BoundedString, TransactionParseError> {
        printk!("Parsing muxed account: {:?}\n", muxed_account);
        match muxed_account {
            MuxedAccount::Ed25519(account_id) => self.bytes_to_hex_string(&account_id.0),
            MuxedAccount::MuxedEd25519(muxed) => {
                // For muxed accounts, we'd normally encode both the account and ID
                // For simplicity, just return the account part as hex
                self.bytes_to_hex_string(&muxed.ed25519.0)
            }
        }
    }

    /// Convert stroops to XLM string representation with bounded string
    fn stroops_to_xlm_bounded(&self, stroops: i64) -> Result<BoundedString, TransactionParseError> {
        // Simple conversion avoiding floating point in no-std
        let whole_xlm = stroops / 10_000_000;
        let fractional = stroops % 10_000_000;
        
        if fractional == 0 {
            self.i64_to_bounded_string(whole_xlm)
        } else {
            // For simplicity, just return the stroops value as string in no-std
            self.i64_to_bounded_string(stroops)
        }
    }

    fn u64_to_bounded_string(&self, value: u64) -> Result<BoundedString, TransactionParseError> {
        let mut buffer = [0u8; 20]; // u64 max is 20 digits
        let mut i = buffer.len();
        let mut n = value;
        
        if n == 0 {
            return BoundedString::try_from("0").map_err(|_| TransactionParseError::StringTooLong);
        }
        
        while n > 0 {
            i -= 1;
            buffer[i] = b'0' + (n % 10) as u8;
            n /= 10;
        }
        
        let str_slice = core::str::from_utf8(&buffer[i..]).map_err(|_| TransactionParseError::StringTooLong)?;
        BoundedString::try_from(str_slice).map_err(|_| TransactionParseError::StringTooLong)
    }

    fn i64_to_bounded_string(&self, value: i64) -> Result<BoundedString, TransactionParseError> {
        if value >= 0 {
            self.u64_to_bounded_string(value as u64)
        } else {
            let pos_str = self.u64_to_bounded_string((-value) as u64)?;
            let mut result = BoundedString::new();
            result.push('-').map_err(|_| TransactionParseError::StringTooLong)?;
            result.push_str(&pos_str).map_err(|_| TransactionParseError::StringTooLong)?;
            Ok(result)
        }
    }

    fn bytes_to_hex_string(&self, bytes: &[u8]) -> Result<BoundedString, TransactionParseError> {
        const HEX_CHARS: &[u8] = b"0123456789abcdef";
        let mut result = BoundedString::new();
        
        for &byte in bytes.iter() {
            let high = (byte >> 4) as usize;
            let low = (byte & 0x0f) as usize;
            result.push(HEX_CHARS[high] as char).map_err(|_| TransactionParseError::StringTooLong)?;
            result.push(HEX_CHARS[low] as char).map_err(|_| TransactionParseError::StringTooLong)?;
        }
        
        Ok(result)
    }
}

impl Default for StellarTransactionParser {
    fn default() -> Self {
        Self::new()
    }
}