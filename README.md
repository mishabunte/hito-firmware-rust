
# Hito Firmware (Rust Edition)

This repository contains the Rust-based firmware for the **Hito Hardware Wallet**, including:

- A **device simulator**, devboard configuration, and production device firmware.
- A **Stellar transaction signing library** written in `#![no_std]`.
- A set of **crypto primitive implementations** used internally by the firmware (Ed25519, hashing, base32/base64, CRC, etc.).
- UI implementation using **Slint** and a custom display driver.

This branch (`feature/stellar`) focuses on integrating a full **Stellar signing flow**, including parsing, displaying, and signing XDR transactions inside the firmware.


---

## 1. Architecture Overview

### Components
```

┌────────────────────────┐
│  Rust Firmware (no_std)│
│  - Stellar signing     │
│  - Crypto primitives   │
│  - Slint UI            │
│  - BLE/NFC message I/O │
└───────────┬────────────┘
│
▼
┌────────────────────────┐
│ Device Runtime         │
│ - DevBoard (nRF5340 DK)│
│ - Production PCB       │
└───────────┬────────────┘
│
▼
┌────────────────────────┐
│ Simulator (minifb)     │
│ - Desktop UI           │
│ - Socket API           │
└────────────────────────┘

```

### Workflow

#### **Simulator**
- Runs on desktop using **minifb** (320×240 framebuffer).
- Receives messages through a Unix socket:  
  `"/tmp/hito_Linux.sock"`.
- Used for rapid UI testing and verifying transaction flows.
- Implements the same logic as the real device.

#### **DevBoard / nRF5340 DK**
- Used for hardware bring-up and debugging.
- Firmware compiled with Zephyr support (RTOS, drivers, BLE).

#### **Production Device**
- Same logic as devboard, but with custom:
  - display driver  
  - input controller  
  - power management  
  - secure memory layout  

The simulator, devboard, and real device all run *identical Stellar and crypto code*.  
Only the platform layer differs.


---

## 2. Stellar Transaction Signing Library

Located in:

```

rust-firmware/src/crypto/libcrypt0pro/stellar

````

This library implements a **no_std Stellar parser + signing engine** compatible with Stellar’s XDR format.

### Key Modules

| Module | Description |
|--------|-------------|
| `transaction_parser.rs` | Parses base64 XDR into internal `ParsedTransaction` struct. Supports `NETWORK_HASH:XDR_BYTES` format + Minimal XDR definitions required for payments, memos, source accounts, operations. |
| `transaction_serializer.rs` | Produces Stellar signatures (via Ed25519) and constructs the decorated signature. |
| `address.rs` | Produces Stellar keypair from seed. |

### Core Parsers

```rust
impl StellarTransactionParser {
  pub fn parse_transaction(tx_data: &str) -> Result<ParsedTransaction, TransactionParseError> {
}

impl StellarTransactionSigner {
    pub fn build_signature_base(
        parsed_tx: &ParsedTransaction,
    ) -> Result<Vec<u8>, TransactionParseError>;
}
````

### Signing Flow

```
1. Host sends: "stellar.sign:NETWORK_HASH:XDR_BASE64"
2. Device parses XDR → `ParsedTransaction`
3. UI displays 1–2-3 page confirmation view
4. User approves
5. Device computes hash = SHA256(network_hash || tx_body)
6. Sign with Ed25519 private key stored in secure memory
7. Display base64 encoded signed transaction
```

The parser handles `LimitedString`, operations, memos, and public key encoding into `G...` addresses.

---

## 3. Cryptographic Primitives Library

Located in:

```
rust-firmware/src/crypto/crypt0/
```

This is a minimal, dependency-free `#![no_std]` crypto library specifically for hardware wallet use written in C and Rust

### Includes:

* **Ed25519**

  * Key generation
  * Deterministic signing
  * Signature verification
* **SHA-256**
* **HMAC-SHA-256**
* **CRC16-XModem** (Stellar address checksum)
* **Base32 (RFC4648)** — for Stellar address encoding
* **Base64 (custom no_std implementation)**
* **Hex encoder/decoder**

### API Summary

#### Ed25519

```rust
pub fn crypt0_ed25519_sign(
  message: *const u8,
  messagelen: usize,
  privkey: *const u8,
  privlen: usize,
  pubkey: *const u8,
  publen: usize,
  sig: *mut u8,
  siglen: usize
) -> c_int;
```

#### SHA256

```rust
pub fn crypt0_sha256(
  data: *const u8,
  data_len: usize,
  out: *mut u8,
  outlen: usize
) -> bool;
```

#### Base64 (no_std)

```rust
fn decode_base64(base64_data: &str) -> Result<Vec<u8>, TransactionParseError>
```

#### CRC16 (Stellar)

```rust
pub fn crypt0_crc16_ccitt(data: *const u8, len: usize) -> u16;
```

#### Address Encoding

```rust
pub fn encode_stellar_address(public_key: &[u8; 32]) -> Result<String, StellarError>;
```

These primitives are optimized for **constant-time execution**, no heap allocation (using optional heapless), and deterministic behavior for embedded systems.

---

## 4. UI Flow (Slint + Firmware Logic)

The Stellar signing flow renders as:

1. **Page 1**

   * Source account (shortened)
   * Destination address
   * Amount (`XLM` or asset)
   * Network (Testnet/Futurenet)

2. **Page 2**

   * Operation details (Payment, AccountMerge, etc.)
   * Fee
   * Memo button

2. **Page 3**
   * Memo overview

3. **Buttons**

   * SIGN → signs transaction
   * BACK → goes back 

The same UI definitions work in simulator and hardware.

---

## 5. Building

### Simulator

```
cargo run --bin hito-firmware-rust
```

### Running simulator + host test client

Send a test Stellar signing request on Testnet:

```bash
echo "stellar.sign:cee0302d59844d32bdca915c8203dd44b33fbb7edc19051ea37abedf28ecd472:BASE64XDR" \
  | nc -U /tmp/hito_Linux.sock
```

---

## 6. Future Work

* Android/iOS integration (Unstoppable Wallet fork)
* Soroban support
* 100 hardware devices assembled and ready-to-use for Stellar network

---

## 7. License

This project is licensed under the MIT License - see the [LICENSE.md](LICENSE.md) file for details.
