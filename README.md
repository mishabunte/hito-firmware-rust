# Hito Firmware

Cross-platform rust firmware for hito hardware wallet, designed for both desktop simulation and embedded environments 

---

## Project Structure

- `simulator/` – Desktop simulator application (initially targeting macOS, with planned Linux support)
- `drivers/` – Hardware drivers for Zephyr RTOS and simulator environment
  - `zephyr/` – C implementations for hardware targets
  - `simulator/` – Rust-based mock implementations for desktop simulation
- `lib/` – Cryptographic libraries
  - `libcrypt0/` – Minimalistic crypto primitives (SHA, RSA, Ed25519, secp256k1)
  - `libcrypt0_pro/` – High-level crypto operations for various blockchains (BTC, Ethereum, Solana, NEAR, TON)
- `src/` – Core firmware written in Rust, including UI components and hardware abstractions
- `resources/` – Assets (fonts, images, and configuration files)
- `tests/` - Desktop functionality tests and helping scripts

---

## Getting Started

### Prerequisites

- Rust toolchain (https://rustup.rs)
- Cargo (included with Rust)
- Zephyr RTOS SDK (optional for hardware integration)
- Optional: pyqt6 Python library for Hito Communication Emulator
 

### Setup

```bash
git clone https://github.com/yourusername/hito-firmware-rust.git
cd hito-firmware-rust
cargo build
```

### Running the Simulator

```bash
cargo run --bin hito-simulator
```

---

## Components

### UI
Reusable, modular Rust-based UI components optimized for clarity and ease of use.

### Drivers
Hardware driver abstractions allow seamless switching between simulated and actual hardware environments.

### Cryptography
`libcrypt0` and `libcrypt0_pro` provide essential cryptographic functionalities, designed for performance and security.

## Hito Communication Emulator
- Under `tests/` folder there is a `communication_emulator.py` script that launches __"Hito Communication Emulator"__ desktop app
### Prerequisites:
- Python 3.9 (or higher)
- PyQt6 Python library
### Setup
- After cloning the repo:
```bash
cd rust-firmware/tests/
pip install PyQt6
python communication_emulator.py
```


---

## Roadmap

- [x] Initial simulator for macOS
- [x] Integration with Zephyr RTOS 
- [x] - Display driver support
- [x] - Touch driver support
- [x] Linux simulator support
- [ ] Expanded blockchain support
- [ ] Enhanced UI and UX

---

## Contributing

We welcome contributions! Please fork the repository and open a pull request.

---

## License

This project is licensed under the MIT License - see the [LICENSE.md](LICENSE.md) file for details.


