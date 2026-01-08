#![no_std]

extern crate alloc;
use alloc::{string::String, vec::Vec, format};

use base64ct::{Base64, Decoder, Encoding};

use core::{convert::TryInto, hash};

struct Xdr<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> Xdr<'a> {
    fn new(buf: &'a [u8]) -> Self {
        Self { buf, pos: 0 }
    }

    fn remaining(&self) -> usize {
        self.buf.len().saturating_sub(self.pos)
    }

    fn read_exact(&mut self, len: usize) -> Result<&'a [u8], TransactionParseError> {
        if self.pos + len > self.buf.len() {
            return Err(TransactionParseError::XdrError);
        }
        let out = &self.buf[self.pos..self.pos + len];
        self.pos += len;
        Ok(out)
    }

    fn align4(&mut self) {
        let r = self.pos & 3;
        if r != 0 {
            self.pos += 4 - r;
        }
    }

    fn read_u32(&mut self) -> Result<u32, TransactionParseError> {
        let b = self.read_exact(4)?;
        Ok(u32::from_be_bytes(b.try_into().unwrap()))
    }

    fn read_i32(&mut self) -> Result<i32, TransactionParseError> {
        Ok(self.read_u32()? as i32)
    }

    fn read_u64(&mut self) -> Result<u64, TransactionParseError> {
        let hi = self.read_u32()? as u64;
        let lo = self.read_u32()? as u64;
        Ok((hi << 32) | lo)
    }

    fn read_i64(&mut self) -> Result<i64, TransactionParseError> {
        Ok(self.read_u64()? as i64)
    }

    fn read_bool(&mut self) -> Result<bool, TransactionParseError> {
        match self.read_u32()? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(TransactionParseError::XdrError),
        }
    }

    fn read_fixed_opaque(&mut self, len: usize) -> Result<&'a [u8], TransactionParseError> {
        // fixed-length opaque is NOT length-prefixed in Stellar XDR, just raw bytes.
        self.read_exact(len)
    }

    fn read_opaque(&mut self, max_len: usize) -> Result<&'a [u8], TransactionParseError> {
        let len = self.read_u32()? as usize;
        if len > max_len {
            return Err(TransactionParseError::XdrError);
        }
        let data = self.read_exact(len)?;
        self.align4();
        Ok(data)
    }

    fn read_string(&mut self, max_len: usize) -> Result<&'a str, TransactionParseError> {
        let bytes = self.read_opaque(max_len)?;
        core::str::from_utf8(bytes).map_err(|_| TransactionParseError::XdrError)
    }
}

// EnvelopeType (from Stellar XDR)
const ENVELOPE_TYPE_TX_V0: i32       = 0;
const ENVELOPE_TYPE_TX: i32          = 2;
const ENVELOPE_TYPE_TX_FEE_BUMP: i32 = 5;

// MemoType
const MEMO_NONE: i32   = 0;
const MEMO_TEXT: i32   = 1;
const MEMO_ID: i32     = 2;
const MEMO_HASH: i32   = 3;
const MEMO_RETURN: i32 = 4;

// AssetType
const ASSET_TYPE_NATIVE: i32            = 0;
const ASSET_TYPE_CREDIT_ALPHANUM4: i32  = 1;
const ASSET_TYPE_CREDIT_ALPHANUM12: i32 = 2;
const ASSET_TYPE_POOL_SHARE: i32        = 3;

// OperationType
const OP_CREATE_ACCOUNT: i32           = 0;
const OP_PAYMENT: i32                  = 1;
const OP_PATH_PAYMENT_STRICT_RECEIVE: i32 = 2;
const OP_CHANGE_TRUST: i32             = 6;
const OP_ALLOW_TRUST: i32              = 7;
const OP_ACCOUNT_MERGE: i32            = 8;
const OP_PATH_PAYMENT_STRICT_SEND: i32 = 13;
const OP_SET_TRUST_LINE_FLAGS: i32     = 21;

// Preconditions types
const PRECOND_NONE: i32 = 0;
const PRECOND_TIME: i32 = 1;
const PRECOND_V2: i32   = 2;

