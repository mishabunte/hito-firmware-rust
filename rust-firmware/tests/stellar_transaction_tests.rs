// tests/stellar_real_transaction_test.rs
// Test for analyzing the real Stellar transaction with provided XDR data

use hito_firmware_rust::crypto::libcrypt0pro::stellar::*;
use hito_firmware_rust::crypto::ffi::{crypt0_sha256, crypt0_ed25519_sign};
use hito_firmware_rust::crypto::crypt0::{bytes_to_hex, hex_to_bytes};

const TEST_SEED: &str = "38b6a363e88b28138cc71f0145ab429c251baa8cd8fa6d80bcfb39c35076f1766e24dfc01ce0e22e8dfec185ad7a67ce748cd6551ad1b738619b8859808bbf88";


#[test]
fn test_create_account() {
    
    //let mut serializer = StellarTransactionSerializer::new();
    
    // Example base64 transaction (you'll need to replace with actual test data)
    let base64_tx = "AAAAAgAAAACdr++ECgMp7XJRAM8An6JDIwr7HfywJyQCDQd2Cn6CLwAAAGQACsu/AAAAAQAAAAEAAAAAAAAAAAAAAABo1DIfAAAAAAAAAAEAAAAAAAAAAAAAAACIx8Prxn5MWLiGLoLCNDqXGf+TW6IFodlD9KeQf5j2YwAAAAAA5OHAAAAAAAAAAAA=";
    match StellarTransactionParser::parse_transaction(base64_tx) {
        Ok(parsed) => {
            println!("Parsed transaction: {:#?}", parsed);
            assert!(!parsed.operations.is_empty());
            // match serializer.serialize_to_base64(&parsed) {
            //     Ok(serialized_xdr) => {
            //         println!("Serialized XDR: {}", serialized_xdr);
            //         assert_eq!(serialized_xdr, base64_tx);
            //     }
            //     Err(e) => println!("Serialization error: {}", e),
            // }
        }
        Err(e) => println!("Parse error: {}", e),
    }
}

#[test]
fn test_payment() {    
    // Simple payment transaction XDR
    let base64_tx = "AAAAAgAAAACdr++ECgMp7XJRAM8An6JDIwr7HfywJyQCDQd2Cn6CLwAAAGQACsu/AAAAAgAAAAEAAAAAAAAAAAAAAABo1DJRAAAAAAAAAAEAAAAAAAAAAQAAAACIx8Prxn5MWLiGLoLCNDqXGf+TW6IFodlD9KeQf5j2YwAAAAAAAAAAAJiWgAAAAAAAAAAA";
    match StellarTransactionParser::parse_transaction(base64_tx) {
        Ok(parsed) => {
            println!("Parsed transaction: {:#?}", parsed);
            assert!(!parsed.operations.is_empty());
            match StellarTransactionSerializer::serialize_to_base64(&parsed) {
                Ok(serialized_xdr) => {
                    println!("Serialized XDR: {}", serialized_xdr);
                    assert_eq!(serialized_xdr, base64_tx);
                }
                Err(e) => println!("Serialization error: {}", e),
            }
        }
        Err(e) => println!("Parse error: {}", e),
    }
}

#[test]
fn test_payment_with_memo() {
    
    //let mut serializer = StellarTransactionSerializer::new();
    
    // Simple payment transaction XDR
    let base64_tx = "AAAAAgAAAACdr++ECgMp7XJRAM8An6JDIwr7HfywJyQCDQd2Cn6CLwAAAGQACsu/AAAAAgAAAAEAAAAAAAAAAAAAAABo1DRaAAAAAQAAABJGb3IgRmFsYWZlbCBKYWtvdWIAAAAAAAEAAAAAAAAAAQAAAACIx8Prxn5MWLiGLoLCNDqXGf+TW6IFodlD9KeQf5j2YwAAAAAAAAAAAJiWgAAAAAAAAAAA";
    match StellarTransactionParser::parse_transaction(base64_tx) {
        Ok(parsed) => {
            println!("Parsed transaction: {:#?}", parsed);
            assert!(!parsed.operations.is_empty());
            match StellarTransactionSerializer::serialize_to_base64(&parsed) {
                Ok(serialized_xdr) => {
                    println!("Serialized XDR: {}", serialized_xdr);
                    assert_eq!(serialized_xdr, base64_tx);
                }
                Err(e) => println!("Serialization error: {}", e),
            }
        }
        Err(e) => println!("Parse error: {}", e),
    }
}

