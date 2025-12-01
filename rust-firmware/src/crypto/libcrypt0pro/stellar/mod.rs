pub mod address;
pub use address::{StellarKeypair, StellarWallet, StellarError};

pub mod transaction_parser;
pub use transaction_parser::StellarTransactionParser;
//pub use transaction_serializer::StellarTransactionSerializer;

//Re-export commonly used types
pub use transaction_parser::{
    ParsedTransaction, ParsedMemo, ParsedTimeBounds, ParsedLedgerBounds,
    ParsedOperation, ParsedMuxedAccount, OperationDetails, ParsedAsset,
    ParsedChangeTrustAsset, ParsedSignature, TransactionEnvelopeType,
    TransactionParseError,
    MAX_OPERATIONS, MAX_PATH_ASSETS, MAX_SIGNATURES, MAX_CLAIMANTS,
    MAX_STRING_LEN, MAX_DATA_VALUE_LEN, MAX_ASSET_CODE_LEN, MAX_XDR_LEN
};

// // Re-export address functionality  
// pub use address::*;