// CryptoKeyType
const KEY_TYPE_ED25519: i32         = 0;
const KEY_TYPE_MUXED_ED25519: i32   = 256;

pub const NETWORK_ID_MAINNET: &str = "7ac33997544e3175d266bd022439b22cdb16508c01163f26e5cb2a3e1045a979";
pub const NETWORK_ID_TESTNET: &str = "cee0302d59844d32bdca915c8203dd44b33fbb7edc19051ea37abedf28ecd472";
pub const NETWORK_ID_FUTURENET: &str = "a3a1c6a78286713e29be0e9785670fa838d13917cd8eaeb4a3579ff1debc7fd5";

const NETWORK_IDS : [&str; 3] = [
    NETWORK_ID_MAINNET,
    NETWORK_ID_TESTNET,
    NETWORK_ID_FUTURENET,
];

use crate::{crypto::{crypt0::{bytes_to_hex, hex_to_bytes}, libcrypt0pro::stellar::StellarWallet}, log_info, printk};


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
    pub network_hash: Option<Network>,
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
pub struct ParsedFeeBumpTransaction {
  pub fee_source: ParsedMuxedAccount,
  pub fee: i64,
  pub inner_tx: ParsedTransaction,
  pub signatures: Vec<ParsedSignature>,
}

#[derive(Debug, Clone)]
pub enum TransactionEnvelope {
    Transaction(ParsedTransaction),
    FeeBump(ParsedFeeBumpTransaction),
}

#[derive(Debug, Clone)]
enum NetworkId {
    Mainnet,
    Testnet,
    Futurenet,
}

#[derive(Debug, Clone)]
pub struct Network {
    id: NetworkId,
    hash: String,
}

impl Network {
  pub fn new(id: NetworkId, hash_str: String) -> Self {
      Self {
          id,
          hash: hash_str,
      }
  }
  pub fn get_hash_bytes(&self) -> [u8; 32] {
      let mut hash_bytes = [0u8; 32];
      let decoded = hex_to_bytes(&self.hash).unwrap();
      hash_bytes.copy_from_slice(&decoded[..32]);
      hash_bytes
  }

  pub fn get_hash_hex(&self) -> &str {
      &self.hash
  }
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
    pub hint: String, // Signature hints are always 4 bytes
    pub signature: String, // Ed25519 signatures are 64 bytes
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
    InvalidPrefix,
    InvalidNetworkId,
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
            TransactionParseError::InvalidPrefix => write!(f, "Invalid transaction format prefix"),
            TransactionParseError::InvalidNetworkId => write!(f, "Invalid network ID"),
        }
    }
}

pub struct StellarTransactionParser;

impl StellarTransactionParser {
    pub fn new() -> Self {
        Self
    }

    fn decode_base64(base64_data: &str) -> Result<Vec<u8>, TransactionParseError> {
        let mut buf = [0u8; MAX_XDR_LEN * 4 / 3 + 4]; // Base64 expansion ratio
        //log_info!("Decoding base64 data: {}", base64_data);
        let encoded = Base64::decode(base64_data.as_bytes(), &mut buf)
            .map_err(|_| TransactionParseError::Base64Error)?;
        //log_info!("Decoded base64 length: {}", encoded.len());
        Ok(encoded.to_vec())
    }

    fn parse_fee_bump_transaction_xdr(xdr_bytes: &[u8], network_hash: &str) -> Result<ParsedFeeBumpTransaction, TransactionParseError> {
        if xdr_bytes.len() > MAX_XDR_LEN {
            return Err(TransactionParseError::PayloadTooLarge);
        }
        if xdr_bytes.len() < 20 {
            return Err(TransactionParseError::PayloadTooSmall);
        }

        let mut xr = Xdr::new(xdr_bytes);

        let envelope_type = xr.read_i32()?;
        if envelope_type != ENVELOPE_TYPE_TX_FEE_BUMP {
            return Err(TransactionParseError::InvalidEnvelopeType);
        }

        let fee_bump_tx = Self::parse_fee_bump_envelope(&mut xr);
        let mut parsed_tx = fee_bump_tx?;
        use alloc::string::ToString;
        parsed_tx.inner_tx.network_hash = Some(Network::new(
            match network_hash {
                NETWORK_ID_MAINNET => NetworkId::Mainnet,
                NETWORK_ID_TESTNET => NetworkId::Testnet,
                NETWORK_ID_FUTURENET => NetworkId::Futurenet,
                _ => return Err(TransactionParseError::InvalidNetworkId),
            },
            network_hash.to_string(),
        ));
        Ok(parsed_tx)
    }

