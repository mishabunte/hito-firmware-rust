#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::{vec::Vec, string::String, string::ToString, format};
use core::convert::TryInto;
use ed25519_dalek::{Signer, SigningKey};
use sha2::{Digest, Sha256};
use base32::{Alphabet, decode};

use crate::crypto::libcrypt0pro::stellar::{StellarWallet, StellarKeypair};

// Stellar network constants
pub const STELLAR_MAINNET_NETWORK_PASSPHRASE: &[u8] = b"Public Global Stellar Network ; September 2015";
pub const STELLAR_TESTNET_NETWORK_PASSPHRASE: &[u8] = b"Test SDF Network ; September 2015";

#[derive(Debug, Clone)]
pub enum StellarNetwork {
    Mainnet,
    Testnet,
    Custom(String),
}

impl StellarNetwork {
    pub fn passphrase(&self) -> &[u8] {
        match self {
            StellarNetwork::Mainnet => STELLAR_MAINNET_NETWORK_PASSPHRASE,
            StellarNetwork::Testnet => STELLAR_TESTNET_NETWORK_PASSPHRASE,
            StellarNetwork::Custom(passphrase) => passphrase.as_bytes(),
        }
    }

    pub fn id(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(self.passphrase());
        hasher.finalize().into()
    }
}

#[derive(Debug)]
pub enum TransactionError {
    InvalidXdr,
    InvalidSignature,
    NetworkMismatch,
    SerializationError,
    DecodingError,
    InvalidTransaction,
}

impl core::fmt::Display for TransactionError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            TransactionError::InvalidXdr => write!(f, "Invalid XDR data"),
            TransactionError::InvalidSignature => write!(f, "Invalid signature"),
            TransactionError::NetworkMismatch => write!(f, "Network mismatch"),
            TransactionError::SerializationError => write!(f, "Serialization error"),
            TransactionError::DecodingError => write!(f, "Decoding error"),
            TransactionError::InvalidTransaction => write!(f, "Invalid transaction"),
        }
    }
}

// Simplified XDR structures for essential Stellar transaction components
#[derive(Debug, Clone)]
pub struct AccountId {
    pub key: [u8; 32], // Ed25519 public key
}

impl AccountId {
    pub fn from_address(address: &str) -> Result<Self, TransactionError> {
        // Remove 'G' prefix and decode base32
        if !address.starts_with('G') || address.len() != 56 {
            return Err(TransactionError::DecodingError);
        }

        let decoded = decode(Alphabet::Rfc4648 { padding: false }, address)
            .ok_or(TransactionError::DecodingError)?;
        
        if decoded.len() != 35 {
            return Err(TransactionError::DecodingError);
        }
        
        // Skip version byte (first byte) and checksum (last 2 bytes)
        let key: [u8; 32] = decoded[1..33]
            .try_into()
            .map_err(|_| TransactionError::DecodingError)?;
        
        Ok(AccountId { key })
    }

    pub fn to_address(&self) -> Result<String, TransactionError> {
        use base32::encode;
        
        let version_byte = 6u8 << 3; // 48 in decimal
        let mut payload = Vec::new();
        payload.push(version_byte);
        payload.extend_from_slice(&self.key);
        
        // Calculate CRC16 checksum
        use crc::{Crc, CRC_16_XMODEM};
        let crc = Crc::<u16>::new(&CRC_16_XMODEM);
        let checksum = crc.checksum(&payload);
        
        payload.extend_from_slice(&checksum.to_le_bytes());
        
        let encoded = encode(Alphabet::Rfc4648 { padding: false}, &payload);
        Ok(encoded)
    }
}

#[derive(Debug, Clone)]
pub enum OperationType {
    CreateAccount,
    Payment,
    PathPaymentStrictReceive,
    PathPaymentStrictSend,
    ManageSellOffer,
    CreatePassiveSellOffer,
    SetOptions,
    ChangeTrust,
    AllowTrust,
    AccountMerge,
    Inflation,
    ManageData,
    BumpSequence,
    ManageBuyOffer,
    // Add more as needed
}

#[derive(Debug, Clone)]
pub struct Asset {
    pub asset_type: AssetType,
}

#[derive(Debug, Clone)]
pub enum AssetType {
    Native, // XLM
    CreditAlphanum4 {
        asset_code: [u8; 4],
        issuer: AccountId,
    },
    CreditAlphanum12 {
        asset_code: [u8; 12],
        issuer: AccountId,
    },
}

#[derive(Debug, Clone)]
pub struct Operation {
    pub source_account: Option<AccountId>,
    pub operation_type: OperationType,
    pub operation_data: OperationData,
}

