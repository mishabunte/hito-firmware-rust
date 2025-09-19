#ifndef __crypt0_keys_h_included__
#define __crypt0_keys_h_included__

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>

// Derive entropy from hw unique key
bool crypt0_hw_key_derive_entropy(uint8_t * entropy, size_t entropy_len);

#endif//__crypt0_keys_h_included__
