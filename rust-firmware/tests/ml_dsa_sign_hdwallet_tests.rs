// tests/ml_dsa_sign_hdwallet_tests.rs
// Integration tests for HDLattice functionality

use std::fmt::Debug;
use base32::*;
use hito_firmware_rust::crypto::libcrypt0pro::stellar::*;
use hito_firmware_rust::crypto::crypt0::hex_to_bytes;
use qp_rusty_crystals_hdwallet::{HDLattice};
use qp_rusty_crystals_dilithium::{ml_dsa_87, PH};

fn get_test_seed() -> [u8; 64] {
    let mut seed = [0u8; 64];
    // This is a test seed derived from the standard test mnemonic:
    // "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"
    let bytes = hex_to_bytes("38b6a363e88b28138cc71f0145ab429c251baa8cd8fa6d80bcfb39c35076f1766e24dfc01ce0e22e8dfec185ad7a67ce748cd6551ad1b738619b8859808bbf88").unwrap();
    seed.copy_from_slice(&bytes);
    seed
}

#[test]
fn test_lattice_wallet_creation() {
    let seed = get_test_seed();
    let hd_wallet = HDLattice::from_seed(seed).expect("Failed to derive wallet");
    let path = "m/44'/148'/0'/0'/0'";
    let keys = hd_wallet.generate_derived_keys(path).expect("failed to derive hdlattice keys");
    let message = b"baby boomer";
    
    let signature = keys.sign(message, None, None);
    println!("{}, {}", base32::encode(Alphabet::Crockford, &signature), signature.len());
    println!("{}", keys.public.bytes.len());
    
    let signature_ph = keys.prehash_sign(message, None, None, PH::SHA256).expect("Couldnt prehash sign");
    println!("{}, {}", base32::encode(Alphabet::Crockford, &signature_ph), signature_ph.len());
    // let lattice_address = StellarWallet::encode_stellar_address(&keys.public.);
}