#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use ed25519_dalek::SigningKey;
use sha2::{Sha512};
use hmac::{Hmac, Mac};
use base32::{Alphabet, encode};
use crc::{Crc, CRC_16_XMODEM};

// Import alloc types for no_std compatibility
use alloc::{
    vec::Vec,
    string::String,
    boxed::Box,
    format,
};
use core::{convert::TryInto, cmp};

type HmacSha512 = Hmac<Sha512>;

const STELLAR_COIN_TYPE: u32 = 148;
const HARDENED_OFFSET: u32 = 0x80000000;

#[derive(Debug)]
pub struct StellarWallet {
    pub seed: [u8; 64],
}

#[derive(Debug)]
pub struct StellarKeypair {
    pub secret_key: [u8; 32],
    pub public_key: [u8; 32],
    pub address: String,
}

// Custom error type for no_std compatibility
#[derive(Debug)]
pub enum StellarError {
    InvalidSeed,
    DerivationError,
    AddressEncodeError,
    CryptoError,
}

impl core::fmt::Display for StellarError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            StellarError::InvalidSeed => write!(f, "Invalid seed"),
            StellarError::DerivationError => write!(f, "Key derivation error"),
            StellarError::AddressEncodeError => write!(f, "Address encoding error"),
            StellarError::CryptoError => write!(f, "Cryptographic error"),
        }
    }
}

impl StellarWallet {
    /// Create a new wallet from a BIP39 seed
    pub fn from_seed(seed: [u8; 64]) -> Self {
        Self { seed }
    }

    /// Derive a Stellar keypair at the given account index
    /// Uses derivation path: m/44'/148'/account'
    pub fn derive_keypair(&self, account_index: u32) -> Result<StellarKeypair, StellarError> {
        // BIP32 master key derivation
        let master_key = self.derive_master_key()?;
        
        // Derive m/44'
        let purpose_key = self.derive_child_key(&master_key, 44 | HARDENED_OFFSET)?;
        
        // Derive m/44'/148' (Stellar coin type)
        let coin_key = self.derive_child_key(&purpose_key, STELLAR_COIN_TYPE | HARDENED_OFFSET)?;
        
        // Derive m/44'/148'/account'
        let account_key = self.derive_child_key(&coin_key, account_index | HARDENED_OFFSET)?;
        
        // Extract the private key (first 32 bytes of the key)
        let secret_key: [u8; 32] = account_key.key[..32]
            .try_into()
            .map_err(|_| StellarError::DerivationError)?;
        
        // Generate Ed25519 keypair
        let signing_key = SigningKey::from_bytes(&secret_key);
        let verifying_key = signing_key.verifying_key();
        let public_key = verifying_key.to_bytes();
        
        // Generate Stellar address
        let address = self.encode_stellar_address(&public_key)?;
        
        Ok(StellarKeypair {
            secret_key,
            public_key,
            address,
        })
    }

    /// Derive master key from seed using HMAC-SHA512
    fn derive_master_key(&self) -> Result<ExtendedKey, StellarError> {
        let mut mac = HmacSha512::new_from_slice(b"ed25519 seed")
            .map_err(|_| StellarError::CryptoError)?;
        mac.update(&self.seed);
        let result = mac.finalize().into_bytes();
        
        let key = result[..32]
            .try_into()
            .map_err(|_| StellarError::DerivationError)?;
        let chain_code = result[32..]
            .try_into()
            .map_err(|_| StellarError::DerivationError)?;
        
        Ok(ExtendedKey { key, chain_code })
    }

    /// Derive child key using BIP32 derivation
    fn derive_child_key(&self, parent: &ExtendedKey, index: u32) -> Result<ExtendedKey, StellarError> {
        let mut data = Vec::new();
        
        if index >= HARDENED_OFFSET {
            // Hardened derivation: use 0x00 + parent_key + index
            data.push(0x00);
            data.extend_from_slice(&parent.key);
        } else {
            // Non-hardened derivation: use parent_public_key + index
            let signing_key = SigningKey::from_bytes(&parent.key);
            let public_key = signing_key.verifying_key().to_bytes();
            data.extend_from_slice(&public_key);
        }
        
        data.extend_from_slice(&index.to_be_bytes());
        
        let mut mac = HmacSha512::new_from_slice(&parent.chain_code)
            .map_err(|_| StellarError::CryptoError)?;
        mac.update(&data);
        let result = mac.finalize().into_bytes();
        
        let key = result[..32]
            .try_into()
            .map_err(|_| StellarError::DerivationError)?;
        let chain_code = result[32..]
            .try_into()
            .map_err(|_| StellarError::DerivationError)?;
        
        Ok(ExtendedKey { key, chain_code })
    }