#[test]
fn test_transaction_sign() {
    let network_hash = NETWORK_ID_MAINNET;
    let base64_tx = network_hash.to_string() + ":AAAAAgAAAACIx8Prxn5MWLiGLoLCNDqXGf+TW6IFodlD9KeQf5j2YwAAAGQACsu/AAAAAgAAAAEAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAEAAAAAAAAAAQAAAACdr++ECgMp7XJRAM8An6JDIwr7HfywJyQCDQd2Cn6CLwAAAAAAAAAABfXhAAAAAAAAAAAA";
    let wallet = StellarWallet::from_seed(hex_to_bytes(TEST_SEED).unwrap().try_into().unwrap());
    let keypair = wallet.derive_keypair(0).expect("Failed to derive keypair");
    let secret_encoded = StellarWallet::encode_stellar_secret(&keypair.secret_key).expect("Failed to encode secret key");
    println!("Derived secret key address: {}", secret_encoded);
    println!("Derived keypair: public={}, secret={}", bytes_to_hex(&keypair.public_key), bytes_to_hex(&keypair.secret_key));
    match StellarTransactionParser::parse_transaction(&base64_tx) {
        Ok(parsed) => {
            println!("Parsed transaction: {:#?}", parsed);
            assert!(!parsed.operations.is_empty());
            let sig_base = StellarTransactionSerializer::build_signature_base(&parsed).expect("sig_base");
            let sha256_sig_base = unsafe {
                let mut hash = [0u8; 32];
                let res = crypt0_sha256(
                    sig_base.as_ptr(),
                    sig_base.len(),
                    hash.as_mut_ptr(),
                    hash.len(),
                );
                if !res {
                    panic!("crypt0_sha256 failed");
                }
                hash
            };
            println!("sha256 base (hex): {}", bytes_to_hex(&sha256_sig_base));
            let signature = unsafe {
                let mut sig = [0u8; 64];
                let privkey_bytes = keypair.secret_key;
                let pubkey_bytes = keypair.public_key;
                let res = crypt0_ed25519_sign(
                    sha256_sig_base.as_ptr(),
                    sha256_sig_base.len(),
                    privkey_bytes.as_ptr(),
                    privkey_bytes.len(),
                    pubkey_bytes.as_ptr(),
                    pubkey_bytes.len(),
                    sig.as_mut_ptr(),
                    sig.len()
                );
                if res != 0 {
                    panic!("crypt0_ed25519_sign failed");
                }
                sig
            };
            println!("Signature base (hex): {}", bytes_to_hex(&sig_base));
            println!("Signature (hex): {}", bytes_to_hex(&signature));
            let parsed_signature = ParsedSignature {
                hint: {
                    let mut hint = [0u8; 4];
                    hint.copy_from_slice(&keypair.public_key[28..32]);
                    hint
                },
                signature,
            };
            let mut parsed = parsed;
            parsed.signatures.push(parsed_signature);
            match StellarTransactionSerializer::serialize_to_base64(&parsed) {
                Ok(serialized_xdr) => {
                    let to_compare = "AAAAAgAAAACIx8Prxn5MWLiGLoLCNDqXGf+TW6IFodlD9KeQf5j2YwAAAGQACsu/AAAAAgAAAAEAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAEAAAAAAAAAAQAAAACdr++ECgMp7XJRAM8An6JDIwr7HfywJyQCDQd2Cn6CLwAAAAAAAAAABfXhAAAAAAAAAAABf5j2YwAAAECQ6Pu11cAb3ApUkrMGAjZzdBdvWUokA3y7C925hkA1x7nyLRplRzVXMPpDuN/7/UOGcEjBLtSgOL8rUtet8PgF";
                    assert_eq!(serialized_xdr, to_compare);
                }
                Err(e) => println!("Serialization error: {}", e),
            }
        }
        Err(e) => println!("Parse error: {}", e),
    }
}

// #[test]
// fn test_serialized_transaction_xdr_match() {
//     let mut serializer = StellarTransactionSerializer::new();
//     
//     let base64_tx = "AAAAAgAAAACdr++ECgMp7XJRAM8An6JDIwr7HfywJyQCDQd2Cn6CLwAAAGQACsu/AAAAAgAAAAEAAAAAAAAAAAAAAABo1DRaAAAAAQAAABJGb3IgRmFsYWZlbCBKYWtvdWIAAAAAAAEAAAAAAAAAAQAAAACIx8Prxn5MWLiGLoLCNDqXGf+TW6IFodlD9KeQf5j2YwAAAAAAAAAAAJiWgAAAAAAAAAAA";
//     match StellarTransactionParser::parse_transaction(base64_tx) {
//         Ok(parsed) => {
//             println!("Parsed transaction: {:#?}", parsed);
//             assert!(!parsed.operations.is_empty());
//             match serializer.serialize_to_base64(&parsed) {
//                 Ok(serialized_xdr) => {
//                     println!("Serialized XDR: {}", serialized_xdr);
//                     assert_eq!(serialized_xdr, base64_tx);
//                 }
//                 Err(e) => println!("Serialization error: {}", e),
//             }
//         }
//         Err(e) => println!("Parse error: {}", e),
//     }
// }
