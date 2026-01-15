extern crate alloc;
use alloc::{string::String, vec::Vec, format};

use base64ct::{Base64, Decoder, Encoding};

use crate::{crypto::{crypt0::{bytes_to_hex, hex_to_bytes}, libcrypt0pro::stellar::{StellarWallet }}, log_info};
use super::*;
use super::xdr::*;

//use base64ct::{Base64, Encoding};

pub struct StellarTransactionParser;

impl StellarTransactionParser {
    pub fn new() -> Self {
        Self
    }

    fn decode_base64(base64_data: &str) -> Result<Vec<u8>, StellarTransactionError> {
        let mut buf = [0u8; MAX_XDR_LEN * 4 / 3 + 4]; // Base64 expansion ratio
        //log_info!("Decoding base64 data: {}", base64_data);
        let encoded = Base64::decode(base64_data.as_bytes(), &mut buf)
            .map_err(|_| StellarTransactionError::Base64Error)?;
        //log_info!("Decoded base64 length: {}", encoded.len());
        Ok(encoded.to_vec())
    }

    fn parse_fee_bump_transaction_xdr(xdr_bytes: &[u8], network_hash: &str) -> Result<FeeBumpTransaction, StellarTransactionError> {
        if xdr_bytes.len() > MAX_XDR_LEN {
            return Err(StellarTransactionError::PayloadTooLarge);
        }
        if xdr_bytes.len() < 20 {
            return Err(StellarTransactionError::PayloadTooSmall);
        }

        let mut xr = Xdr::new(xdr_bytes);

        let envelope_type = xr.read_i32()?;
        if envelope_type != ENVELOPE_TYPE_TX_FEE_BUMP {
            return Err(StellarTransactionError::InvalidEnvelopeType);
        }

        let fee_bump_tx = Self::parse_fee_bump_envelope(&mut xr);
        let mut parsed_tx = fee_bump_tx?;
        use alloc::string::ToString;
        parsed_tx.inner_tx.network_hash = Some(Network::new(
            match network_hash {
                NETWORK_ID_MAINNET => NetworkId::Mainnet,
                NETWORK_ID_TESTNET => NetworkId::Testnet,
                NETWORK_ID_FUTURENET => NetworkId::Futurenet,
                _ => return Err(StellarTransactionError::InvalidNetworkId),
            },
            network_hash.to_string(),
        ));
        Ok(parsed_tx)
    }

    fn parse_transaction_xdr(xdr_bytes: &[u8], network_hash: &str) -> Result<Transaction, StellarTransactionError> {
        if xdr_bytes.len() > MAX_XDR_LEN {
            return Err(StellarTransactionError::PayloadTooLarge);
        }
        if xdr_bytes.len() < 20 {
            return Err(StellarTransactionError::PayloadTooSmall);
        }

        let mut xr = Xdr::new(xdr_bytes);

        let envelope_type = xr.read_i32()?;
        let tx = match envelope_type {
            ENVELOPE_TYPE_TX_V0 => Self::parse_v0_envelope(&mut xr),
            ENVELOPE_TYPE_TX => Self::parse_v1_envelope(&mut xr),
            //ENVELOPE_TYPE_TX_FEE_BUMP => Self::parse_fee_bump_envelope(&mut xr),
            _ => Err(StellarTransactionError::InvalidEnvelopeType),
        };
        let mut parsed_tx = tx?;
        use alloc::string::ToString;
        parsed_tx.network_hash = Some(Network::new(
            match network_hash {
                NETWORK_ID_MAINNET => NetworkId::Mainnet,
                NETWORK_ID_TESTNET => NetworkId::Testnet,
                NETWORK_ID_FUTURENET => NetworkId::Futurenet,
                _ => return Err(StellarTransactionError::InvalidNetworkId),
            },
            network_hash.to_string(),
        ));
        Ok(parsed_tx)
    }

