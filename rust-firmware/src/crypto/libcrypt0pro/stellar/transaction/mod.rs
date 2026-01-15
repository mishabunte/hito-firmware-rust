mod xdr;

pub mod transaction_parser;
pub use transaction_parser::StellarTransactionParser;

pub mod transaction_signer;
pub use transaction_signer::StellarTransactionSigner;

pub mod transaction_serializer;
pub use transaction_serializer::StellarTransactionSerializer;

extern crate alloc;
use alloc::{string::String, vec::Vec};

use crate::crypto::crypt0::hex_to_bytes;

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

// Configuration constants - adjust based on your needs
pub const MAX_OPERATIONS: usize = 100;
pub const MAX_PATH_ASSETS: usize = 5;
pub const MAX_SIGNATURES: usize = 20;
pub const MAX_CLAIMANTS: usize = 10;
pub const MAX_STRING_LEN: usize = 64;
pub const MAX_DATA_VALUE_LEN: usize = 64;
pub const MAX_ASSET_CODE_LEN: usize = 12;

#[derive(Debug, Clone)]
pub struct Transaction {
    pub network_hash: Option<Network>,
    pub source_account: String,
    pub sequence_number: i64,
    pub fee: i64,
    pub memo: Option<Memo>,
    pub time_bounds: Option<TimeBounds>,
    pub ledger_bounds: Option<LedgerBounds>,
    pub operations: Vec<Operation>,
    pub signatures: Vec<Signature>,
    pub envelope_type: TransactionEnvelopeType,
}

#[derive(Debug, Clone)]
pub struct FeeBumpTransaction {
  pub fee_source: MuxedAccount,
  pub fee: i64,
  pub inner_tx: Transaction,
  pub signatures: Vec<Signature>,
}

#[derive(Debug, Clone)]
pub enum TransactionEnvelope {
    Transaction(Transaction),
    FeeBump(FeeBumpTransaction),
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
pub struct Memo {
    pub memo_type: String,
    pub value: Option<String>,
}

#[derive(Debug, Clone)]
pub struct TimeBounds {
    pub min_time: Option<u64>,
    pub max_time: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct LedgerBounds {
    pub min_ledger: u32,
    pub max_ledger: u32,
}

#[derive(Debug, Clone)]
pub struct Operation {
    pub operation_type: String,
    pub source_account: Option<String>,
    pub details: OperationDetails,
}

#[derive(Debug, Clone)]
pub enum MuxedAccount {
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
        destination: MuxedAccount,
        asset: Asset,
        amount: i64,
    },
    Other {
        operation_type: String,
        raw_data_len: usize, // Store only length in no-std
    },
    PathPaymentStrictSend {
        send_asset: Asset,
        send_amount: i64,
        destination: MuxedAccount,
        dest_asset: Asset,
        dest_min: i64,
        path: Vec<Asset>,
    },
    PathPaymentStrictReceive {
        send_asset: Asset,
        send_max: i64,
        destination: MuxedAccount,
        dest_asset: Asset,
        dest_amount: i64,
        path: Vec<Asset>,
    },
    ChangeTrust {
        asset: ChangeTrustAsset,
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
        asset: Asset,
        clear_flags: u32,
        set_flags: u32,
    },
}

#[derive(Debug, Clone)]
pub enum Asset {
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
pub enum ChangeTrustAsset {
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
        asset_a: Asset,
        asset_b: Asset,
        fee: i32,
    },
}

#[derive(Debug, Clone)]
pub struct Signer {
    pub key: String,
    pub weight: u32,
}

#[derive(Debug, Clone)]
pub struct Signature {
    pub hint: String, // Signature hints are always 4 bytes
    pub signature: String, // Ed25519 signatures are 64 bytes
}

#[derive(Debug)]
pub enum StellarTransactionError {
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
    InvalidTransaction,
    SigningFailed,
}

impl core::fmt::Display for StellarTransactionError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            StellarTransactionError::Base64Error => write!(f, "Base64 decode error"),
            StellarTransactionError::XdrError => write!(f, "XDR parsing error"),
            StellarTransactionError::InvalidEnvelopeType => write!(f, "Invalid transaction envelope type"),
            StellarTransactionError::UnsupportedOperation => write!(f, "Unsupported operation type"),
            StellarTransactionError::StringTooLong => write!(f, "String too long"),
            StellarTransactionError::TooManyOperations => write!(f, "Too many operations in transaction"),
            StellarTransactionError::TooManySignatures => write!(f, "Too many signatures in transaction"),
            StellarTransactionError::PayloadTooLarge => write!(f, "Payload too large"),
            StellarTransactionError::AccountIdError => write!(f, "Failed to parse account ID"),
            StellarTransactionError::MuxedAccountError => write!(f, "Failed to parse muxed account"),
            StellarTransactionError::PayloadTooSmall => write!(f, "Payload too small"),
            StellarTransactionError::InvalidPrefix => write!(f, "Invalid transaction format prefix"),
            StellarTransactionError::InvalidNetworkId => write!(f, "Invalid network ID"),
            StellarTransactionError::InvalidTransaction => write!(f, "Invalid transaction"),
            StellarTransactionError::SigningFailed => write!(f, "Signing failed"),
        }
    }
}

// // Re-export address functionality  
// pub use address::*;