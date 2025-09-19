#include "crypt0.h"

#include "crypt0_hmac.h"
#include "crypt0_secp256k1.h"
#include "crypt0_bip32.h"
#include "crypt0_sha.h"
#include "crypt0_ripemd160.h"

#ifdef __ZEPHYR__
#include <ocrypto_curve_p256.h>
#include <ocrypto_ecdsa_p256.h>
#endif

#include <crypt0_byteorder.h>
#include <crypt0_log.h>

#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Wunused-variable"
LOG_MODULE_REGISTER(crypt0_bip32, LOG_LEVEL_DBG);
#pragma GCC diagnostic pop

#define COIN_ID_BTC_TEST 1
#define COIN_ID_LITECOIN 2
#define COIN_ID_DOGECOIN 3
#define COIN_ID_NEAR 397

//-----------------------------------------------------------------------------
int crypt0_bip32_seed_to_account_pubkey(const uint8_t * seed, uint8_t * pubkey, int bip_standard, int coin, int account)
{
  if (bip_standard - CRYPT0_BIP32_INDEX_HARDENED == 84 || bip_standard == 84) {
    if (coin == COIN_ID_BTC_TEST || coin - CRYPT0_BIP32_INDEX_HARDENED == COIN_ID_BTC_TEST) {
      // vpub 045f1cf6
      pubkey[0] = 0x04;
      pubkey[1] = 0x5F;
      pubkey[2] = 0x1C;
      pubkey[3] = 0xF6;
    } else { // xpub if (coin == 0 || coin - CRYPT0_BIP32_INDEX_HARDENED == 0) {
      pubkey[0] = 0x04;
      pubkey[1] = 0xB2;
      pubkey[2] = 0x47;
      pubkey[3] = 0x46;
    }
  }

  else { //bip44-ish if (bip_standard - CRYPT0_BIP32_INDEX_HARDENED == 44 || bip_standard == 44) {
    if (coin == COIN_ID_BTC_TEST || coin - CRYPT0_BIP32_INDEX_HARDENED == COIN_ID_BTC_TEST) {
      // tpub
      pubkey[0] = 0x04;
      pubkey[1] = 0x35;
      pubkey[2] = 0x87;
      pubkey[3] = 0xCF;
    } else if (coin == COIN_ID_LITECOIN || coin - CRYPT0_BIP32_INDEX_HARDENED == COIN_ID_LITECOIN) {
      // Ltub 019da462
      pubkey[0] = 0x01;
      pubkey[1] = 0x9D;
      pubkey[2] = 0xA4;
      pubkey[3] = 0x62;
    } else if (coin == COIN_ID_DOGECOIN || coin - CRYPT0_BIP32_INDEX_HARDENED == COIN_ID_DOGECOIN) {
      // dgub 02facafd
      pubkey[0] = 0x02;
      pubkey[1] = 0xFA;
      pubkey[2] = 0xCA;
      pubkey[3] = 0xFD;
    } else { //if (coin == 0 || coin - CRYPT0_BIP32_INDEX_HARDENED == 0) {
      // xpub
      pubkey[0] = 0x04;
      pubkey[1] = 0x88;
      pubkey[2] = 0xB2;
      pubkey[3] = 0x1E;
    }
  }

  pubkey[4] = 0x03;

  uint8_t secret[CRYPT0_BIP32_SECRET_BYTES];
  if (coin == COIN_ID_NEAR || (coin - CRYPT0_BIP32_INDEX_HARDENED == COIN_ID_NEAR)) {
    crypt0_bip32_seed_to_secret_custom(seed, secret, "ed25519 seed");
    crypt0_bip32_derive_secret_index_near(secret, bip_standard);
    crypt0_bip32_derive_secret_index_near(secret, coin);
  } else {
    crypt0_bip32_seed_to_secret(seed, secret);
    crypt0_bip32_derive_secret_index(secret, bip_standard);
    crypt0_bip32_derive_secret_index(secret, coin);
  }

  uint8_t pub[33];
  crypt0_secp256k1_public_key_compressed(secret, 32, pub, 33);

  // get sha256 + ripemd160 of public key
  uint8_t sha[32];
  crypt0_sha256(pub, 33, sha, 32);
  crypt0_ripemd160(sha, 32, pub);

  // copy first four bytes of a ripemd160
  pubkey[5] = pub[0];
  pubkey[6] = pub[1];
  pubkey[7] = pub[2];
  pubkey[8] = pub[3];

  // LOG_HEXDUMP_DBG(pub, 4, "parent fingerprint");

  if (crypt0_bip32_derive_secret_index(secret, account) != CRYPT0_OK)
    {
      LOG_DBG("Child secret generation failed");
      return -1;
    }

  pubkey[9] = (account >> 24) & 0xFF;
  pubkey[10] = (account >> 16) & 0xFF;
  pubkey[11] = (account >> 8) & 0xFF;
  pubkey[12] = account & 0xFF;

  memcpy(&pubkey[13], &secret[32], 32);

  uint8_t pubkey_compressed[33];
  int ret = crypt0_secp256k1_public_key_compressed(secret, 32, pubkey_compressed, 33);
  if (ret < 0)
  {
    LOG_ERR("pubkey compresser returned error");
    return -1;
  }
  memcpy(&pubkey[45], pubkey_compressed, 33);

  return 79;
}