    /// Parse a base64-encoded transaction envelope into structured data
    pub fn parse_transaction(tx_data: &str, network_hash: &str) -> Result<TransactionEnvelope, StellarTransactionError> {
        let xdr_bytes = Self::decode_base64(tx_data)?;
        
        // Peek at envelope type to determine which parser to use
        if xdr_bytes.len() < 4 {
            return Err(StellarTransactionError::PayloadTooSmall);
        }
        let envelope_type = i32::from_be_bytes([xdr_bytes[0], xdr_bytes[1], xdr_bytes[2], xdr_bytes[3]]);
        
        match envelope_type {
            ENVELOPE_TYPE_TX_FEE_BUMP => {
                let fee_bump = Self::parse_fee_bump_transaction_xdr(&xdr_bytes, network_hash)?;
                Ok(TransactionEnvelope::FeeBump(fee_bump))
            }
            _ => {
                let tx = Self::parse_transaction_xdr(&xdr_bytes, network_hash)?;
                Ok(TransactionEnvelope::Transaction(tx))
            }
        }
    }

    fn uint256_to_bounded_hex(v: &[u8]) -> Result<String, StellarTransactionError> {
        let hex = bytes_to_hex(v);
        if hex.len() > MAX_STRING_LEN {
            return Err(StellarTransactionError::StringTooLong);
        }
        Ok(hex)
    }

    fn u8_array_to_stellar_address(key: &[u8]) -> Result<String, StellarTransactionError> {
        // let hex = Self::uint256_to_bounded_hex(key);
        // if hex.is_err() {
        //     return Err(StellarTransactionError::AccountIdError);
        // }
        let mut key_array = [0u8; 32];
        key_array.copy_from_slice(key);
        let address = StellarWallet::encode_stellar_address(&key_array);
        if address.is_err() {
            return Err(StellarTransactionError::AccountIdError);
        }
        let address = address.unwrap();
        Ok(address)
    }

    fn parse_v0_envelope(xr: &mut Xdr) -> Result<Transaction, StellarTransactionError> {
        // TransactionV0
        let source = xr.read_fixed_opaque(32)?; // uint256
        let fee = xr.read_u32()? as i64;
        let seq_num = xr.read_i64()?;

        // TimeBounds* timeBounds;
        let has_tb = xr.read_bool()?;
        let time_bounds = if has_tb {
            Some(Self::parse_time_bounds(xr)?)
        } else {
            None
        };

        let memo = Self::parse_memo(xr)?;

        let operations = Self::parse_operations(xr)?;

        // ext (int v = 0; void)
        let _ext = xr.read_i32()?; // should be 0

        let signatures = Self::parse_signatures(xr)?;
        
        Ok(Transaction {
            network_hash: None,
            source_account: Self::u8_array_to_stellar_address(source)?,
            sequence_number: seq_num,
            fee,
            memo: Some(memo),
            time_bounds,
            ledger_bounds: None,
            operations,
            signatures,
            envelope_type: TransactionEnvelopeType::TxV0,
        })
    }

    fn parse_v1_envelope(xr: &mut Xdr) -> Result<Transaction, StellarTransactionError> {
        // Transaction
        let source = Self::parse_muxed_account(xr)?;
        let fee = xr.read_u32()? as i64;
        let seq_num = xr.read_i64()?;

        let (time_bounds, ledger_bounds) = Self::parse_preconditions(xr)?;

        let memo = Self::parse_memo(xr)?;
        let operations = Self::parse_operations(xr)?;

        // tx ext (int v; case 0: void)
        let _ext = xr.read_i32()?; // 0

        let signatures = Self::parse_signatures(xr)?;

        Ok(Transaction {
            network_hash: None,
            source_account: Self::muxed_account_to_string(&source)?,
            sequence_number: seq_num,
            fee,
            memo: Some(memo),
            time_bounds,
            ledger_bounds,
            operations,
            signatures,
            envelope_type: TransactionEnvelopeType::Tx,
        })
    }