    fn parse_transaction_xdr(xdr_bytes: &[u8], network_hash: &str) -> Result<ParsedTransaction, TransactionParseError> {
        if xdr_bytes.len() > MAX_XDR_LEN {
            return Err(TransactionParseError::PayloadTooLarge);
        }
        if xdr_bytes.len() < 20 {
            return Err(TransactionParseError::PayloadTooSmall);
        }

        let mut xr = Xdr::new(xdr_bytes);

        let envelope_type = xr.read_i32()?;
        let tx = match envelope_type {
            ENVELOPE_TYPE_TX_V0 => Self::parse_v0_envelope(&mut xr),
            ENVELOPE_TYPE_TX => Self::parse_v1_envelope(&mut xr),
            //ENVELOPE_TYPE_TX_FEE_BUMP => Self::parse_fee_bump_envelope(&mut xr),
            _ => Err(TransactionParseError::InvalidEnvelopeType),
        };
        let mut parsed_tx = tx?;
        use alloc::string::ToString;
        parsed_tx.network_hash = Some(Network::new(
            match network_hash {
                NETWORK_ID_MAINNET => NetworkId::Mainnet,
                NETWORK_ID_TESTNET => NetworkId::Testnet,
                NETWORK_ID_FUTURENET => NetworkId::Futurenet,
                _ => return Err(TransactionParseError::InvalidNetworkId),
            },
            network_hash.to_string(),
        ));
        Ok(parsed_tx)
    }

    /// Parse a base64-encoded transaction envelope into structured data
    pub fn parse_transaction(tx_data: &str, network_hash: &str) -> Result<TransactionEnvelope, TransactionParseError> {
        let xdr_bytes = Self::decode_base64(tx_data)?;
        
        // Peek at envelope type to determine which parser to use
        if xdr_bytes.len() < 4 {
            return Err(TransactionParseError::PayloadTooSmall);
        }
        let envelope_type = i32::from_be_bytes([xdr_bytes[0], xdr_bytes[1], xdr_bytes[2], xdr_bytes[3]]);
        
        match envelope_type {
            ENVELOPE_TYPE_TX_FEE_BUMP => {
                let fee_bump = Self::parse_fee_bump_transaction_xdr(&xdr_bytes, network_hash)?;
                Ok(TransactionEnvelope::FeeBump(fee_bump))
            }
            _ => {
                let tx = Self::parse_transaction_xdr(&xdr_bytes, network_hash)?;
                Ok(TransactionEnvelope::Transaction(tx))
            }
        }
    }

    fn uint256_to_bounded_hex(v: &[u8]) -> Result<String, TransactionParseError> {
        let hex = bytes_to_hex(v);
        if hex.len() > MAX_STRING_LEN {
            return Err(TransactionParseError::StringTooLong);
        }
        Ok(hex)
    }

    fn u8_array_to_stellar_address(key: &[u8]) -> Result<String, TransactionParseError> {
        // let hex = Self::uint256_to_bounded_hex(key);
        // if hex.is_err() {
        //     return Err(TransactionParseError::AccountIdError);
        // }
        let mut key_array = [0u8; 32];
        key_array.copy_from_slice(key);
        let address = StellarWallet::encode_stellar_address(&key_array);
        if address.is_err() {
            return Err(TransactionParseError::AccountIdError);
        }
        let address = address.unwrap();
        Ok(address)
    }

