
#include <crypt0_ed25519.h>
#include <crypt0_hmac.h>

#include <stdint.h>
#include <string.h>

#ifdef __ZEPHYR__
#include <ocrypto_ed25519.h>
#else
#include <openssl/evp.h>
#include <openssl/ec.h>
#include <openssl/core_names.h>
#include <openssl/err.h>
#endif

#define ED25519_CHAIN_CODE_SIZE 32
#define ED25519_PRIVATE_KEY_SIZE 32
#define ED25519_DERIVE_DATA_SIZE (1 + ED25519_PRIVATE_KEY_SIZE + 4) // 0x00 + private_key + index

/** convert secret key to public */
int crypt0_ed25519_public_key(const uint8_t * priv, size_t privlen, 
        uint8_t * pub, size_t publen)
{
  if (publen != CRYPT0_ED25519_PUBKEY_BYTES) {
    return -1;
  }
  if (privlen != CRYPT0_ED25519_PRIVKEY_BYTES) {
    return -1;
  }

  #ifdef __ZEPHYR__
    ocrypto_ed25519_public_key(pub, priv);

    return 0;
  
  #else

    EVP_PKEY *pkey = NULL;
    EVP_PKEY_CTX *ctx = EVP_PKEY_CTX_new_id(EVP_PKEY_ED25519, NULL);

    if (ctx == NULL) {
      fprintf(stderr, "Failed to create EVP_PKEY_CTX\n");
      return -1;
    }

    if (EVP_PKEY_keygen_init(ctx) <= 0) {
      EVP_PKEY_CTX_free(ctx);
      return -1;
    }

    pkey = EVP_PKEY_new_raw_private_key(EVP_PKEY_ED25519, NULL, priv, 32);
    if (pkey == NULL) {
      EVP_PKEY_CTX_free(ctx);
      return -1; 
    }

    size_t publen_ret = publen;
    if (EVP_PKEY_get_raw_public_key(pkey, pub, &publen_ret) <= 0) {
      EVP_PKEY_free(pkey);
      EVP_PKEY_CTX_free(ctx);
      return -1;
    }

    EVP_PKEY_free(pkey);
    EVP_PKEY_CTX_free(ctx);

    return 0;
  
  #endif

}

/** sign sha256 hash */
int crypt0_ed25519_sign(const uint8_t * message, size_t messagelen, 
        const uint8_t * priv, size_t privlen, 
        const uint8_t * pub,  size_t publen, 
        uint8_t * sig, uint8_t siglen)
{
  if (publen != CRYPT0_ED25519_PUBKEY_BYTES) {
    return -1;
  }
  if (privlen != CRYPT0_ED25519_PRIVKEY_BYTES) {
    return -1;
  }
  if (siglen != CRYPT0_ED25519_SIG_BYTES) {
    return -1;
  }
  #ifdef __ZEPHYR__
    ocrypto_ed25519_sign(sig, message, messagelen, priv, pub);
  #else
    EVP_PKEY *pkey = NULL;
    EVP_PKEY_CTX *ctx = EVP_PKEY_CTX_new_id(EVP_PKEY_ED25519, NULL);

    if (EVP_PKEY_keygen_init(ctx) <= 0) {
      return -1;
    }

    if ((pkey = EVP_PKEY_new_raw_private_key(EVP_PKEY_ED25519, NULL, priv, 32)) == NULL) {
      return -1; 
    }

    EVP_MD_CTX *mdctx = EVP_MD_CTX_new();
    if (mdctx == NULL) {
        return -1;
    }

    if (EVP_DigestSignInit(mdctx, NULL, NULL, NULL, pkey) <= 0) {
        return -1;
    }

    size_t siglen_ret = siglen;
    if (EVP_DigestSign(mdctx, sig, &siglen_ret, message, messagelen) <= 0) {
      return -1;
    }

    EVP_MD_CTX_free(mdctx);
    EVP_PKEY_CTX_free(ctx);
  #endif
  return 0;
}

int crypt0_ed25519_derive_secret_index(uint8_t * secret, uint32_t index)
{
  if (index < 0x80000000) {
    return -1;
  }

  uint8_t data[ED25519_DERIVE_DATA_SIZE];
  uint8_t hash[64];

  // Prepare data: 0x00 || private_key || index (big-endian)
  data[0] = 0x00;
  memcpy(data + 1, secret, ED25519_PRIVATE_KEY_SIZE);
  data[ED25519_PRIVATE_KEY_SIZE + 1] = (index >> 24) & 0xFF;
  data[ED25519_PRIVATE_KEY_SIZE + 2] = (index >> 16) & 0xFF;
  data[ED25519_PRIVATE_KEY_SIZE + 3] = (index >> 8) & 0xFF;
  data[ED25519_PRIVATE_KEY_SIZE + 4] = index & 0xFF;

  // HMAC-SHA512(chain_code, data)
  crypt0_hmac_sha512(&secret[32], ED25519_CHAIN_CODE_SIZE, data, sizeof(data), hash);
  memcpy(secret, hash, ED25519_PRIVATE_KEY_SIZE + ED25519_CHAIN_CODE_SIZE);
  return 0;
}