    fn parse_fee_bump_envelope(xr: &mut Xdr) -> Result<FeeBumpTransaction, StellarTransactionError> {
        // FeeBumpTransaction
        let fee_source = Self::parse_muxed_account(xr)?;
        let fee = xr.read_i64()?;

        // innerTx
        let inner_type = xr.read_i32()?;
        if inner_type != ENVELOPE_TYPE_TX {
            return Err(StellarTransactionError::InvalidEnvelopeType);
        }

        // Inner TransactionV1Envelope (tx + signatures)
        // Reuse same logic as v1 envelope, but we need to parse into temp first
        let inner_tx = {
            let source = Self::parse_muxed_account(xr)?;
            let fee_inner = xr.read_u32()? as i64;
            let seq_num = xr.read_i64()?;
            let (time_bounds, ledger_bounds) = Self::parse_preconditions(xr)?;
            let memo = Self::parse_memo(xr)?;
            let operations = Self::parse_operations(xr)?;
            let _ext = xr.read_i32()?; // tx ext v=0
            let inner_signatures = Self::parse_signatures(xr)?;

            Transaction {
                network_hash: None,
                source_account: Self::muxed_account_to_string(&source)?,
                sequence_number: seq_num,
                fee: fee_inner,
                memo: Some(memo),
                time_bounds,
                ledger_bounds,
                operations,
                signatures: inner_signatures,
                envelope_type: TransactionEnvelopeType::Tx,
            }
        };

        // fee bump ext
        let _ext = xr.read_i32()?; // 0

        let outer_sigs = Self::parse_signatures(xr)?;

        Ok(FeeBumpTransaction {
            fee_source,
            fee,
            inner_tx,
            signatures: outer_sigs,
        })
    }


    fn parse_memo(xr: &mut Xdr) -> Result<Memo, StellarTransactionError> {
        let t = xr.read_i32()?;
        match t {
            MEMO_NONE => Ok(Memo {
                memo_type: String::try_from("none").map_err(|_| StellarTransactionError::StringTooLong)?,
                value: None,
            }),
            MEMO_TEXT => {
                let s = xr.read_string(MAX_STRING_LEN)?;
                Ok(Memo {
                    memo_type: String::try_from("text").map_err(|_| StellarTransactionError::StringTooLong)?,
                    value: Some(String::try_from(s).map_err(|_| StellarTransactionError::StringTooLong)?),
                })
            }
            MEMO_ID => {
                let id = xr.read_u64()?;
                Ok(Memo {
                    memo_type: String::try_from("id").map_err(|_| StellarTransactionError::StringTooLong)?,
                    value: Some(Self::u64_to_string(id)?),
                })
            }
            MEMO_HASH => {
                let h = xr.read_fixed_opaque(32)?;
                Ok(Memo {
                    memo_type: String::try_from("hash").map_err(|_| StellarTransactionError::StringTooLong)?,
                    value: Some(bytes_to_hex(h)),
                })
            }
            MEMO_RETURN => {
                let h = xr.read_fixed_opaque(32)?;
                Ok(Memo {
                    memo_type: String::try_from("return").map_err(|_| StellarTransactionError::StringTooLong)?,
                    value: Some(bytes_to_hex(h)),
                })
            }
            _ => Err(StellarTransactionError::XdrError),
        }
    }