    fn parse_v0_envelope(xr: &mut Xdr) -> Result<ParsedTransaction, TransactionParseError> {
        // TransactionV0
        let source = xr.read_fixed_opaque(32)?; // uint256
        let fee = xr.read_u32()? as i64;
        let seq_num = xr.read_i64()?;

        // TimeBounds* timeBounds;
        let has_tb = xr.read_bool()?;
        let time_bounds = if has_tb {
            Some(Self::parse_time_bounds(xr)?)
        } else {
            None
        };

        let memo = Self::parse_memo(xr)?;

        let operations = Self::parse_operations(xr)?;

        // ext (int v = 0; void)
        let _ext = xr.read_i32()?; // should be 0

        let signatures = Self::parse_signatures(xr)?;
        
        Ok(ParsedTransaction {
            network_hash: None,
            source_account: Self::u8_array_to_stellar_address(source)?,
            sequence_number: seq_num,
            fee,
            memo: Some(memo),
            time_bounds,
            ledger_bounds: None,
            operations,
            signatures,
            envelope_type: TransactionEnvelopeType::TxV0,
        })
    }

    fn parse_v1_envelope(xr: &mut Xdr) -> Result<ParsedTransaction, TransactionParseError> {
        // Transaction
        let source = Self::parse_muxed_account(xr)?;
        let fee = xr.read_u32()? as i64;
        let seq_num = xr.read_i64()?;

        let (time_bounds, ledger_bounds) = Self::parse_preconditions(xr)?;

        let memo = Self::parse_memo(xr)?;
        let operations = Self::parse_operations(xr)?;

        // tx ext (int v; case 0: void)
        let _ext = xr.read_i32()?; // 0

        let signatures = Self::parse_signatures(xr)?;

        Ok(ParsedTransaction {
            network_hash: None,
            source_account: Self::muxed_account_to_string(&source)?,
            sequence_number: seq_num,
            fee,
            memo: Some(memo),
            time_bounds,
            ledger_bounds,
            operations,
            signatures,
            envelope_type: TransactionEnvelopeType::Tx,
        })
    }

    fn parse_fee_bump_envelope(xr: &mut Xdr) -> Result<ParsedFeeBumpTransaction, TransactionParseError> {
        // FeeBumpTransaction
        printk!("Parsing fee bump transaction envelope");
        let fee_source = Self::parse_muxed_account(xr)?;
        let fee = xr.read_i64()?;

        // innerTx
        let inner_type = xr.read_i32()?;
        if inner_type != ENVELOPE_TYPE_TX {
            return Err(TransactionParseError::InvalidEnvelopeType);
        }

        // Inner TransactionV1Envelope (tx + signatures)
        // Reuse same logic as v1 envelope, but we need to parse into temp first
        let inner_tx = {
            let source = Self::parse_muxed_account(xr)?;
            let fee_inner = xr.read_u32()? as i64;
            let seq_num = xr.read_i64()?;
            let (time_bounds, ledger_bounds) = Self::parse_preconditions(xr)?;
            let memo = Self::parse_memo(xr)?;
            let operations = Self::parse_operations(xr)?;
            let _ext = xr.read_i32()?; // tx ext v=0
            let inner_signatures = Self::parse_signatures(xr)?;

            ParsedTransaction {
                network_hash: None,
                source_account: Self::muxed_account_to_string(&source)?,
                sequence_number: seq_num,
                fee: fee_inner,
                memo: Some(memo),
                time_bounds,
                ledger_bounds,
                operations,
                signatures: inner_signatures,
                envelope_type: TransactionEnvelopeType::Tx,
            }
        };

        // fee bump ext
        let _ext = xr.read_i32()?; // 0

        let outer_sigs = Self::parse_signatures(xr)?;

        Ok(ParsedFeeBumpTransaction {
            fee_source,
            fee,
            inner_tx,
            signatures: outer_sigs,
        })
    }


