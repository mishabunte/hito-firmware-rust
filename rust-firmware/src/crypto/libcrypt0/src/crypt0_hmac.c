#include "crypt0.h"

#ifdef __ZEPHYR__
#include <ocrypto_hmac_sha512.h>
#include <ocrypto_hmac_sha256.h>
#else 
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <openssl/hmac.h>
#endif

/**
 * Calculate HMAC SHA512 
 * returns CRYPT0_OK on success
 */
int crypt0_hmac_sha512(const uint8_t * key, uint16_t key_len, 
  const uint8_t * msg, uint16_t msg_len, uint8_t * digest)
{
#ifdef __ZEPHYR__
  ocrypto_hmac_sha512(digest, key, key_len, msg, msg_len);    
  return CRYPT0_OK;
#else
  unsigned int digest_len;
  HMAC(EVP_sha512(), key, key_len, msg, msg_len, digest, &digest_len);
  return CRYPT0_OK;
#endif
}

int crypt0_hmac_sha256(const uint8_t * key, uint16_t key_len, 
  const uint8_t * msg, uint16_t msg_len, uint8_t * digest)
{
#ifdef __ZEPHYR__
  ocrypto_hmac_sha256(digest, key, key_len, msg, msg_len);    
  return CRYPT0_OK;
#else
  unsigned int digest_len;
  HMAC(EVP_sha256(), key, key_len, msg, msg_len, digest, &digest_len);
  return CRYPT0_OK;
#endif
}