    fn parse_preconditions(xr: &mut Xdr) -> Result<(Option<TimeBounds>, Option<LedgerBounds>), StellarTransactionError> {
        let t = xr.read_i32()?;
        match t {
            PRECOND_NONE => Ok((None, None)),
            PRECOND_TIME => {
                let tb = Self::parse_time_bounds(xr)?;
                Ok((Some(tb), None))
            }
            PRE => {
                let has_tb = xr.read_bool()?;
                let tb = if has_tb {
                    Some(Self::parse_time_bounds(xr)?)
                } else {
                    None
                };
                let has_lb = xr.read_bool()?;
                let lb = if has_lb {
                    Some(Self::parse_ledger_bounds(xr)?)
                } else {
                    None
                };
                // Skip minSeqNum*
                let has_min_seq = xr.read_bool()?;
                if has_min_seq {
                    let _ = xr.read_i64()?; // ignore
                }
                // minSeqAge, minSeqLedgerGap
                let _ = xr.read_u32()?;
                let _ = xr.read_u32()?;
                // extraSigners<2> - we skip them
                let count = xr.read_u32()? as usize;
                for _ in 0..count {
                    let key_type = xr.read_i32()?;
                    match key_type {
                        // All known signer key variants have 32-byte bodies here
                        _ => {
                            let _key = xr.read_fixed_opaque(32)?;
                        }
                    }
                }
                Ok((tb, lb))
            }
            _ => Err(StellarTransactionError::XdrError),
        }
    }
    
    fn parse_time_bounds(xr: &mut Xdr) -> Result<TimeBounds, StellarTransactionError> {
        let min_time = xr.read_u64()?;
        let max_time = xr.read_u64()?;
        Ok(TimeBounds {
            min_time: Some(min_time),
            max_time: Some(max_time),
        })
    }

    fn parse_ledger_bounds(xr: &mut Xdr) -> Result<LedgerBounds, StellarTransactionError> {
        let min_ledger = xr.read_u32()?;
        let max_ledger = xr.read_u32()?;
        Ok(LedgerBounds {
            min_ledger,
            max_ledger,
        })
    }

    fn parse_operations(xr: &mut Xdr) -> Result<Vec<Operation>, StellarTransactionError> {
        let count = xr.read_u32()? as usize;
        if count > MAX_OPERATIONS {
            return Err(StellarTransactionError::TooManyOperations);
        }

        let mut ops = Vec::new();
        for _ in 0..count {
            ops.push(Self::parse_operation(xr)?);
        }
        Ok(ops)
    }