    fn parse_memo(xr: &mut Xdr) -> Result<ParsedMemo, TransactionParseError> {
        let t = xr.read_i32()?;
        match t {
            MEMO_NONE => Ok(ParsedMemo {
                memo_type: String::try_from("none").map_err(|_| TransactionParseError::StringTooLong)?,
                value: None,
            }),
            MEMO_TEXT => {
                let s = xr.read_string(MAX_STRING_LEN)?;
                Ok(ParsedMemo {
                    memo_type: String::try_from("text").map_err(|_| TransactionParseError::StringTooLong)?,
                    value: Some(String::try_from(s).map_err(|_| TransactionParseError::StringTooLong)?),
                })
            }
            MEMO_ID => {
                let id = xr.read_u64()?;
                Ok(ParsedMemo {
                    memo_type: String::try_from("id").map_err(|_| TransactionParseError::StringTooLong)?,
                    value: Some(Self::u64_to_string(id)?),
                })
            }
            MEMO_HASH => {
                let h = xr.read_fixed_opaque(32)?;
                Ok(ParsedMemo {
                    memo_type: String::try_from("hash").map_err(|_| TransactionParseError::StringTooLong)?,
                    value: Some(bytes_to_hex(h)),
                })
            }
            MEMO_RETURN => {
                let h = xr.read_fixed_opaque(32)?;
                Ok(ParsedMemo {
                    memo_type: String::try_from("return").map_err(|_| TransactionParseError::StringTooLong)?,
                    value: Some(bytes_to_hex(h)),
                })
            }
            _ => Err(TransactionParseError::XdrError),
        }
    }

    fn parse_preconditions(xr: &mut Xdr) -> Result<(Option<ParsedTimeBounds>, Option<ParsedLedgerBounds>), TransactionParseError> {
        let t = xr.read_i32()?;
        match t {
            PRECOND_NONE => Ok((None, None)),
            PRECOND_TIME => {
                let tb = Self::parse_time_bounds(xr)?;
                Ok((Some(tb), None))
            }
            PRECOND_V2 => {
                let has_tb = xr.read_bool()?;
                let tb = if has_tb {
                    Some(Self::parse_time_bounds(xr)?)
                } else {
                    None
                };
                let has_lb = xr.read_bool()?;
                let lb = if has_lb {
                    Some(Self::parse_ledger_bounds(xr)?)
                } else {
                    None
                };
                // Skip minSeqNum*
                let has_min_seq = xr.read_bool()?;
                if has_min_seq {
                    let _ = xr.read_i64()?; // ignore
                }
                // minSeqAge, minSeqLedgerGap
                let _ = xr.read_u32()?;
                let _ = xr.read_u32()?;
                // extraSigners<2> - we skip them
                let count = xr.read_u32()? as usize;
                for _ in 0..count {
                    let key_type = xr.read_i32()?;
                    match key_type {
                        // All known signer key variants have 32-byte bodies here
                        _ => {
                            let _key = xr.read_fixed_opaque(32)?;
                        }
                    }
                }
                Ok((tb, lb))
            }
            _ => Err(TransactionParseError::XdrError),
        }
    }
    
    fn parse_time_bounds(xr: &mut Xdr) -> Result<ParsedTimeBounds, TransactionParseError> {
        let min_time = xr.read_u64()?;
        let max_time = xr.read_u64()?;
        Ok(ParsedTimeBounds {
            min_time: Some(min_time),
            max_time: Some(max_time),
        })
    }

    fn parse_ledger_bounds(xr: &mut Xdr) -> Result<ParsedLedgerBounds, TransactionParseError> {
        let min_ledger = xr.read_u32()?;
        let max_ledger = xr.read_u32()?;
        Ok(ParsedLedgerBounds {
            min_ledger,
            max_ledger,
        })
    }

    fn parse_operations(xr: &mut Xdr) -> Result<Vec<ParsedOperation>, TransactionParseError> {
        let count = xr.read_u32()? as usize;
        if count > MAX_OPERATIONS {
            return Err(TransactionParseError::TooManyOperations);
        }

        let mut ops = Vec::new();
        for _ in 0..count {
            ops.push(Self::parse_operation(xr)?);
        }
        Ok(ops)
    }

