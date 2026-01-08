import {
  Account,
  Asset,
  Networks,
  Operation,
  TransactionBuilder,
  Memo,
} from "@stellar/stellar-sdk";

// 1) Set network + fee
const networkPassphrase = Networks.PUBLIC; // or Networks.PUBLIC
const fee = "10000000"; // stroops per operation

// 2) You need a source account + correct sequence number
// IMPORTANT: Sequence must be current on-chain sequence for the source account.
// Example placeholder:
const sourcePublicKey = "GCEMPQ7LYZ7EYWFYQYXIFQRUHKLRT74TLORALIOZIP2KPED7TD3GG352";
const sourceSequence = "1234567890122"; // string

const sourceAccount = new Account(sourcePublicKey, sourceSequence);

// 3) Build transaction
const tx = new TransactionBuilder(sourceAccount, {
  fee,
  networkPassphrase,
  // optional:
  // timebounds: { minTime: 0, maxTime: 0 }, // 0/0 means "no timebounds" in v2 builder usage; see note below
})
  .addMemo(Memo.text("aaaaaaaaaaaaaaaaaaaaaaaa"))
  .addOperation(
    Operation.payment({
      destination: "GCO2734EBIBST3LSKEAM6AE7UJBSGCX3DX6LAJZEAIGQO5QKP2BC7NZ4",
      asset: Asset.native(),
      amount: "1.337",
    })
  )
  .setTimeout(180) // adds timebounds (recommended). Use 0 for no timeout (not recommended).
  .build();

// 4) Get XDR (TransactionEnvelope, base64)
const xdrBase64 = tx.toEnvelope().toXDR("base64");
console.log(xdrBase64);