    fn parse_operation(xr: &mut Xdr) -> Result<Operation, StellarTransactionError> {
        // sourceAccount*
        let has_src = xr.read_bool()?;
        let source_account = if has_src {
            let src = Self::parse_muxed_account(xr)?;
            Some(Self::muxed_account_to_string(&src)?)
        } else {
            None
        };

        let op_type = xr.read_i32()?;

        match op_type {
            OP_CREATE_ACCOUNT => {
                let dest = Self::account_id_to_string(xr)?;
                let starting_balance = xr.read_i64()?;
                Ok(Operation {
                    operation_type: String::from("create_account"),
                    source_account,
                    details: OperationDetails::CreateAccount {
                        destination: dest,
                        starting_balance,
                    },
                })
            }

            OP_PAYMENT => {
                let destination = Self::parse_muxed_account(xr)?;
                let asset = Self::parse_asset(xr)?;
                let amount = xr.read_i64()?;
                Ok(Operation {
                    operation_type: String::from("payment"),
                    source_account,
                    details: OperationDetails::Payment {
                        destination,
                        asset,
                        amount,
                    },
                })
            }

            OP_PATH_PAYMENT_STRICT_SEND => {
                let send_asset = Self::parse_asset(xr)?;
                let send_amount = xr.read_i64()?;
                let destination = Self::parse_muxed_account(xr)?;
                let dest_asset = Self::parse_asset(xr)?;
                let dest_min = xr.read_i64()?;
                let path = Self::parse_asset_path(xr)?;
                Ok(Operation {
                    operation_type: String::from("path_payment_strict_send"),
                    source_account,
                    details: OperationDetails::PathPaymentStrictSend {
                        send_asset,
                        send_amount,
                        destination,
                        dest_asset,
                        dest_min,
                        path,
                    },
                })
            }

            OP_PATH_PAYMENT_STRICT_RECEIVE => {
                let send_asset = Self::parse_asset(xr)?;
                let send_max = xr.read_i64()?;
                let destination = Self::parse_muxed_account(xr)?;
                let dest_asset = Self::parse_asset(xr)?;
                let dest_amount = xr.read_i64()?;
                let path = Self::parse_asset_path(xr)?;
                Ok(Operation {
                    operation_type: String::from("path_payment_strict_receive"),
                    source_account,
                    details: OperationDetails::PathPaymentStrictReceive {
                        send_asset,
                        send_max,
                        destination,
                        dest_asset,
                        dest_amount,
                        path,
                    },
                })
            }

            OP_CHANGE_TRUST => {
                let line = Self::parse_change_trust_asset(xr)?;
                let limit = xr.read_i64()?;
                Ok(Operation {
                    operation_type: String::from("change_trust"),
                    source_account,
                    details: OperationDetails::ChangeTrust { asset: line, limit },
                })
            }

            OP_ACCOUNT_MERGE => {
                // union body is just a MuxedAccount
                let dest = Self::parse_muxed_account(xr)?;
                Ok(Operation {
                    operation_type: String::from("account_merge"),
                    source_account,
                    details: OperationDetails::AccountMerge {
                        destination: Self::muxed_account_to_string(&dest)?,
                    },
                })
            }

            OP_ALLOW_TRUST => {
                let trustor = Self::account_id_to_string(xr)?;
                // asset code union
                let t = xr.read_i32()?;
                let asset_code: String = match t {
                    ASSET_TYPE_CREDIT_ALPHANUM4 => {
                        let code_bytes = xr.read_fixed_opaque(4)?;
                        let raw = core::str::from_utf8(code_bytes).unwrap_or("invalid");
                        let trimmed = raw.trim_end_matches('\0');
                        String::try_from(trimmed)
                            .map_err(|_| StellarTransactionError::StringTooLong)?
                    }
                    ASSET_TYPE_CREDIT_ALPHANUM12 => {
                        let code_bytes = xr.read_fixed_opaque(12)?;
                        let raw = core::str::from_utf8(code_bytes).unwrap_or("invalid");
                        let trimmed = raw.trim_end_matches('\0');
                        String::try_from(trimmed)
                            .map_err(|_| StellarTransactionError::StringTooLong)?
                    }
                    _ => return Err(StellarTransactionError::UnsupportedOperation),
                };

                let authorize = xr.read_u32()?;

                Ok(Operation {
                    operation_type: String::try_from("allow_trust")
                        .map_err(|_| StellarTransactionError::StringTooLong)?,
                    source_account,
                    details: OperationDetails::AllowTrust {
                        trustor,
                        asset_code,
                        authorize,
                    },
                })
            }



            OP_SET_TRUST_LINE_FLAGS => {
                let trustor = Self::account_id_to_string(xr)?;
                let asset = Self::parse_asset(xr)?;
                let clear_flags = xr.read_u32()?;
                let set_flags = xr.read_u32()?;
                Ok(Operation {
                    operation_type: String::from("set_trust_line_flags"),
                    source_account,
                    details: OperationDetails::SetTrustLineFlags {
                        trustor,
                        asset,
                        clear_flags,
                        set_flags,
                    },
                })
            }

            _ => Err(StellarTransactionError::UnsupportedOperation),
        }
    }

    fn parse_asset_path(xr: &mut Xdr) -> Result<Vec<Asset>, StellarTransactionError> {
        let count = xr.read_u32()? as usize;
        if count > MAX_PATH_ASSETS {
            return Err(StellarTransactionError::PayloadTooLarge);
        }
        let mut v = Vec::new();
        for _ in 0..count {
            v.push(Self::parse_asset(xr)?);
        }
        Ok(v)
    }