/**
 * Converts seed to secret - chaincode (32 bytes) + private key (32 bytes)
 * returns secret length on success, negative error code otherwise 
 */
int crypt0_bip32_seed_to_secret(const uint8_t * seed, uint8_t * secret)
{
  return crypt0_hmac_sha512((const uint8_t *)"Bitcoin seed", sizeof("Bitcoin seed") - 1, 
    seed, CRYPT0_BIP32_SEED_BYTES, secret);
}

int crypt0_bip32_seed_to_secret_custom(const uint8_t * seed, uint8_t * secret,
    const char * key)
{
  return crypt0_hmac_sha512((const uint8_t *)key, strlen(key), 
    seed, CRYPT0_BIP32_SEED_BYTES, secret);
}

//-----------------------------------------------------------------------------
/**
 * Derives chaincode and private key by index, modifies secret
 * returns CRYPT0_OK on success, negative error code otherwise
 * secret is privkey (32 bytes) and chaincode (32 bytes)
 */
int crypt0_bip32_derive_secret_index(uint8_t * secret, uint32_t index)
{
  uint8_t payload[64];  
  uint8_t hmac_src[37];
  uint32_t index_be;
#ifdef __ZEPHYR__
  ocrypto_cp_p256 r;
  ocrypto_cp_p256 n;
  ocrypto_cp_p256 p;
#else
  uint32_t r[8];
  uint32_t n[8];
  uint32_t p[8];
#endif

  if (index & CRYPT0_BIP32_INDEX_HARDENED) {

    // 0x00, private_key (32 bytes), index (4 bytes big endian)
    hmac_src[0] = 0x00;
    memcpy(&hmac_src[1], secret, 32);
    index_be = crypt0_be32_to_cpu(index);
    memcpy(&hmac_src[33], &index_be, 4);

    // LOG_HEXDUMP_DBG(hmac_src, sizeof(hmac_src), "hmac_src");
    crypt0_hmac_sha512(&secret[32], 32, hmac_src, 37, payload);

  } else {
    index_be = crypt0_be32_to_cpu(index);
    crypt0_secp256k1_public_key_compressed(secret, 32, hmac_src, CRYPT0_SECP256_PUBKEY_COMPRESSED_BYTES);
    memcpy(&hmac_src[33], &index_be, 4);

    crypt0_hmac_sha512(&secret[32], 32, hmac_src, 37, payload);
  }
  
  // LOG_HEXDUMP_DBG(payload, 64, "payload");

#ifdef __ZEPHYR__
  ocrypto_curve_p256_from32bytes(&n, payload);
  ocrypto_curve_p256_from32bytes(&p, secret);

	secp256k1_scalar_add(r.x.w, n.x.w, p.x.w);

  ocrypto_curve_p256_to32bytes(secret, &r);
#else
  secp256k1_scalar_set_b32(n, payload);
  secp256k1_scalar_set_b32(p, secret);
	secp256k1_scalar_add(r, n, p);
  secp256k1_scalar_get_b32(secret, r);
#endif

  memcpy(&secret[32], &payload[32], 32);

  // LOG_HEXDUMP_DBG(secret, 64, "secret");
  
  return CRYPT0_OK;
}

//-----------------------------------------------------------------------------
/**
 * Derives chaincode and private key by index, modifies secret
 * returns CRYPT0_OK on success, negative error code otherwise
 * secret is privkey (32 bytes) and chaincode (32 bytes)
 */
int crypt0_bip32_derive_secret_index_near(uint8_t * secret, uint32_t index)
{
  uint8_t payload[64];  
  uint8_t hmac_src[37];
  uint32_t index_be;

    hmac_src[0] = 0x00;
    memcpy(&hmac_src[1], secret, 32);
    index_be = crypt0_be32_to_cpu(index);
    memcpy(&hmac_src[33], &index_be, 4);

    // LOG_HEXDUMP_DBG(hmac_src, sizeof(hmac_src), "hmac_src");
    crypt0_hmac_sha512(&secret[32], 32, hmac_src, 37, payload);

  memcpy(secret, payload, 64);

    // LOG_HEXDUMP_DBG(secret, 64, "secret");
  
  return CRYPT0_OK;
}

//-----------------------------------------------------------------------------
/**
 * Derives chaincode and private key by index, modifies secret
 * returns secret length on success, negative error code otherwise
 */
int crypt0_bip32_derive_secret_path(uint8_t * secret, const uint8_t * path)
{
  return CRYPT0_ERR_NOT_IMPLEMENTED;
}