#[derive(Debug, Clone)]
pub enum OperationData {
    CreateAccount {
        destination: AccountId,
        starting_balance: i64,
    },
    Payment {
        destination: AccountId,
        asset: Asset,
        amount: i64, // In stroops (1 XLM = 10^7 stroops)
    },
    ChangeTrust {
        line: Asset,
        limit: i64,
    },
    SetOptions {
        inflation_dest: Option<AccountId>,
        clear_flags: Option<u32>,
        set_flags: Option<u32>,
        master_weight: Option<u32>,
        low_threshold: Option<u32>,
        med_threshold: Option<u32>,
        high_threshold: Option<u32>,
        home_domain: Option<String>,
        signer: Option<(AccountId, u32)>, // (key, weight)
    },
    // Add more operation types as needed
    Raw(Vec<u8>), // For unsupported operations
}

#[derive(Debug, Clone)]
pub struct Memo {
    pub memo_type: MemoType,
}

#[derive(Debug, Clone)]
pub enum MemoType {
    None,
    Text(String),
    Id(u64),
    Hash([u8; 32]),
    Return([u8; 32]),
}

#[derive(Debug, Clone)]
pub struct TimeBounds {
    pub min_time: u64,
    pub max_time: u64,
}

#[derive(Debug, Clone)]
pub struct TransactionV1 {
    pub source_account: AccountId,
    pub fee: u32,
    pub seq_num: u64,
    pub time_bounds: Option<TimeBounds>,
    pub memo: Memo,
    pub operations: Vec<Operation>,
}

#[derive(Debug, Clone)]
pub struct Transaction {
    pub network: StellarNetwork,
    pub transaction_v1: TransactionV1,
}

#[derive(Debug, Clone)]
pub struct TransactionEnvelope {
    pub transaction: Transaction,
    pub signatures: Vec<DecoratedSignature>,
}

#[derive(Debug, Clone)]
pub struct DecoratedSignature {
    pub hint: [u8; 4], // Last 4 bytes of public key
    pub signature: [u8; 64], // Ed25519 signature
}

impl Transaction {
    /// Parse transaction from XDR base64 string
    pub fn from_xdr(xdr_base64: &str, network: StellarNetwork) -> Result<Self, TransactionError> {
        // Decode base64
        let xdr_bytes = base64_decode(xdr_base64)
            .map_err(|_| TransactionError::InvalidXdr)?;
        
        Self::from_xdr_bytes(&xdr_bytes, network)
    }

    /// Parse transaction from XDR bytes (simplified parser)
    pub fn from_xdr_bytes(xdr_bytes: &[u8], network: StellarNetwork) -> Result<Self, TransactionError> {
        if xdr_bytes.len() < 100 {
            return Err(TransactionError::InvalidXdr);
        }

        let mut cursor = XdrCursor::new(xdr_bytes);
        
        // Parse transaction envelope discriminant
        let envelope_type = cursor.read_u32()?;
        
        // For TransactionV1Envelope (ENVELOPE_TYPE_TX = 2)
        if envelope_type != 2 {
            return Err(TransactionError::InvalidXdr);
        }

        // Parse source account (32 bytes public key + discriminant)
        let _account_type = cursor.read_u32()?; // Should be 0 for PUBLIC_KEY_TYPE_ED25519
        let source_account_key = cursor.read_bytes(32)?;
        let source_account = AccountId { 
            key: source_account_key.try_into()
                .map_err(|_| TransactionError::InvalidXdr)? 
        };

        // Parse fee
        let fee = cursor.read_u32()?;

        // Parse sequence number
        let seq_num = cursor.read_u64()?;

        // Parse time bounds (optional)
        let has_time_bounds = cursor.read_u32()? != 0;
        let time_bounds = if has_time_bounds {
            let min_time = cursor.read_u64()?;
            let max_time = cursor.read_u64()?;
            Some(TimeBounds { min_time, max_time })
        } else {
            None
        };

        // Parse memo
        let memo_type = cursor.read_u32()?;
        let memo = match memo_type {
            0 => Memo { memo_type: MemoType::None },
            1 => {
                let len = cursor.read_u32()? as usize;
                let text_bytes = cursor.read_bytes(len)?;
                let text = String::from_utf8_lossy(&text_bytes).to_string();
                Memo { memo_type: MemoType::Text(text) }
            },
            2 => {
                let id = cursor.read_u64()?;
                Memo { memo_type: MemoType::Id(id) }
            },
            3 => {
                let hash = cursor.read_bytes(32)?.try_into()
                    .map_err(|_| TransactionError::InvalidXdr)?;
                Memo { memo_type: MemoType::Hash(hash) }
            },
            4 => {
                let return_hash = cursor.read_bytes(32)?.try_into()
                    .map_err(|_| TransactionError::InvalidXdr)?;
                Memo { memo_type: MemoType::Return(return_hash) }
            },
            _ => return Err(TransactionError::InvalidXdr),
        };

        // Parse operations
        let num_operations = cursor.read_u32()? as usize;
        let mut operations = Vec::with_capacity(num_operations);
        
        for _ in 0..num_operations {
            let operation = Self::parse_operation(&mut cursor)?;
            operations.push(operation);
        }

        let transaction_v1 = TransactionV1 {
            source_account,
            fee,
            seq_num,
            time_bounds,
            memo,
            operations,
        };

        Ok(Transaction {
            network,
            transaction_v1,
        })
    }

