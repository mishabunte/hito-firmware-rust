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

---

## Getting Started

### Prerequisites

- Rust toolchain (https://rustup.rs)
- Cargo (included with Rust)
- Zephyr RTOS SDK (optional for hardware integration)

### Setup

```bash
git clone https://github.com/yourusername/hito-firmware.git
cd hito-firmware
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

---

## Roadmap

- [x] Initial simulator for macOS
- [ ] Integration with Zephyr RTOS 
- [ ] - Display driver support
- [ ] - Touch driver support
- [ ] Linux simulator support
- [ ] Expanded blockchain support
- [ ] Enhanced UI and UX

---

## Contributing

We welcome contributions! Please fork the repository and open a pull request.

---

## License

This project is licensed under the MIT License - see the [LICENSE.md](LICENSE.md) file for details.