    fn parse_muxed_account(xr: &mut Xdr) -> Result<MuxedAccount, StellarTransactionError> {
        let t = xr.read_i32()?;
        match t {
            KEY_TYPE_ED25519 => {
                let key = xr.read_fixed_opaque(32)?;
                Ok(MuxedAccount::Ed25519 {
                    account_id: Self::u8_array_to_stellar_address(key)?,
                })
            }
            KEY_TYPE_MUXED_ED25519 => {
                let id = xr.read_u64()?;
                let key = xr.read_fixed_opaque(32)?;
                Ok(MuxedAccount::MuxedEd25519 {
                    id,
                    account_id: Self::u8_array_to_stellar_address(key)?,
                })
            }
            _ => Err(StellarTransactionError::MuxedAccountError),
        }
    }

    fn parse_asset(xr: &mut Xdr) -> Result<Asset, StellarTransactionError> {
        let t = xr.read_i32()?;
        match t {
            ASSET_TYPE_NATIVE => Ok(Asset::Native),
            ASSET_TYPE_CREDIT_ALPHANUM4 => {
                let code_bytes = xr.read_fixed_opaque(4)?;
                let code_str = core::str::from_utf8(code_bytes).unwrap_or("invalid");
                let trimmed = code_str.trim_end_matches('\0');
                let issuer = Self::account_id_to_string(xr)?;
                Ok(Asset::CreditAlphanum4 {
                    code: String::from(trimmed),
                    issuer,
                })
            }
            ASSET_TYPE_CREDIT_ALPHANUM12 => {
                let code_bytes = xr.read_fixed_opaque(12)?;
                let code_str = core::str::from_utf8(code_bytes).unwrap_or("invalid");
                let trimmed = code_str.trim_end_matches('\0');
                let issuer = Self::account_id_to_string(xr)?;
                Ok(Asset::CreditAlphanum12 {
                    code: String::from(trimmed),
                    issuer,
                })
            }
            _ => Err(StellarTransactionError::UnsupportedOperation),
        }
    }

    fn parse_change_trust_asset(xr: &mut Xdr) -> Result<ChangeTrustAsset, StellarTransactionError> {
        let t = xr.read_i32()?;
        match t {
            ASSET_TYPE_NATIVE => Ok(ChangeTrustAsset::Native),
            ASSET_TYPE_CREDIT_ALPHANUM4 => {
                let code_bytes = xr.read_fixed_opaque(4)?;
                let code_str = core::str::from_utf8(code_bytes).unwrap_or("invalid");
                let trimmed = code_str.trim_end_matches('\0');
                let issuer = Self::account_id_to_string(xr)?;
                Ok(ChangeTrustAsset::CreditAlphanum4 {
                    code: String::from(trimmed),
                    issuer,
                })
            }
            ASSET_TYPE_CREDIT_ALPHANUM12 => {
                let code_bytes = xr.read_fixed_opaque(12)?;
                let code_str = core::str::from_utf8(code_bytes).unwrap_or("invalid");
                let trimmed = code_str.trim_end_matches('\0');
                let issuer = Self::account_id_to_string(xr)?;
                Ok(ChangeTrustAsset::CreditAlphanum12 {
                    code: String::from(trimmed),
                    issuer,
                })
            }
            ASSET_TYPE_POOL_SHARE => {
                // LiquidityPoolParameters (only constant product now)
                // enum LiquidityPoolType { LIQUIDITY_POOL_CONSTANT_PRODUCT = 0; }
                let lp_type = xr.read_i32()?; // expect 0
                if lp_type != 0 {
                    return Err(StellarTransactionError::UnsupportedOperation);
                }
                // LiquidityPoolConstantProductParameters
                let asset_a = Self::parse_asset(xr)?;
                let asset_b = Self::parse_asset(xr)?;
                let fee = xr.read_i32()?;
                Ok(ChangeTrustAsset::LiquidityPool { asset_a, asset_b, fee })
            }
            _ => Err(StellarTransactionError::UnsupportedOperation),
        }
    }

