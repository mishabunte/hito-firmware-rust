/**
 * RLP encoding and decoding primitives
 * https://ethereum.org/en/developers/docs/data-structures-and-encoding/rlp/
 *
 * Author: Mikhail Bunte, m@bunte.art
 */
#ifndef __crypt0_rlp_h_included__
#define __crypt0_rlp_h_included__

#include <stddef.h>
#include <stdint.h>
#include <stdbool.h>
#include <stdarg.h>
#include "intc.h"

/** 
 * RLP encode list, returns length of RLP list encoded
 */
int crypt0_rlp_encode_list(uint8_t * out, size_t outlen, const char * fmt, ...);

typedef int (*crypt0_rlp_decode_handler_t)(const uint8_t * value, size_t len, size_t index);

// returns payload length on success
int crypt0_rlp_decode_list(const uint8_t * buf, size_t buflen, 
  size_t items_count, crypt0_rlp_decode_handler_t handler);

/** 
 * returns length of RLP encoded list
 */
int crypt0_rlp_encode_list_len(const char * fmt, ...);

int crypt0_rlp_encode_list_len_v(const char * fmt, va_list ap);


/**
 * RLP encode value and return encoded length of it
 */
int crypt0_rlp_encode_val(const uint8_t * buf, size_t buflen, 
		uint8_t * out, size_t outlen);

//
int crypt0_rlp_encode_val_uint(uint64_t val, uint8_t * out, size_t outlen);
int crypt0_rlp_encode_val_uint_len(uint64_t val);

int crypt0_rlp_encode_val_uint256(intc_u256 *val, uint8_t * out, size_t outlen);
int crypt0_rlp_encode_val_uint256_len(intc_u256 *val);

/**
 * Returns length of rlp-encoded value
 */
int crypt0_rlp_encode_val_len(const uint8_t * buf, size_t buflen);

#endif//__crypt0_rlp_h_included__
