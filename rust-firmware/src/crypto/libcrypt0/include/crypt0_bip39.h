#ifndef __crypt0_bip39_h_included__
#define __crypt0_bip39_h_included__

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

#define CRYPT0_BIP39_MNEMONIC_MAXBYTES 215 //(9 * 24)

extern const char * crypt0_bip39_english[];
#define CRYPT0_BIP39_MNEMONIC_ENGLISH_MAXWORDS 2048 //(9 * 24)

/** 
 * Converts entropy to mnemonic english
 * returns mnemonic length on success, negative error code otherwise 
 */
int crypt0_bip39_entropy_to_mnemonic_en(const uint8_t * entropy, 
    uint8_t entropy_len, uint8_t * mnemonic, size_t mnemonic_len);

/** 
 * Converts mnemonic to seed, 
 * returns CRYPT0_OK on success, negative error code otherwise
 */
int crypt0_bip39_mnemonic_to_seed(const uint8_t * mnemonic, 
    uint16_t mnemonic_len, uint8_t * seed, uint16_t seed_len);

/**
 * Convert entropy to seed using english mnemonic
 * returns CRYPT0_OK on success, negative error code otherwise
 */
int crypt0_bip39_entropy_to_seed_en(const uint8_t * entropy, 
    uint16_t entropyLen, uint8_t * seed, uint16_t seedLen);

// convert mnemonic provided as array of words indexes to entropy
bool crypt0_bip39_mnemonic_to_entropy(
    const uint16_t * mnemonic, uint16_t mnemonic_len,
    uint8_t * entropy, uint16_t entropy_len);

int crypt0_ton_mnemonic_to_seed(const uint8_t *mnemonic_str, uint16_t mnemonic_len, uint8_t *seed_out, uint32_t seed_len);
bool crypt0_bip39_ton_mnemonic_to_entropy(
    const uint8_t *mnemonic, uint16_t mnemonic_len, 
    uint8_t *entropy, uint16_t entropy_len);


typedef struct
{
    uint32_t purpose;
    uint32_t coin_type;
    uint32_t account;
    uint32_t change;
    uint32_t address_index;
} bip_path_data;

#endif//__crypt0_bip39_h_included__
