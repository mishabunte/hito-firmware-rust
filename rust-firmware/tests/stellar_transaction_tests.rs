// tests/stellar_real_transaction_test.rs
// Test for analyzing the real Stellar transaction with provided XDR data

const TEST_SEED: &str = "38b6a363e88b28138cc71f0145ab429c251baa8cd8fa6d80bcfb39c35076f1766e24dfc01ce0e22e8dfec185ad7a67ce748cd6551ad1b738619b8859808bbf88";

mod stellar_transaction_tests {
    use hito_firmware_rust::crypto::{crypt0::{bytes_to_hex, hex_to_bytes}, libcrypt0pro::stellar::{NETWORK_ID_MAINNET, NETWORK_ID_TESTNET, OperationDetails, ParsedAsset, ParsedMuxedAccount, StellarTransactionParser, StellarTransactionSerializer, StellarTransactionSigner, StellarWallet, TransactionEnvelope}};

    use crate::TEST_SEED;

    #[test]
    fn test_create_account() {
        let base64_tx = "AAAAAgAAAACdr++ECgMp7XJRAM8An6JDIwr7HfywJyQCDQd2Cn6CLwAAAGQACsu/AAAAAQAAAAEAAAAAAAAAAAAAAABo1DIfAAAAAAAAAAEAAAAAAAAAAAAAAACIx8Prxn5MWLiGLoLCNDqXGf+TW6IFodlD9KeQf5j2YwAAAAAA5OHAAAAAAAAAAAA=";
        match StellarTransactionParser::parse_transaction(base64_tx, NETWORK_ID_TESTNET) {
            Ok(envelope) => {
                println!("Parsed transaction: {:#?}", envelope);
                match &envelope {
                    TransactionEnvelope::Transaction(tx) => {
                      assert_eq!(tx.operations.len(), 1);
                      assert_eq!(tx.source_account, "GCO2734EBIBST3LSKEAM6AE7UJBSGCX3DX6LAJZEAIGQO5QKP2BC7NZ4");
                      assert!(tx.network_hash.is_some());
                      let operation = &tx.operations[0];
                      match &operation.details {
                          OperationDetails::CreateAccount { starting_balance, destination } => {
                              assert_eq!(*starting_balance, 15000000); // 1.5 XLM in stroops
                              assert_eq!(destination, "GCEMPQ7LYZ7EYWFYQYXIFQRUHKLRT74TLORALIOZIP2KPED7TD3GG352");
                          },
                          _ => panic!("Expected CreateAccount operation"),
                      }
                      match StellarTransactionSerializer::serialize_to_base64(&envelope) {
                          Ok(serialized_xdr) => {
                              println!("Serialized XDR: {}", serialized_xdr);
                              assert_eq!(serialized_xdr, base64_tx);
                          }
                          Err(e) => panic!("Serialization error: {}", e),
                      }
                    }
                    _ => panic!("Expected TransactionEnvelope::Transaction"),
                }
            }
            Err(e) => panic!("Parse error: {}", e),
        }
    }

    #[test]
    fn test_payment() {    
        // Simple payment transaction XDR
        let base64_tx = "AAAAAgAAAACdr++ECgMp7XJRAM8An6JDIwr7HfywJyQCDQd2Cn6CLwAAAGQACsu/AAAAAgAAAAEAAAAAAAAAAAAAAABo1DJRAAAAAAAAAAEAAAAAAAAAAQAAAACIx8Prxn5MWLiGLoLCNDqXGf+TW6IFodlD9KeQf5j2YwAAAAAAAAAAAJiWgAAAAAAAAAAA";
        match StellarTransactionParser::parse_transaction(base64_tx, NETWORK_ID_TESTNET) {
            Ok(envelope) => {
                println!("Parsed transaction: {:#?}", envelope);
                match &envelope {
                    TransactionEnvelope::Transaction(tx) => {
                      assert_eq!(tx.operations.len(), 1);
                      assert_eq!(tx.source_account, "GCO2734EBIBST3LSKEAM6AE7UJBSGCX3DX6LAJZEAIGQO5QKP2BC7NZ4");
                      assert!(tx.network_hash.is_some());
                      let operation = &tx.operations[0];
                      match &operation.details {
                          OperationDetails::Payment { amount, destination, asset } => {
                              assert_eq!(*amount, 10000000); // 1 XLM
                              match destination {
                                  ParsedMuxedAccount::Ed25519 { account_id } => {
                                      assert_eq!(account_id, "GCEMPQ7LYZ7EYWFYQYXIFQRUHKLRT74TLORALIOZIP2KPED7TD3GG352");
                                  },
                                  _ => panic!("Expected Ed25519 destination account"),
                              }
                              match asset {
                                  ParsedAsset::Native => {},
                                  _ => panic!("Expected Native asset"),
                              }
                          },
                          _ => panic!("Expected Payment operation"),
                      }
                      match StellarTransactionSerializer::serialize_to_base64(&envelope) {
                          Ok(serialized_xdr) => {
                              println!("Serialized XDR: {}", serialized_xdr);
                              assert_eq!(serialized_xdr, base64_tx);
                          }
                          Err(e) => panic!("Serialization error: {}", e),
                      }
                    }
                    _ => panic!("Expected TransactionEnvelope::Transaction"),
                }
            }
            Err(e) => panic!("Parse error: {}", e),
        }
    }

