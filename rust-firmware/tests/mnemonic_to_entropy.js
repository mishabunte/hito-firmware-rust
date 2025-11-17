// mnemonic_to_entropy.js
import bip39 from "bip39";

// Get mnemonic from command line args or stdin-like string
const mnemonic = "zero zero zero zero zero zero zero zero zero zero zero zoo";

if (!bip39.validateMnemonic(mnemonic)) {
  console.error("Invalid BIP-39 mnemonic");
  process.exit(1);
}

// entropyHex is the original entropy in hex form
const entropyHex = bip39.mnemonicToEntropy(mnemonic);
const entropyBuffer = Buffer.from(entropyHex, "hex");

console.log("Mnemonic:      ", mnemonic);
console.log("Entropy (hex): ", entropyHex.toString('hex'));
console.log("Entropy (bytes):", entropyBuffer);
console.log("Entropy length:", entropyBuffer.length, "bytes");
