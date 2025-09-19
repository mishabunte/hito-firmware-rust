#ifndef __crypt0_ripemd160_h_included__
#define __crypt0_ripemd160_h_included__

#include <stdint.h>

#define CRYPT0_RIPEMD_BYTES 20

#ifdef __cplusplus
extern "C" {
#endif

void ripemd160(const uint8_t* msg, uint32_t msg_len, uint8_t* hash);
#define crypt0_ripemd160 ripemd160

#ifdef __cplusplus
}
#endif
#endif//__crypt0_ripemd160_h_included__
