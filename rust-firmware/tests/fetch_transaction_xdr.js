const txHash = "aba2857ec2b09c74c5871c29f5566b9bb795fcf1a9980ab2c0a6d9fbd7f33cb8";
const network = "https://horizon-testnet.stellar.org";

async function getTxXdr(hash) {
  const res = await fetch(`${network}/transactions/${hash}`);
  if (!res.ok) {
    throw new Error(`Horizon error: ${res.status}`);
  }
  const data = await res.json();
  return data.envelope_xdr;
}

getTxXdr(txHash)
  .then(xdr => {
    console.log("Transaction XDR (testnet):", xdr);
  })
  .catch(console.error);