    fn parse_operation(xr: &mut Xdr) -> Result<ParsedOperation, TransactionParseError> {
        // sourceAccount*
        let has_src = xr.read_bool()?;
        let source_account = if has_src {
            let src = Self::parse_muxed_account(xr)?;
            Some(Self::muxed_account_to_string(&src)?)
        } else {
            None
        };

        let op_type = xr.read_i32()?;

        match op_type {
            OP_CREATE_ACCOUNT => {
                let dest = Self::account_id_to_string(xr)?;
                let starting_balance = xr.read_i64()?;
                Ok(ParsedOperation {
                    operation_type: String::from("create_account"),
                    source_account,
                    details: OperationDetails::CreateAccount {
                        destination: dest,
                        starting_balance,
                    },
                })
            }

            OP_PAYMENT => {
                let destination = Self::parse_muxed_account(xr)?;
                let asset = Self::parse_asset(xr)?;
                let amount = xr.read_i64()?;
                Ok(ParsedOperation {
                    operation_type: String::from("payment"),
                    source_account,
                    details: OperationDetails::Payment {
                        destination,
                        asset,
                        amount,
                    },
                })
            }

            OP_PATH_PAYMENT_STRICT_SEND => {
                let send_asset = Self::parse_asset(xr)?;
                let send_amount = xr.read_i64()?;
                let destination = Self::parse_muxed_account(xr)?;
                let dest_asset = Self::parse_asset(xr)?;
                let dest_min = xr.read_i64()?;
                let path = Self::parse_asset_path(xr)?;
                Ok(ParsedOperation {
                    operation_type: String::from("path_payment_strict_send"),
                    source_account,
                    details: OperationDetails::PathPaymentStrictSend {
                        send_asset,
                        send_amount,
                        destination,
                        dest_asset,
                        dest_min,
                        path,
                    },
                })
            }

            OP_PATH_PAYMENT_STRICT_RECEIVE => {
                let send_asset = Self::parse_asset(xr)?;
                let send_max = xr.read_i64()?;
                let destination = Self::parse_muxed_account(xr)?;
                let dest_asset = Self::parse_asset(xr)?;
                let dest_amount = xr.read_i64()?;
                let path = Self::parse_asset_path(xr)?;
                Ok(ParsedOperation {
                    operation_type: String::from("path_payment_strict_receive"),
                    source_account,
                    details: OperationDetails::PathPaymentStrictReceive {
                        send_asset,
                        send_max,
                        destination,
                        dest_asset,
                        dest_amount,
                        path,
                    },
                })
            }

            OP_CHANGE_TRUST => {
                let line = Self::parse_change_trust_asset(xr)?;
                let limit = xr.read_i64()?;
                Ok(ParsedOperation {
                    operation_type: String::from("change_trust"),
                    source_account,
                    details: OperationDetails::ChangeTrust { asset: line, limit },
                })
            }

            OP_ACCOUNT_MERGE => {
                // union body is just a MuxedAccount
                let dest = Self::parse_muxed_account(xr)?;
                Ok(ParsedOperation {
                    operation_type: String::from("account_merge"),
                    source_account,
                    details: OperationDetails::AccountMerge {
                        destination: Self::muxed_account_to_string(&dest)?,
                    },
                })
            }

            OP_ALLOW_TRUST => {
                let trustor = Self::account_id_to_string(xr)?;
                // asset code union
                let t = xr.read_i32()?;
                let asset_code: String = match t {
                    ASSET_TYPE_CREDIT_ALPHANUM4 => {
                        let code_bytes = xr.read_fixed_opaque(4)?;
                        let raw = core::str::from_utf8(code_bytes).unwrap_or("invalid");
                        let trimmed = raw.trim_end_matches('\0');
                        String::try_from(trimmed)
                            .map_err(|_| TransactionParseError::StringTooLong)?
                    }
                    ASSET_TYPE_CREDIT_ALPHANUM12 => {
                        let code_bytes = xr.read_fixed_opaque(12)?;
                        let raw = core::str::from_utf8(code_bytes).unwrap_or("invalid");
                        let trimmed = raw.trim_end_matches('\0');
                        String::try_from(trimmed)
                            .map_err(|_| TransactionParseError::StringTooLong)?
                    }
                    _ => return Err(TransactionParseError::UnsupportedOperation),
                };

                let authorize = xr.read_u32()?;

                Ok(ParsedOperation {
                    operation_type: String::try_from("allow_trust")
                        .map_err(|_| TransactionParseError::StringTooLong)?,
                    source_account,
                    details: OperationDetails::AllowTrust {
                        trustor,
                        asset_code,
                        authorize,
                    },
                })
            }



            OP_SET_TRUST_LINE_FLAGS => {
                let trustor = Self::account_id_to_string(xr)?;
                let asset = Self::parse_asset(xr)?;
                let clear_flags = xr.read_u32()?;
                let set_flags = xr.read_u32()?;
                Ok(ParsedOperation {
                    operation_type: String::from("set_trust_line_flags"),
                    source_account,
                    details: OperationDetails::SetTrustLineFlags {
                        trustor,
                        asset,
                        clear_flags,
                        set_flags,
                    },
                })
            }

            _ => Err(TransactionParseError::UnsupportedOperation),
        }
    }

