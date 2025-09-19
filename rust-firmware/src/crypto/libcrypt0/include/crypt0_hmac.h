#ifndef __crypt0_hmac_h_included__
#define __crypt0_hmac_h_included__

#include <stdint.h>

/**
 * Calculate HMAC SHA512 
 * returns CRYPT0_OK on success
 */
int crypt0_hmac_sha512(const uint8_t * key, uint16_t key_len, 
  const uint8_t * msg, uint16_t msg_len, uint8_t * digest);

int crypt0_hmac_sha256(const uint8_t * key, uint16_t key_len, 
  const uint8_t * msg, uint16_t msg_len, uint8_t * digest);

#endif//__crypt0_hmac_h_included__
