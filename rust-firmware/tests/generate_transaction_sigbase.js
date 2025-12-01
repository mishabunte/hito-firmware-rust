// Run: node generate_transaction_sigbase.js
import { Transaction, Networks, hash } from "@stellar/stellar-base";

// Raw base64 transaction
const base64Tx =
  "AAAAAgAAAACdr++ECgMp7XJRAM8An6JDIwr7HfywJyQCDQd2Cn6CLwL68IAACsu/AAAAAgAAAAEAAAAAAAAAAAAAAABpG2bVAAAAAAAAAAEAAAAAAAAAAQAAAACIx8Prxn5MWLiGLoLCNDqXGf+TW6IFodlD9KeQf5j2YwAAAAAAAAAAB7+kgAAAAAAAAAAA";

// 1. Parse the TX using the NETWORK passphrase directly
const tx = new Transaction(base64Tx, Networks.PUBLIC);

// 2. Compute networkId = SHA256(passphrase)
const networkId = hash(Networks.PUBLIC); // returns Buffer(32)

// 3. Compute signature base using official SDK
const sigBase = tx.signatureBase();

// Output
console.log("networkId:", networkId.toString("hex"));
console.log("signatureBase:", sigBase.toString("hex"));
console.log("signatureBase length:", sigBase.length);