    /// Encode public key as Stellar address (starting with 'G')
    fn encode_stellar_address(&self, public_key: &[u8; 32]) -> Result<String, StellarError> {
        // Stellar uses account ID version byte (6 << 3 = 48)
        let version_byte = 6u8 << 3; // 48 in decimal
        
        // Create payload: version_byte + public_key
        let mut payload = Vec::new();
        payload.push(version_byte);
        payload.extend_from_slice(public_key);
        
        // Calculate CRC16 checksum
        let crc = Crc::<u16>::new(&CRC_16_XMODEM);
        let checksum = crc.checksum(&payload);
        
        // Append checksum (little-endian)
        payload.extend_from_slice(&checksum.to_le_bytes());
        
        // Encode with base32 (RFC 4648 without padding)
        let encoded = encode(Alphabet::Rfc4648 { padding: false }, &payload);

        Ok(encoded)
    }

    /// Generate a seed phrase address (for account 0)
    pub fn get_default_address(&self) -> Result<String, StellarError> {
        let keypair = self.derive_keypair(0)?;
        Ok(keypair.address)
    }
}

#[derive(Debug)]
struct ExtendedKey {
    key: [u8; 32],
    chain_code: [u8; 32],
}

// Helper function to convert BIP39 mnemonic to seed (proper PBKDF2 for no_std)
pub fn mnemonic_to_seed(mnemonic: &str, passphrase: &str) -> [u8; 64] {
    use sha2::Sha512;
    use hmac::{Hmac, Mac};
    
    type HmacSha512 = Hmac<Sha512>;
    
    let salt = if passphrase.is_empty() {
        "mnemonic".as_bytes()
    } else {
        return pbkdf2_hmac_sha512(mnemonic.as_bytes(), format!("mnemonic{}", passphrase).as_bytes(), 2048);
    };
    
    pbkdf2_hmac_sha512(mnemonic.as_bytes(), salt, 2048)
}

// Proper PBKDF2-HMAC-SHA512 implementation for no_std
fn pbkdf2_hmac_sha512(password: &[u8], salt: &[u8], iterations: u32) -> [u8; 64] {
    use sha2::Sha512;
    use hmac::{Hmac, Mac};
    
    type HmacSha512 = Hmac<Sha512>;
    
    let mut result = [0u8; 64];
    let hlen = 64; // SHA512 output length
    let dklen = 64; // Desired key length
    
    // Calculate number of blocks needed
    let l = (dklen + hlen - 1) / hlen; // Ceiling division
    
    for i in 1..=l {
        let mut u = Vec::new();
        let mut t = [0u8; 64];
        
        // First iteration: U1 = PRF(P, S || INT_32_BE(i))
        let mut mac = HmacSha512::new_from_slice(password).unwrap();
        mac.update(salt);
        mac.update(&(i as u32).to_be_bytes());
        u = mac.finalize().into_bytes().to_vec();
        t.copy_from_slice(&u[..64]);
        
        // Remaining iterations: Uj = PRF(P, Uj-1)
        for _ in 2..=iterations {
            mac = HmacSha512::new_from_slice(password).unwrap();
            mac.update(&u);
            u = mac.finalize().into_bytes().to_vec();
            
            // XOR with previous result
            for j in 0..64 {
                t[j] ^= u[j];
            }
        }
        
        // Copy to result (for single block, this is the full result)
        let start_idx = (i - 1) * hlen;
        let end_idx = core::cmp::min(start_idx + hlen, dklen);
        let copy_len = end_idx - start_idx;
        
        if start_idx < dklen {
            result[start_idx..start_idx + copy_len].copy_from_slice(&t[..copy_len]);
        }
    }
    
    result
}