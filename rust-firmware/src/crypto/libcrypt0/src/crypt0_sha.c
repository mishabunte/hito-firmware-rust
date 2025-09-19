#include "crypt0_sha.h"
#include "crypt0.h"

#include <sha3.h>
#include <string.h>

#ifdef __ZEPHYR__
#include <ocrypto_sha256.h>
#include <ocrypto_sha512.h>
#else 
#include <openssl/sha.h>
#endif//__ZEPHYR__

bool crypt0_sha3_keccak2(
		const uint8_t * data1, size_t data1len, 
		const uint8_t * data2, size_t data2len, 
		uint8_t * out, size_t outlen)
{
  sha3_return_t err;
  sha3_context c;

  err = sha3_Init(&c, 256);
  if( err != SHA3_RETURN_OK )
      return false;
  if( sha3_SetFlags(&c, SHA3_FLAGS_KECCAK) != SHA3_FLAGS_KECCAK ) {
      return false;
  }
  sha3_Update(&c, data1, data1len);
  sha3_Update(&c, data2, data2len);
  const void *h = sha3_Finalize(&c);

  memcpy(out, h, outlen);
	return true;
}

bool crypt0_sha3_keccak(const uint8_t * data, size_t datalen, 
		uint8_t * out, size_t outlen)
{
	sha3_HashBuffer(256, SHA3_FLAGS_KECCAK, data, datalen, out, outlen);
	return true;
}

bool crypt0_sha256(const void * data, size_t datalen,
		uint8_t * out, size_t outlen)
{
#ifdef __ZEPHYR__  
	ocrypto_sha256(out, data, datalen);
  return true;
#else
	SHA256(data, datalen, out);
  return true;
#endif
  return true;
}

bool crypt0_sha512(const void * data, uint32_t datalen,
		uint8_t * out, uint32_t outlen)
{
  if (outlen != 64) {
    return false;
  }
#ifdef __ZEPHYR__  
	ocrypto_sha512(out, data, datalen);
  return true;
#else
	SHA512(data, datalen, out);
  return true;
#endif
  return true;
}

// eof
