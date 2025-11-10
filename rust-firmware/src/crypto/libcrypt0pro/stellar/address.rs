#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

// use ed25519_dalek::SigningKey;
// use sha2::{Sha512};
// use hmac::{Hmac, Mac};
use base32::{Alphabet, encode};
use crc::{Crc, CRC_16_XMODEM};
use crate::crypto::ffi;
use crate::crypto::crypt0::crypt0_ed25519_derive_secret_index;

// Import alloc types for no_std compatibility
use alloc::{
    vec::Vec,
    string::String,
    format,
};
use core::{cmp, convert::TryInto, result};

//type HmacSha512 = Hmac<Sha512>;

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
        let secret = self.derive_master_key()?;

        crypt0_ed25519_derive_secret_index(
            &mut secret.as_bytes().try_into().map_err(|_| StellarError::DerivationError)?,
            44 | HARDENED_OFFSET,
        );
        crypt0_ed25519_derive_secret_index(
            &mut secret.as_bytes().try_into().map_err(|_| StellarError::DerivationError)?,
            STELLAR_COIN_TYPE | HARDENED_OFFSET,
        );
        crypt0_ed25519_derive_secret_index(
            &mut secret.as_bytes().try_into().map_err(|_| StellarError::DerivationError)?,
            account_index | HARDENED_OFFSET,
        );
        // Extract the private key (first 32 bytes of the key)
        let secret_key: [u8; 32] = secret.key[..32]
            .try_into()
            .map_err(|_| StellarError::DerivationError)?;
        
        // Generate Ed25519 keypair
        let public_key: [u8; 32] = {
            let mut pubkey = [0u8; 32];
            let res = unsafe {
                ffi::crypt0_ed25519_public_key(
                    secret_key.as_ptr(),
                    secret_key.len(),
                    pubkey.as_mut_ptr(),
                    pubkey.len(),
                )
            };
            if res != ffi::CRYPT0_OK {
                return Err(StellarError::CryptoError);
            }
            pubkey
        };
        
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
        let mut result = {
            let mut buf = [0u8; 64];
            let res = unsafe {
                ffi::crypt0_hmac_sha512(
                    b"ed25519 seed".as_ptr(),
                    b"ed25519 seed".len() as u16,
                    self.seed.as_ptr(),
                    self.seed.len() as u16,
                    buf.as_mut_ptr(),
                )
            };
            if res != ffi::CRYPT0_OK {
                return Err(StellarError::CryptoError);
            }
            buf
        };

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

impl ExtendedKey {
    pub fn as_bytes(&self) -> [u8; 64] {
        let mut bytes = [0u8; 64];
        bytes[..32].copy_from_slice(&self.key);
        bytes[32..].copy_from_slice(&self.chain_code);
        bytes
    }
}