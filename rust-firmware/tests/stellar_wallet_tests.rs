// tests/stellar_wallet_tests.rs
// Integration tests for Stellar wallet functionality

use hito_firmware_rust::crypto::libcrypt0pro::stellar::*;
use hex;

// Test with a known seed for reproducible results
fn get_test_seed() -> [u8; 64] {
    // This is a test seed derived from the standard test mnemonic:
    // "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"
    let test_seed_hex = "38b6a363e88b28138cc71f0145ab429c251baa8cd8fa6d80bcfb39c35076f1766e24dfc01ce0e22e8dfec185ad7a67ce748cd6551ad1b738619b8859808bbf88";
    hex::decode(test_seed_hex).unwrap().try_into().unwrap()
}

#[test]
fn test_wallet_creation() {
    let seed = get_test_seed();
    let wallet = StellarWallet::from_seed(seed);
    
    // Wallet should be created successfully
    assert_eq!(wallet.seed.len(), 64);
    println!("✓ Wallet creation test passed");
}

#[test]
fn test_keypair_derivation_account_0() {
    let seed = get_test_seed();
    let wallet = StellarWallet::from_seed(seed);
    
    let keypair = wallet.derive_keypair(0).expect("Failed to derive keypair");
    
    // Verify key lengths
    assert_eq!(keypair.secret_key.len(), 32);
    assert_eq!(keypair.public_key.len(), 32);
    
    // Stellar address should start with 'G'
    assert!(keypair.address.starts_with('G'));
    
    // Address should be the expected length (56 characters)
    assert_eq!(keypair.address.len(), 56);
    
    // Print for manual verification
    println!("✓ Account 0 derivation test passed");
    println!("  Secret Key: {}", hex::encode(keypair.secret_key));
    println!("  Public Key: {}", hex::encode(keypair.public_key));
    println!("  Address: {}", keypair.address);
}

#[test]
fn test_multiple_accounts_derivation() {
    let seed = get_test_seed();
    let wallet = StellarWallet::from_seed(seed);
    
    let account0 = wallet.derive_keypair(0).expect("Failed to derive account 0");
    let account1 = wallet.derive_keypair(1).expect("Failed to derive account 1");
    let account2 = wallet.derive_keypair(2).expect("Failed to derive account 2");
    
    // All addresses should be different
    assert_ne!(account0.address, account1.address);
    assert_ne!(account1.address, account2.address);
    assert_ne!(account0.address, account2.address);
    
    // All should start with 'G'
    assert!(account0.address.starts_with('G'));
    assert!(account1.address.starts_with('G'));
    assert!(account2.address.starts_with('G'));
    
    println!("✓ Multiple accounts derivation test passed");
    println!("  Account 0: {}", account0.address);
    println!("  Account 1: {}", account1.address);
    println!("  Account 2: {}", account2.address);
}

#[test]
fn test_deterministic_derivation() {
    let seed = get_test_seed();
    
    // Create two wallet instances with the same seed
    let wallet1 = StellarWallet::from_seed(seed);
    let wallet2 = StellarWallet::from_seed(seed);
    
    // Derive the same account from both wallets
    let keypair1 = wallet1.derive_keypair(0).expect("Failed to derive from wallet1");
    let keypair2 = wallet2.derive_keypair(0).expect("Failed to derive from wallet2");
    
    // Results should be identical
    assert_eq!(keypair1.secret_key, keypair2.secret_key);
    assert_eq!(keypair1.public_key, keypair2.public_key);
    assert_eq!(keypair1.address, keypair2.address);
    
    println!("✓ Deterministic derivation test passed");
}

#[test]
fn test_address_format_validation() {
    let seed = get_test_seed();
    let wallet = StellarWallet::from_seed(seed);
    
    for account in 0..5 {
        let keypair = wallet.derive_keypair(account).expect("Failed to derive keypair");
        
        // Check address format
        assert!(keypair.address.starts_with('G'), "Address should start with G");
        assert_eq!(keypair.address.len(), 56, "Address should be 56 characters long");
        
        // Check that address contains only valid base32 characters
        let valid_chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
        assert!(keypair.address.chars().all(|c| valid_chars.contains(c)), 
               "Address contains invalid base32 characters");
    }
    
    println!("✓ Address format validation test passed");
}

