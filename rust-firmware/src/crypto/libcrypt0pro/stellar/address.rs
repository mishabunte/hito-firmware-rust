#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use base32::{Alphabet, encode};
use crc::{Crc, CRC_16_XMODEM};
use crate::{crypto::ffi, log_info};
use crate::crypto::crypt0::crypt0_ed25519_derive_secret_index;
use crate::crypto::crypt0::bytes_to_hex;

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

        let purpose_key = crypt0_ed25519_derive_secret_index(
            &secret,
            44 | HARDENED_OFFSET,
        ).unwrap();
        let coin_type_key = crypt0_ed25519_derive_secret_index(
            &purpose_key,
            STELLAR_COIN_TYPE | HARDENED_OFFSET,
        ).unwrap();
        let account_key = crypt0_ed25519_derive_secret_index(
            &coin_type_key,
            account_index | HARDENED_OFFSET,
        ).unwrap();
        // Extract the private key (first 32 bytes of the key)
        let secret_key: [u8; 32] = account_key[..32]
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
        
        // // Generate Stellar address
        // let address = StellarWallet::encode_stellar_address(&public_key)?;
        
        Ok(StellarKeypair {
            secret_key,
            public_key,
        })
    }

    /// Derive master key from seed using HMAC-SHA512
    fn derive_master_key(&self) -> Result<[u8; 64], StellarError> {
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
        Ok(result)
    }

    /// Encode public key as Stellar address (starting with 'G')
    pub fn encode_stellar_address(public_key: &[u8; 32]) -> Result<String, StellarError> {
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

    /// Encode public key as Stellar address (starting with 'G')
    pub fn encode_stellar_secret(secret_key: &[u8; 32]) -> Result<String, StellarError> {
        // Stellar uses account ID version byte (6 << 3 = 48)
        let version_byte = 18u8 << 3; // 144 in decimal for secret key
        
        // Create payload: version_byte + secret_key
        let mut payload = Vec::new();
        payload.push(version_byte);
        payload.extend_from_slice(secret_key);
        
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
        Ok(StellarWallet::encode_stellar_address(&keypair.public_key)?)
    }
}
