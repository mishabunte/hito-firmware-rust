#ifndef __crypt0_bip32_h_included__
#define __crypt0_bip32_h_included__

#define CRYPT0_BIP32_INDEX_HARDENED 0x80000000
#define ACCOUNT_PKEY_LEN 78

#include <stdint.h>

/**
 * Converts seed to secret - chaincode (32 bytes) + private key (32 bytes)
 * returns secret length on success, negative error code otherwise 
 */
int crypt0_bip32_seed_to_secret(const uint8_t * seed, uint8_t * secret);

/**
 * Converts seed to secret - chaincode (32 bytes) + private key (32 bytes)
 * returns secret length on success, negative error code otherwise 
 */
int crypt0_bip32_seed_to_secret_custom(const uint8_t * seed, uint8_t * secret,
    const char * key);

/**
 * Derives chaincode and private key by index, modifies secret
 * returns secret length on success, negative error code otherwise
 */
int crypt0_bip32_derive_secret_index(uint8_t * secret, uint32_t index);

/**
 * Derives chaincode and private key by index, modifies secret
 * returns secret length on success, negative error code otherwise
 * NEAR protocol version derivation
 */
int crypt0_bip32_derive_secret_index_near(uint8_t * secret, uint32_t index);

/**
 * Derives chaincode and private key by index, modifies secret
 * returns secret length on success, negative error code otherwise
 */
int crypt0_bip32_derive_secret_path(uint8_t * secret, const uint8_t * path);

int crypt0_bip32_seed_to_account_pubkey(const uint8_t * m_seed, uint8_t * pubkey, int bip_standard, int coin, int account);

#endif//__crypt0_bip32_h_included__