    #[test]
    fn test_payment_with_memo() {
        // Simple payment transaction XDR
        let base64_tx = "AAAAAgAAAACdr++ECgMp7XJRAM8An6JDIwr7HfywJyQCDQd2Cn6CLwAAAGQACsu/AAAAAgAAAAEAAAAAAAAAAAAAAABo1DRaAAAAAQAAABJGb3IgRmFsYWZlbCBKYWtvdWIAAAAAAAEAAAAAAAAAAQAAAACIx8Prxn5MWLiGLoLCNDqXGf+TW6IFodlD9KeQf5j2YwAAAAAAAAAAAJiWgAAAAAAAAAAA";
        match StellarTransactionParser::parse_transaction(base64_tx, NETWORK_ID_TESTNET) {
            Ok(envelope) => {
                println!("Parsed transaction: {:#?}", envelope);
                match &envelope {
                    TransactionEnvelope::Transaction(tx) => {
                      assert!(tx.memo.is_some());
                      assert_eq!(tx.operations.len(), 1);
                      assert_eq!(tx.source_account, "GCO2734EBIBST3LSKEAM6AE7UJBSGCX3DX6LAJZEAIGQO5QKP2BC7NZ4");
                      assert!(tx.network_hash.is_some());
                      let memo = tx.memo.clone().unwrap();
                      assert!(memo.value.is_some());
                      let memo = memo.value.unwrap();
                      assert_eq!(memo, "For Falafel Jakoub");
                      let operation = &tx.operations[0];
                      match &operation.details {
                          OperationDetails::Payment { amount, destination, asset } => {
                              assert_eq!(*amount, 10000000); // 1 XLM
                              match destination {
                                  ParsedMuxedAccount::Ed25519 { account_id } => {
                                      assert_eq!(account_id, "GCEMPQ7LYZ7EYWFYQYXIFQRUHKLRT74TLORALIOZIP2KPED7TD3GG352");
                                  },
                                  _ => panic!("Expected Ed25519 destination account"),
                              }
                              match asset {
                                  ParsedAsset::Native => {},
                                  _ => panic!("Expected Native asset"),
                              }
                          },
                          _ => panic!("Expected Payment operation"),
                      }
                      match StellarTransactionSerializer::serialize_to_base64(&envelope) {
                          Ok(serialized_xdr) => {
                              println!("Serialized XDR: {}", serialized_xdr);
                              assert_eq!(serialized_xdr, base64_tx);
                          }
                          Err(e) => panic!("Serialization error: {}", e),
                      }
                    }
                    _ => panic!("Expected TransactionEnvelope::Transaction"),
                }
            }
            Err(e) => panic!("Parse error: {}", e),
        }
    }

