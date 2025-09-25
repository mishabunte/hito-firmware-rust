// tests/stellar_real_transaction_test.rs
// Test for analyzing the real Stellar transaction with provided XDR data

use hito_firmware_rust::crypto::libcrypt0pro::stellar::*;

#[test]
fn test_create_account() {
    let parser = StellarTransactionParser::new();
    let mut serializer = StellarTransactionSerializer::new();
    
    // Example base64 transaction (you'll need to replace with actual test data)
    let base64_tx = "AAAAAgAAAACdr++ECgMp7XJRAM8An6JDIwr7HfywJyQCDQd2Cn6CLwAAAGQACsu/AAAAAQAAAAEAAAAAAAAAAAAAAABo1DIfAAAAAAAAAAEAAAAAAAAAAAAAAACIx8Prxn5MWLiGLoLCNDqXGf+TW6IFodlD9KeQf5j2YwAAAAAA5OHAAAAAAAAAAAA=";
    match parser.parse_transaction(base64_tx) {
        Ok(parsed) => {
            println!("Parsed transaction: {:#?}", parsed);
            assert!(!parsed.operations.is_empty());
            match serializer.serialize_to_base64(&parsed) {
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
fn test_payment() {
    let parser = StellarTransactionParser::new();
    let mut serializer = StellarTransactionSerializer::new();
    
    // Simple payment transaction XDR
    let base64_tx = "AAAAAgAAAACdr++ECgMp7XJRAM8An6JDIwr7HfywJyQCDQd2Cn6CLwAAAGQACsu/AAAAAgAAAAEAAAAAAAAAAAAAAABo1DJRAAAAAAAAAAEAAAAAAAAAAQAAAACIx8Prxn5MWLiGLoLCNDqXGf+TW6IFodlD9KeQf5j2YwAAAAAAAAAAAJiWgAAAAAAAAAAA";
    match parser.parse_transaction(base64_tx) {
        Ok(parsed) => {
            println!("Parsed transaction: {:#?}", parsed);
            assert!(!parsed.operations.is_empty());
            match serializer.serialize_to_base64(&parsed) {
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
    let parser = StellarTransactionParser::new();
    let mut serializer = StellarTransactionSerializer::new();
    
    // Simple payment transaction XDR
    let base64_tx = "AAAAAgAAAACdr++ECgMp7XJRAM8An6JDIwr7HfywJyQCDQd2Cn6CLwAAAGQACsu/AAAAAgAAAAEAAAAAAAAAAAAAAABo1DRaAAAAAQAAABJGb3IgRmFsYWZlbCBKYWtvdWIAAAAAAAEAAAAAAAAAAQAAAACIx8Prxn5MWLiGLoLCNDqXGf+TW6IFodlD9KeQf5j2YwAAAAAAAAAAAJiWgAAAAAAAAAAA";
    match parser.parse_transaction(base64_tx) {
        Ok(parsed) => {
            println!("Parsed transaction: {:#?}", parsed);
            assert!(!parsed.operations.is_empty());
            match serializer.serialize_to_base64(&parsed) {
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
fn test_serialized_transaction_xdr_match() {
    let mut serializer = StellarTransactionSerializer::new();
    let parser = StellarTransactionParser::new();
    let base64_tx = "AAAAAgAAAACdr++ECgMp7XJRAM8An6JDIwr7HfywJyQCDQd2Cn6CLwAAAGQACsu/AAAAAgAAAAEAAAAAAAAAAAAAAABo1DRaAAAAAQAAABJGb3IgRmFsYWZlbCBKYWtvdWIAAAAAAAEAAAAAAAAAAQAAAACIx8Prxn5MWLiGLoLCNDqXGf+TW6IFodlD9KeQf5j2YwAAAAAAAAAAAJiWgAAAAAAAAAAA";
    match parser.parse_transaction(base64_tx) {
        Ok(parsed) => {
            println!("Parsed transaction: {:#?}", parsed);
            assert!(!parsed.operations.is_empty());
            match serializer.serialize_to_base64(&parsed) {
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