    fn parse_operation(cursor: &mut XdrCursor) -> Result<Operation, TransactionError> {
        // Parse source account (optional)
        let has_source_account = cursor.read_u32()? != 0;
        let source_account = if has_source_account {
            let _account_type = cursor.read_u32()?;
            let key = cursor.read_bytes(32)?;
            Some(AccountId { 
                key: key.try_into().map_err(|_| TransactionError::InvalidXdr)? 
            })
        } else {
            None
        };

        // Parse operation type
        let op_type_num = cursor.read_u32()?;
        let operation_type = match op_type_num {
            0 => OperationType::CreateAccount,
            1 => OperationType::Payment,
            2 => OperationType::PathPaymentStrictReceive,
            6 => OperationType::SetOptions,
            8 => OperationType::ChangeTrust,
            // Add more mappings as needed
            _ => return Ok(Operation {
                source_account,
                operation_type: OperationType::Payment, // Default fallback
                operation_data: OperationData::Raw(Vec::new()),
            }),
        };

        // Parse operation data based on type
        let operation_data = match operation_type {
            OperationType::Payment => {
                let _dest_account_type = cursor.read_u32()?;
                let dest_key = cursor.read_bytes(32)?;
                let destination = AccountId { 
                    key: dest_key.try_into().map_err(|_| TransactionError::InvalidXdr)? 
                };
                
                let asset = Self::parse_asset(cursor)?;
                let amount = cursor.read_i64()?;
                
                OperationData::Payment {
                    destination,
                    asset,
                    amount,
                }
            },
            OperationType::CreateAccount => {
                let _dest_account_type = cursor.read_u32()?;
                let dest_key = cursor.read_bytes(32)?;
                let destination = AccountId { 
                    key: dest_key.try_into().map_err(|_| TransactionError::InvalidXdr)? 
                };
                let starting_balance = cursor.read_i64()?;
                
                OperationData::CreateAccount {
                    destination,
                    starting_balance,
                }
            },
            _ => {
                // For unsupported operations, store raw bytes
                OperationData::Raw(Vec::new())
            },
        };

        Ok(Operation {
            source_account,
            operation_type,
            operation_data,
        })
    }

    fn parse_asset(cursor: &mut XdrCursor) -> Result<Asset, TransactionError> {
        let asset_type = cursor.read_u32()?;
        
        let asset_type = match asset_type {
            0 => AssetType::Native,
            1 => {
                let asset_code = cursor.read_bytes(4)?;
                let _issuer_type = cursor.read_u32()?;
                let issuer_key = cursor.read_bytes(32)?;
                
                AssetType::CreditAlphanum4 {
                    asset_code: asset_code.try_into().map_err(|_| TransactionError::InvalidXdr)?,
                    issuer: AccountId { 
                        key: issuer_key.try_into().map_err(|_| TransactionError::InvalidXdr)? 
                    },
                }
            },
            2 => {
                let asset_code = cursor.read_bytes(12)?;
                let _issuer_type = cursor.read_u32()?;
                let issuer_key = cursor.read_bytes(32)?;
                
                AssetType::CreditAlphanum12 {
                    asset_code: asset_code.try_into().map_err(|_| TransactionError::InvalidXdr)?,
                    issuer: AccountId { 
                        key: issuer_key.try_into().map_err(|_| TransactionError::InvalidXdr)? 
                    },
                }
            },
            _ => return Err(TransactionError::InvalidXdr),
        };

        Ok(Asset { asset_type })
    }

    /// Get transaction hash for signing
    pub fn hash(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        
        // Add network ID
        hasher.update(&self.network.id());
        
        // Add transaction envelope type (ENVELOPE_TYPE_TX = 2)
        hasher.update(&[0, 0, 0, 2]);
        
        // Add transaction XDR (simplified)
        let tx_xdr = self.to_xdr_bytes();
        hasher.update(&tx_xdr);
        
        hasher.finalize().into()
    }