    fn parse_asset_path(xr: &mut Xdr) -> Result<Vec<ParsedAsset>, TransactionParseError> {
        let count = xr.read_u32()? as usize;
        if count > MAX_PATH_ASSETS {
            return Err(TransactionParseError::PayloadTooLarge);
        }
        let mut v = Vec::new();
        for _ in 0..count {
            v.push(Self::parse_asset(xr)?);
        }
        Ok(v)
    }

    fn parse_muxed_account(xr: &mut Xdr) -> Result<ParsedMuxedAccount, TransactionParseError> {
        let t = xr.read_i32()?;
        match t {
            KEY_TYPE_ED25519 => {
                let key = xr.read_fixed_opaque(32)?;
                Ok(ParsedMuxedAccount::Ed25519 {
                    account_id: Self::u8_array_to_stellar_address(key)?,
                })
            }
            KEY_TYPE_MUXED_ED25519 => {
                let id = xr.read_u64()?;
                let key = xr.read_fixed_opaque(32)?;
                Ok(ParsedMuxedAccount::MuxedEd25519 {
                    id,
                    account_id: Self::u8_array_to_stellar_address(key)?,
                })
            }
            _ => Err(TransactionParseError::MuxedAccountError),
        }
    }

    fn parse_asset(xr: &mut Xdr) -> Result<ParsedAsset, TransactionParseError> {
        let t = xr.read_i32()?;
        match t {
            ASSET_TYPE_NATIVE => Ok(ParsedAsset::Native),
            ASSET_TYPE_CREDIT_ALPHANUM4 => {
                let code_bytes = xr.read_fixed_opaque(4)?;
                let code_str = core::str::from_utf8(code_bytes).unwrap_or("invalid");
                let trimmed = code_str.trim_end_matches('\0');
                let issuer = Self::account_id_to_string(xr)?;
                Ok(ParsedAsset::CreditAlphanum4 {
                    code: String::from(trimmed),
                    issuer,
                })
            }
            ASSET_TYPE_CREDIT_ALPHANUM12 => {
                let code_bytes = xr.read_fixed_opaque(12)?;
                let code_str = core::str::from_utf8(code_bytes).unwrap_or("invalid");
                let trimmed = code_str.trim_end_matches('\0');
                let issuer = Self::account_id_to_string(xr)?;
                Ok(ParsedAsset::CreditAlphanum12 {
                    code: String::from(trimmed),
                    issuer,
                })
            }
            _ => Err(TransactionParseError::UnsupportedOperation),
        }
    }

