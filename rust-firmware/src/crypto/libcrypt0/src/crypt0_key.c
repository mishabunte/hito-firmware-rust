#ifndef __crypt0_key_h_included__
#define __crypt0_key_h_included__

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>

// Returns device unique entropy
bool crypt0_hw_get_unique_entropy(uint8_t * entropy, size_t entropy_len);

#endif//__crypt0_key_h_included__