#[test]
fn test_different_seeds_different_addresses() {
    let seed1 = [1u8; 64];
    let seed2 = [2u8; 64];
    
    let wallet1 = StellarWallet::from_seed(seed1);
    let wallet2 = StellarWallet::from_seed(seed2);
    
    let keypair1 = wallet1.derive_keypair(0).expect("Failed to derive from seed1");
    let keypair2 = wallet2.derive_keypair(0).expect("Failed to derive from seed2");
    
    // Different seeds should produce different keys and addresses
    assert_ne!(keypair1.secret_key, keypair2.secret_key);
    assert_ne!(keypair1.public_key, keypair2.public_key);
    assert_ne!(keypair1.address, keypair2.address);
    
    println!("✓ Different seeds produce different addresses test passed");
}

#[test]
fn test_firmware_performance() {
    let seed = get_test_seed();
    let wallet = StellarWallet::from_seed(seed);
    
    let start = std::time::Instant::now();
    
    // Derive 10 accounts (reasonable for hardware wallet testing)
    for i in 0..10 {
        let _keypair = wallet.derive_keypair(i).expect("Failed to derive keypair");
    }
    
    let duration = start.elapsed();
    println!("✓ Performance test completed");
    println!("  Time to derive 10 accounts: {:?}", duration);
    
    // Should complete in reasonable time for embedded system
    assert!(duration.as_millis() < 1000, "Derivation took too long: {:?}", duration);
}

#[test]
fn test_ed25519_keypair_consistency() {
    let seed = get_test_seed();
    let wallet = StellarWallet::from_seed(seed);
    
    let keypair = wallet.derive_keypair(0).expect("Failed to derive keypair");
    
    // Verify that the public key can be derived from the private key
    use ed25519_dalek::SigningKey;
    let signing_key = SigningKey::from_bytes(&keypair.secret_key);
    let derived_public_key = signing_key.verifying_key().to_bytes();
    
    assert_eq!(keypair.public_key, derived_public_key);
    
    println!("✓ Ed25519 keypair consistency test passed");
}

#[test]
fn test_mnemonic_to_seed_conversion() {
    let mnemonic = "zero zero zero zero zero zero zero zero zero zero zero zoo";
    let seed = mnemonic_to_seed(mnemonic, "");
    
    // Seed should be 64 bytes
    assert_eq!(seed.len(), 64);
    
    // Should be deterministic
    let seed2 = mnemonic_to_seed(mnemonic, "");
    assert_eq!(seed, seed2);
    
    // Different passphrase should produce different seed
    let seed_with_passphrase = mnemonic_to_seed(mnemonic, "test");
    assert_ne!(seed, seed_with_passphrase);
    
    println!("✓ Mnemonic to seed conversion test passed");
    println!("  Seed (hex): {}", hex::encode(&seed[..32])); // Print first 32 bytes
}

// Integration test simulating full hardware wallet workflow
#[test]
fn test_hardware_wallet_workflow() {
    println!("\n=== Hardware Wallet Integration Test ===");
    
    // 1. Simulate user entering mnemonic
    let mnemonic = "zero zero zero zero zero zero zero zero zero zero zero zoo";
    println!("1. Mnemonic entered: {}", mnemonic);
    
    // 2. Convert to seed (this would be done securely in hardware)
    let seed = mnemonic_to_seed(mnemonic, "");
    println!("2. Seed generated (length: {} bytes)", seed.len());
    
    // 3. Create wallet instance
    let wallet = StellarWallet::from_seed(seed);
    println!("3. Wallet instance created");
    
    // 4. Derive first few accounts (typical hardware wallet behavior)
    println!("4. Deriving accounts:");
    for i in 0..3 {
        let keypair = wallet.derive_keypair(i).expect("Failed to derive keypair");
        println!("   Account {}: {}", i, keypair.address);
        
        // Validate each address
        assert!(keypair.address.starts_with('G'));
        assert_eq!(keypair.address.len(), 56);
    }
    
    // 5. Test default address functionality
    let default_addr = wallet.get_default_address().expect("Failed to get default address");
    println!("5. Default address: {}", default_addr);
    
    println!("=== Hardware Wallet Integration Test Completed Successfully ===\n");
}

// Test specific to embedded/no-std environment constraints
#[cfg(not(feature = "std"))]
#[test]
fn test_no_std_compatibility() {
    // This test ensures the wallet works in no-std environment
    let seed = [42u8; 64]; // Simple test seed
    let wallet = StellarWallet::from_seed(seed);
    
    let keypair = wallet.derive_keypair(0).expect("Failed to derive in no-std");
    
    assert!(keypair.address.starts_with('G'));
    assert_eq!(keypair.address.len(), 56);
    
    println!("✓ No-std compatibility test passed");
}