// fromSeedBytes.js
import { Keypair } from '@stellar/stellar-sdk'
import { derivePath } from 'ed25519-hd-key'

const bip39SeedHex = '38b6a363e88b28138cc71f0145ab429c251baa8cd8fa6d80bcfb39c35076f1766e24dfc01ce0e22e8dfec185ad7a67ce748cd6551ad1b738619b8859808bbf88'

// Derive the 32-byte Stellar account seed (path m/44'/148'/0')
const { key } = derivePath("m/44'/148'/0'", bip39SeedHex)

console.log('Derived key:\t', Buffer.from(key).toString('hex'))

const keypair = Keypair.fromRawEd25519Seed(key)

console.log('Keypair generated from seed:', keypair)

console.log('Public address:\t', keypair.publicKey())
console.log('Expected address:\tGCEMPQ7LYZ7EYWFYQYXIFQRUHKLRT74TLORALIOZIP2KPED7TD3GG352')
console.log('Secret seed:', keypair.secret())