    #[test]
    fn test_transaction_asset_usdc_fee_bump() {
      let network_hash = NETWORK_ID_MAINNET;
      let base64_tx = "AAAABQAAAABUkFG1pkUgljfGVUjXpVHZawpNT9YdZo4NUmSBYaI+AQAAAAAAB3twAAAAAgAAAABUkFG1pkUgljfGVUjXpVHZawpNT9YdZo4NUmSBYaI+AQADvbgAAAOiAAAB9QAAAAEAAAAAAAAAAAAAAABpXwG6AAAAAAAAAAEAAAABAAAAAFSQUbWmRSCWN8ZVSNelUdlrCk1P1h1mjg1SZIFhoj4BAAAAAQAAAABUkFG1pkUgljfGVUjXpVHZawpNT9YdZo4NUmSBYaI+AQAAAAFVU0RDAAAAAEI+fQXy7K+/7BkrIVo/G+lq7bjY5wJUq+NBPgIH3layAAAAAACYloAAAAAAAAAAAWGiPgEAAABAOA+f4aVXwAAmmEH4RsiwkgiY/5zrHYUClvxfAzO7k+U5QxQAIuJxlWh8xQWeE3lzluCt0DZ2J2ARMCc2AK5XCAAAAAAAAAABYaI+AQAAAEBaRhhluwSrGvGW367XS0dyXqd2cmaMLNa2ARSxYuzYPymYoctNU4MoeMLiwUu6JxstBgUf95bDNm6ADfuVDyIL";
      match StellarTransactionParser::parse_transaction(base64_tx, network_hash) {
          Ok(envelope) => {
            match &envelope {
                TransactionEnvelope::Transaction(_) => {
                  panic!("Expected FeeBump transaction, got Transaction");
                }
                TransactionEnvelope::FeeBump(fee_bump) => {
                  println!("Parsed transaction: {:#?}", fee_bump);
                  assert!(!fee_bump.inner_tx.operations.is_empty());
                  match StellarTransactionSerializer::serialize_to_base64(&envelope) {
                      Ok(serialized_xdr) => {
                          println!("Serialized XDR: {}", serialized_xdr);
                          assert_eq!(serialized_xdr, base64_tx);
                      }
                      Err(e) => panic!("Serialization error: {}", e),
                  }
                }
            }
          }
          Err(e) => panic!("Parse error: {}", e),
      }
    }

    #[test]
    fn test_transaction_sign() {
        let network_hash = NETWORK_ID_TESTNET;
        let base64_tx = "AAAAAgAAAACIx8Prxn5MWLiGLoLCNDqXGf+TW6IFodlD9KeQf5j2YwAAAGQACsu/AAAAAgAAAAEAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAEAAAAAAAAAAQAAAACdr++ECgMp7XJRAM8An6JDIwr7HfywJyQCDQd2Cn6CLwAAAAAAAAAABfXhAAAAAAAAAAAA";
        let wallet = StellarWallet::from_seed(hex_to_bytes(TEST_SEED).unwrap().try_into().unwrap());
        let keypair = wallet.derive_keypair(0).expect("Failed to derive keypair");
        let secret_encoded = StellarWallet::encode_stellar_secret(&keypair.secret_key).expect("Failed to encode secret key");
        println!("Derived secret key address: {}", secret_encoded);
        println!("Derived keypair: public={}, secret={}", bytes_to_hex(&keypair.public_key), bytes_to_hex(&keypair.secret_key));
        match StellarTransactionParser::parse_transaction(&base64_tx, network_hash) {
            Ok(envelope) => {
              let signed_tx = StellarTransactionSigner::sign_transaction(&envelope, keypair).expect("Failed to sign transaction");
              let serialized_tx = StellarTransactionSerializer::serialize_to_base64(&signed_tx).expect("Failed to serialize signed transaction");
              println!("Signed and serialized transaction: {}", serialized_tx);
              println!("Signed transaction: {:#?}", signed_tx);
              let signature_hex = match &signed_tx {
                  TransactionEnvelope::Transaction(tx_envelope) => {
                      if tx_envelope.signatures.is_empty() {
                          panic!("No signatures found in signed transaction");
                      }
                      tx_envelope.signatures[0].signature.clone()
                  },
                  _ => panic!("Expected TransactionEnvelope::Transaction"),
              };
              let signature_expected = "90946182bd41326335d65328a330c077f562064b4229ef4fc16011f97a593b826084106d67666b3196e5d5e9ebd5f8b618e8efec28e85ab7a63b2a28800c860c";
              assert_eq!(signature_hex, signature_expected);
            }
            Err(e) => panic!("Parse error: {}", e),
        }
    }
}