    /// Convert transaction to XDR bytes (simplified)
    fn to_xdr_bytes(&self) -> Vec<u8> {
        let mut xdr = Vec::new();
        
        // This is a simplified implementation
        // In a real implementation, you'd need complete XDR serialization
        
        // Source account
        xdr.extend_from_slice(&[0, 0, 0, 0]); // PUBLIC_KEY_TYPE_ED25519
        xdr.extend_from_slice(&self.transaction_v1.source_account.key);
        
        // Fee
        xdr.extend_from_slice(&self.transaction_v1.fee.to_be_bytes());
        
        // Sequence number
        xdr.extend_from_slice(&self.transaction_v1.seq_num.to_be_bytes());
        
        // Add more fields as needed...
        
        xdr
    }

    /// Sign the transaction with a keypair
    pub fn sign(&self, keypair: &StellarKeypair) -> Result<DecoratedSignature, TransactionError> {
        let tx_hash = self.hash();
        
        let signing_key = SigningKey::from_bytes(&keypair.secret_key);
        let signature = signing_key.try_sign(&tx_hash)
            .map_err(|_| TransactionError::InvalidSignature)?;
        // Create signature hint (last 4 bytes of public key)
        let hint: [u8; 4] = keypair.public_key[28..32]
            .try_into()
            .map_err(|_| TransactionError::InvalidSignature)?;
        
        Ok(DecoratedSignature {
            hint,
            signature: signature.to_bytes(),
        })
    }

    /// Create a signed transaction envelope
    pub fn create_envelope(&self, signatures: Vec<DecoratedSignature>) -> TransactionEnvelope {
        TransactionEnvelope {
            transaction: self.clone(),
            signatures,
        }
    }
}

// Helper struct for parsing XDR
pub(crate) struct XdrCursor<'a> {
    data: &'a [u8],
    position: usize,
}

impl<'a> XdrCursor<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, position: 0 }
    }

    fn read_u32(&mut self) -> Result<u32, TransactionError> {
        if self.position + 4 > self.data.len() {
            return Err(TransactionError::InvalidXdr);
        }
        
        let bytes: [u8; 4] = self.data[self.position..self.position + 4]
            .try_into()
            .map_err(|_| TransactionError::InvalidXdr)?;
        
        self.position += 4;
        Ok(u32::from_be_bytes(bytes))
    }

    fn read_u64(&mut self) -> Result<u64, TransactionError> {
        if self.position + 8 > self.data.len() {
            return Err(TransactionError::InvalidXdr);
        }
        
        let bytes: [u8; 8] = self.data[self.position..self.position + 8]
            .try_into()
            .map_err(|_| TransactionError::InvalidXdr)?;
        
        self.position += 8;
        Ok(u64::from_be_bytes(bytes))
    }

    fn read_i64(&mut self) -> Result<i64, TransactionError> {
        if self.position + 8 > self.data.len() {
            return Err(TransactionError::InvalidXdr);
        }
        
        let bytes: [u8; 8] = self.data[self.position..self.position + 8]
            .try_into()
            .map_err(|_| TransactionError::InvalidXdr)?;
        
        self.position += 8;
        Ok(i64::from_be_bytes(bytes))
    }

    fn read_bytes(&mut self, len: usize) -> Result<Vec<u8>, TransactionError> {
        if self.position + len > self.data.len() {
            return Err(TransactionError::InvalidXdr);
        }
        
        let bytes = self.data[self.position..self.position + len].to_vec();
        self.position += len;
        
        // XDR padding: round up to multiple of 4
        let padding = (4 - (len % 4)) % 4;
        self.position += padding;
        
        Ok(bytes)
    }
}

// Simple base64 decoder for no_std
fn base64_decode(input: &str) -> Result<Vec<u8>, TransactionError> {
    // This is a simplified base64 decoder
    // In production, use a proper base64 crate
    
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    
    let mut result = Vec::new();
    let input = input.trim_end_matches('=');
    
    for chunk in input.as_bytes().chunks(4) {
        let mut buf = [0u8; 4];
        
        for (i, &byte) in chunk.iter().enumerate() {
            let pos = CHARS.iter().position(|&x| x == byte)
                .ok_or(TransactionError::DecodingError)? as u8;
            buf[i] = pos;
        }
        
        result.push((buf[0] << 2) | (buf[1] >> 4));
        if chunk.len() > 2 {
            result.push((buf[1] << 4) | (buf[2] >> 2));
        }
        if chunk.len() > 3 {
            result.push((buf[2] << 6) | buf[3]);
        }
    }
    
    Ok(result)
}