#ifndef __crypt0_sha_h_included__
#define __crypt0_sha_h_included__

#include <stddef.h>
#include <stdint.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

bool crypt0_sha3_keccak2(
		const uint8_t * data1, size_t data1len, 
		const uint8_t * data2, size_t data2len, 
		uint8_t * out, size_t outlen);

bool crypt0_sha3_keccak(const uint8_t * buf, size_t buflen, 
		uint8_t * out, size_t outlen);

bool crypt0_sha256(const void * buf, size_t buflen,
		uint8_t * out, size_t outlen);

bool crypt0_sha512(const void * data, uint32_t datalen,
		uint8_t * out, uint32_t outlen);

#ifdef __cplusplus
}
#endif

#endif//__crypt0_sha_h_included__
