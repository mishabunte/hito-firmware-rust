#include "crypt0_pbkdf2.h"

#include "crypt0.h"
#include "crypt0_byteorder.h"
#include "crypt0_hmac.h"

#include <string.h>

int crypt0_pbkdf2_hmac_sha256(int i, const uint8_t * pass, uint16_t pass_len, 
  const uint8_t * salt, uint16_t salt_len, uint8_t * key, uint16_t key_len)
{
	crypt0_init();

  uint8_t salt_tmp[68];
  uint8_t digest[32];

  if (salt_len > 64) {
    return CRYPT0_ERR_SALTLEN;
  }

	if (key_len != 32) {
		return CRYPT0_ERR_PRIVKEY_LEN;
	}

	memcpy(salt_tmp, salt, salt_len);
	uint32_t index_le = 1;
	index_le = crypt0_cpu_to_be32(index_le);
	memcpy(&salt_tmp[salt_len], &index_le, 4);
	salt_len += 4;

	for(int n = 0; n < i; n++) {
		crypt0_hmac_sha256(pass, pass_len, salt_tmp, salt_len, digest);

		salt_len = 32;
		if (n == 0) {
		  memcpy(key, digest, 32);
		  memcpy(salt_tmp, digest, 32);
		} else {
			for(int j = 0; j < 32; j++) {
				salt_tmp[j] = digest[j];
				key[j] = key[j] ^ digest[j];
			}
		}
	}
  return CRYPT0_OK;
}

int crypt0_pbkdf2_hmac_sha512(int i, const uint8_t * pass, uint16_t pass_len, 
  const uint8_t * salt, uint16_t salt_len, uint8_t * key, uint16_t key_len)
{
	crypt0_init();
  uint8_t salt_tmp[68];
  uint8_t digest[64];

  if (salt_len > 64) {
    return CRYPT0_ERR_SALTLEN;
  }

	if (key_len != 64) {
		return CRYPT0_ERR_PRIVKEY_LEN;
	}

	memcpy(salt_tmp, salt, salt_len);
	uint32_t index_le = 1;
	index_le = crypt0_cpu_to_be32(index_le);
	memcpy(&salt_tmp[salt_len], &index_le, 4);
	salt_len += 4;

	for(int n = 0; n < i; n++) {
		int res = crypt0_hmac_sha512(pass, pass_len, salt_tmp, salt_len, digest);
		if (res != CRYPT0_OK) {
			return res;
		}

		salt_len = 64;
		if (n == 0) {
		  memcpy(key, digest, 64);
		  memcpy(salt_tmp, digest, 64);
		} else {
			for(int j = 0; j < 64; j++) {
				salt_tmp[j] = digest[j];
				key[j] = key[j] ^ digest[j];
			}
		}
	}
  return CRYPT0_OK;
}