    fn parse_signatures(xr: &mut Xdr) -> Result<Vec<Signature>, StellarTransactionError> {
        let count = xr.read_u32()? as usize;
        if count > MAX_SIGNATURES {
            return Err(StellarTransactionError::TooManySignatures);
        }

        let mut sigs = Vec::new();
        for _ in 0..count {
            let hint_bytes = xr.read_fixed_opaque(4)?;
            let mut hint = [0u8; 4];
            hint.copy_from_slice(hint_bytes);

            let sig_bytes = xr.read_opaque(64)?;
            let mut signature = [0u8; 64];
            for (i, b) in sig_bytes.iter().enumerate() {
                if i < 64 {
                    signature[i] = *b;
                }
            }

            let hint = bytes_to_hex(&hint);
            let signature = bytes_to_hex(&signature);

            sigs.push(Signature { hint, signature });
        }

        Ok(sigs)
    }

    fn account_id_to_string(
        xr: &mut Xdr,
    ) -> Result<String, StellarTransactionError> {
        let t = xr.read_i32()?;
        if t != KEY_TYPE_ED25519 {
            return Err(StellarTransactionError::AccountIdError);
        }
        let key = xr.read_fixed_opaque(32)?;
        Self::u8_array_to_stellar_address(key)
    }

    fn muxed_account_to_string(muxed_account: &MuxedAccount) -> Result<String, StellarTransactionError> {
        match muxed_account {
            MuxedAccount::Ed25519 { account_id } => {
                String::try_from(account_id.as_str()).map_err(|_| StellarTransactionError::StringTooLong)
            }
            MuxedAccount::MuxedEd25519 { account_id, .. } => {
                String::try_from(account_id.as_str()).map_err(|_| StellarTransactionError::StringTooLong)
            }
        }
    }

    /// Convert stroops to XLM string representation with bounded string
    fn stroops_to_xlm_bounded(stroops: i64) -> Result<String, StellarTransactionError> {
        // Simple conversion avoiding floating point in no-std
        let whole_xlm = stroops / 10_000_000;
        let fractional = stroops % 10_000_000;
        
        if fractional == 0 {
            Self::i64_to_string(whole_xlm)
        } else {
            // For simplicity, just return the stroops value as string in no-std
            Self::i64_to_string(stroops)
        }
    }

    fn u64_to_string(value: u64) -> Result<String, StellarTransactionError> {
        let mut buffer = [0u8; 20]; // u64 max is 20 digits
        let mut i = buffer.len();
        let mut n = value;
        
        if n == 0 {
            return String::try_from("0").map_err(|_| StellarTransactionError::StringTooLong);
        }
        
        while n > 0 {
            i -= 1;
            buffer[i] = b'0' + (n % 10) as u8;
            n /= 10;
        }
        
        let str_slice = core::str::from_utf8(&buffer[i..]).map_err(|_| StellarTransactionError::StringTooLong)?;
        String::try_from(str_slice).map_err(|_| StellarTransactionError::StringTooLong)
    }

    fn i64_to_string(value: i64) -> Result<String, StellarTransactionError> {
        if value >= 0 {
            Self::u64_to_string(value as u64)
        } else {
            let pos_str = Self::u64_to_string((-value) as u64)?;
            let mut result = String::new();
            result.push('-');
            result.push_str(&pos_str);
            Ok(result)
        }
    }

    pub fn stroops_to_xlm_string(stroops: i64) -> String {
        let whole_xlm = stroops / 10_000_000;
        let fractional = (stroops % 10_000_000).abs();

        log_info!("Converting stroops: {}, whole: {}, fractional: {}", stroops, whole_xlm, fractional);
        
        if fractional == 0 {
            format!("{}", whole_xlm)
        } else {
            // Pad to 7 digits, then trim trailing zeros
            let frac_str = format!("{:07}", fractional);
            let trimmed = frac_str.trim_end_matches('0');
            format!("{}.{}", whole_xlm, trimmed)
        }
    }
}

impl Default for StellarTransactionParser {
    fn default() -> Self {
        Self::new()
    }
}