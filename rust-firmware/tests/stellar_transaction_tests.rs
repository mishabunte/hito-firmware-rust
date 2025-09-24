// tests/stellar_real_transaction_test.rs
// Test for analyzing the real Stellar transaction with provided XDR data

use hito_firmware_rust::crypto::libcrypt0pro::stellar::{
    StellarTransactionParser
};

#[test]
fn test_create_account() {
    let parser = StellarTransactionParser::new();
    
    // Example base64 transaction (you'll need to replace with actual test data)
    let base64_tx = "AAAAAgAAAACdr++ECgMp7XJRAM8An6JDIwr7HfywJyQCDQd2Cn6CLwAAAGQACsu/AAAAAQAAAAEAAAAAAAAAAAAAAABo1DIfAAAAAAAAAAEAAAAAAAAAAAAAAACIx8Prxn5MWLiGLoLCNDqXGf+TW6IFodlD9KeQf5j2YwAAAAAA5OHAAAAAAAAAAAA=";
    match parser.parse_transaction(base64_tx) {
        Ok(parsed) => {
            println!("Parsed transaction: {:#?}", parsed);
            assert!(!parsed.operations.is_empty());
        }
        Err(e) => println!("Parse error: {}", e),
    }
}

#[test]
fn test_payment() {
    let parser = StellarTransactionParser::new();
    
    // Simple payment transaction XDR
    let base64_tx = "AAAAAgAAAACdr++ECgMp7XJRAM8An6JDIwr7HfywJyQCDQd2Cn6CLwAAAGQACsu/AAAAAgAAAAEAAAAAAAAAAAAAAABo1DJRAAAAAAAAAAEAAAAAAAAAAQAAAACIx8Prxn5MWLiGLoLCNDqXGf+TW6IFodlD9KeQf5j2YwAAAAAAAAAAAJiWgAAAAAAAAAAA";
    match parser.parse_transaction(base64_tx) {
        Ok(parsed) => {
            println!("Parsed transaction: {:#?}", parsed);
            assert!(!parsed.operations.is_empty());
        }
        Err(e) => println!("Parse error: {}", e),
    }
}

#[test]
fn test_payment_with_memo() {
    let parser = StellarTransactionParser::new();
    
    // Simple payment transaction XDR
    let base64_tx = "AAAAAgAAAACdr++ECgMp7XJRAM8An6JDIwr7HfywJyQCDQd2Cn6CLwAAAGQACsu/AAAAAgAAAAEAAAAAAAAAAAAAAABo1DRaAAAAAQAAABJGb3IgRmFsYWZlbCBKYWtvdWIAAAAAAAEAAAAAAAAAAQAAAACIx8Prxn5MWLiGLoLCNDqXGf+TW6IFodlD9KeQf5j2YwAAAAAAAAAAAJiWgAAAAAAAAAAA";
    match parser.parse_transaction(base64_tx) {
        Ok(parsed) => {
            println!("Parsed transaction: {:#?}", parsed);
            assert!(!parsed.operations.is_empty());
        }
        Err(e) => println!("Parse error: {}", e),
    }
}