    fn parse_change_trust_asset(xr: &mut Xdr) -> Result<ParsedChangeTrustAsset, TransactionParseError> {
        let t = xr.read_i32()?;
        match t {
            ASSET_TYPE_NATIVE => Ok(ParsedChangeTrustAsset::Native),
            ASSET_TYPE_CREDIT_ALPHANUM4 => {
                let code_bytes = xr.read_fixed_opaque(4)?;
                let code_str = core::str::from_utf8(code_bytes).unwrap_or("invalid");
                let trimmed = code_str.trim_end_matches('\0');
                let issuer = Self::account_id_to_string(xr)?;
                Ok(ParsedChangeTrustAsset::CreditAlphanum4 {
                    code: String::from(trimmed),
                    issuer,
                })
            }
            ASSET_TYPE_CREDIT_ALPHANUM12 => {
                let code_bytes = xr.read_fixed_opaque(12)?;
                let code_str = core::str::from_utf8(code_bytes).unwrap_or("invalid");
                let trimmed = code_str.trim_end_matches('\0');
                let issuer = Self::account_id_to_string(xr)?;
                Ok(ParsedChangeTrustAsset::CreditAlphanum12 {
                    code: String::from(trimmed),
                    issuer,
                })
            }
            ASSET_TYPE_POOL_SHARE => {
                // LiquidityPoolParameters (only constant product now)
                // enum LiquidityPoolType { LIQUIDITY_POOL_CONSTANT_PRODUCT = 0; }
                let lp_type = xr.read_i32()?; // expect 0
                if lp_type != 0 {
                    return Err(TransactionParseError::UnsupportedOperation);
                }
                // LiquidityPoolConstantProductParameters
                let asset_a = Self::parse_asset(xr)?;
                let asset_b = Self::parse_asset(xr)?;
                let fee = xr.read_i32()?;
                Ok(ParsedChangeTrustAsset::LiquidityPool { asset_a, asset_b, fee })
            }
            _ => Err(TransactionParseError::UnsupportedOperation),
        }
    }

    fn parse_signatures(xr: &mut Xdr) -> Result<Vec<ParsedSignature>, TransactionParseError> {
        let count = xr.read_u32()? as usize;
        if count > MAX_SIGNATURES {
            return Err(TransactionParseError::TooManySignatures);
        }

        let mut sigs = Vec::new();
        for _ in 0..count {
            let hint_bytes = xr.read_fixed_opaque(4)?;
            let mut hint = [0u8; 4];
            hint.copy_from_slice(hint_bytes);

            let sig_bytes = xr.read_opaque(64)?;
            let mut signature = [0u8; 64];
            for (i, b) in sig_bytes.iter().enumerate() {
                if i < 64 {
                    signature[i] = *b;
                }
            }

            let hint = bytes_to_hex(&hint);
            let signature = bytes_to_hex(&signature);

            sigs.push(ParsedSignature { hint, signature });
        }

        Ok(sigs)
    }

    fn account_id_to_string(
        xr: &mut Xdr,
    ) -> Result<String, TransactionParseError> {
        let t = xr.read_i32()?;
        if t != KEY_TYPE_ED25519 {
            return Err(TransactionParseError::AccountIdError);
        }
        let key = xr.read_fixed_opaque(32)?;
        Self::u8_array_to_stellar_address(key)
    }

    fn muxed_account_to_string(muxed_account: &ParsedMuxedAccount) -> Result<String, TransactionParseError> {
        match muxed_account {
            ParsedMuxedAccount::Ed25519 { account_id } => {
                String::try_from(account_id.as_str()).map_err(|_| TransactionParseError::StringTooLong)
            }
            ParsedMuxedAccount::MuxedEd25519 { account_id, .. } => {
                String::try_from(account_id.as_str()).map_err(|_| TransactionParseError::StringTooLong)
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
        let fractional = (stroops % 10_000_000).abs();

        log_info!("Converting stroops: {}, whole: {}, fractional: {}", stroops, whole_xlm, fractional);
        
        if fractional == 0 {
            format!("{}", whole_xlm)
        } else {
            // Pad to 7 digits, then trim trailing zeros
            let frac_str = format!("{:07}", fractional);
            let trimmed = frac_str.trim_end_matches('0');
            format!("{}.{}", whole_xlm, trimmed)
        }
    }
}

impl Default for StellarTransactionParser {
    fn default() -> Self {
        Self::new()
    }
}