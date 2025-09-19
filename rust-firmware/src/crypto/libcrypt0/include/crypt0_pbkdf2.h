#ifndef __crypt0_pbkdf2_h_included__
#define __crypt0_pbkdf2_h_included__

#include <stdint.h>

int crypt0_pbkdf2_hmac_sha256(int i, const uint8_t * pass, uint16_t pass_len, 
  const uint8_t * salt, uint16_t salt_len, uint8_t * key, uint16_t key_len);

int crypt0_pbkdf2_hmac_sha512(int i, const uint8_t * pass, uint16_t pass_len, 
  const uint8_t * salt, uint16_t salt_len, uint8_t * key, uint16_t key_len);

#endif//__crypt0_pbkdf2_h_included__
