#ifndef __crypt0_secp256k1_h_included__
#define __crypt0_secp256k1_h_included__

/** secp256k1 privmitives */

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

#define CRYPT0_SECP256_PRIVKEY_BYTES 32
#define CRYPT0_SECP256_PUBKEY_BYTES 65
#define CRYPT0_SECP256_PUBKEY_COMPRESSED_BYTES 33
#define CRYPT0_SECP256_SIG_RECOVERABLE_BYTES 65
#define CRYPT0_SECP256_SIG_COMPACT_BYTES 64

#ifdef __cplusplus
extern "C" {
#endif

/** generate a secret using pubkey and a private key
 * publen = 33
 * privlen = 32
 * secretlen = 32
 */
int crypt0_secp256k1_ecdh_secret(uint8_t * priv, int privlen, uint8_t * pub, int publen, uint8_t *secret, int secretlen);

/** convert secret key to public */
int crypt0_secp256k1_public_key(const uint8_t * priv, size_t privlen, 
        uint8_t * pub, size_t publen);

/** convert secret key to public */
int crypt0_secp256k1_public_key_compressed(const uint8_t * priv, size_t privlen, 
        uint8_t * pub, size_t publen);

/** sign sha256 hash */
int crypt0_secp256k1_sign(const uint8_t * hash, size_t hashlen, 
        const uint8_t * priv, size_t privlen, uint8_t * sig, uint8_t siglen);

/** sign sha256 hash recoverable */
int crypt0_secp256k1_sign_recoverable(const uint8_t * hash, size_t hashlen, 
        const uint8_t * priv, size_t privlen, uint8_t * sig, uint8_t siglen);

#ifdef __cplusplus
}
#endif

/* Limbs of the secp256k1 order. */
#define SECP256K1_N_0 ((uint32_t)0xD0364141UL)
#define SECP256K1_N_1 ((uint32_t)0xBFD25E8CUL)
#define SECP256K1_N_2 ((uint32_t)0xAF48A03BUL)
#define SECP256K1_N_3 ((uint32_t)0xBAAEDCE6UL)
#define SECP256K1_N_4 ((uint32_t)0xFFFFFFFEUL)
#define SECP256K1_N_5 ((uint32_t)0xFFFFFFFFUL)
#define SECP256K1_N_6 ((uint32_t)0xFFFFFFFFUL)
#define SECP256K1_N_7 ((uint32_t)0xFFFFFFFFUL)

/* Limbs of 2^256 minus the secp256k1 order. */
#define SECP256K1_N_C_0 (~SECP256K1_N_0 + 1)
#define SECP256K1_N_C_1 (~SECP256K1_N_1)
#define SECP256K1_N_C_2 (~SECP256K1_N_2)
#define SECP256K1_N_C_3 (~SECP256K1_N_3)
#define SECP256K1_N_C_4 (1)

/* Limbs of half the secp256k1 order. */
#define SECP256K1_N_H_0 ((uint32_t)0x681B20A0UL)
#define SECP256K1_N_H_1 ((uint32_t)0xDFE92F46UL)
#define SECP256K1_N_H_2 ((uint32_t)0x57A4501DUL)
#define SECP256K1_N_H_3 ((uint32_t)0x5D576E73UL)
#define SECP256K1_N_H_4 ((uint32_t)0xFFFFFFFFUL)
#define SECP256K1_N_H_5 ((uint32_t)0xFFFFFFFFUL)
#define SECP256K1_N_H_6 ((uint32_t)0xFFFFFFFFUL)
#define SECP256K1_N_H_7 ((uint32_t)0x7FFFFFFFUL)

// Bitcoin Core https://github.com/bitcoin-core/secp256k1
inline static int secp256k1_scalar_check_overflow(const uint32_t *a) {
    int yes = 0;
    int no = 0;
    no |= (a[7] < SECP256K1_N_7); /* No need for a > check. */
    no |= (a[6] < SECP256K1_N_6); /* No need for a > check. */
    no |= (a[5] < SECP256K1_N_5); /* No need for a > check. */
    no |= (a[4] < SECP256K1_N_4);
    yes |= (a[4] > SECP256K1_N_4) & ~no;
    no |= (a[3] < SECP256K1_N_3) & ~yes;
    yes |= (a[3] > SECP256K1_N_3) & ~no;
    no |= (a[2] < SECP256K1_N_2) & ~yes;
    yes |= (a[2] > SECP256K1_N_2) & ~no;
    no |= (a[1] < SECP256K1_N_1) & ~yes;
    yes |= (a[1] > SECP256K1_N_1) & ~no;
    yes |= (a[0] >= SECP256K1_N_0) & ~no;
    return yes;
}

inline static int secp256k1_scalar_reduce(uint32_t *r, uint32_t overflow) {
    uint64_t t;
    t = (uint64_t)r[0] + overflow * SECP256K1_N_C_0;
    r[0] = t & 0xFFFFFFFFUL; t >>= 32;
    t += (uint64_t)r[1] + overflow * SECP256K1_N_C_1;
    r[1] = t & 0xFFFFFFFFUL; t >>= 32;
    t += (uint64_t)r[2] + overflow * SECP256K1_N_C_2;
    r[2] = t & 0xFFFFFFFFUL; t >>= 32;
    t += (uint64_t)r[3] + overflow * SECP256K1_N_C_3;
    r[3] = t & 0xFFFFFFFFUL; t >>= 32;
    t += (uint64_t)r[4] + overflow * SECP256K1_N_C_4;
    r[4] = t & 0xFFFFFFFFUL; t >>= 32;
    t += (uint64_t)r[5];
    r[5] = t & 0xFFFFFFFFUL; t >>= 32;
    t += (uint64_t)r[6];
    r[6] = t & 0xFFFFFFFFUL; t >>= 32;
    t += (uint64_t)r[7];
    r[7] = t & 0xFFFFFFFFUL;
    return overflow;
}

inline static int secp256k1_scalar_add(uint32_t *r, const uint32_t *a, const uint32_t *b) {
    int overflow = 0;
    uint64_t t = (uint64_t)a[0] + b[0];
    r[0] = t & 0xFFFFFFFFULL; t >>= 32;
    t += (uint64_t)a[1] + b[1];
    r[1] = t & 0xFFFFFFFFULL; t >>= 32;
    t += (uint64_t)a[2] + b[2];
    r[2] = t & 0xFFFFFFFFULL; t >>= 32;
    t += (uint64_t)a[3] + b[3];
    r[3] = t & 0xFFFFFFFFULL; t >>= 32;
    t += (uint64_t)a[4] + b[4];
    r[4] = t & 0xFFFFFFFFULL; t >>= 32;
    t += (uint64_t)a[5] + b[5];
    r[5] = t & 0xFFFFFFFFULL; t >>= 32;
    t += (uint64_t)a[6] + b[6];
    r[6] = t & 0xFFFFFFFFULL; t >>= 32;
    t += (uint64_t)a[7] + b[7];
    r[7] = t & 0xFFFFFFFFULL; t >>= 32;
    overflow = t + secp256k1_scalar_check_overflow(r);

    secp256k1_scalar_reduce(r, overflow);
    return overflow;
}
#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Wunused-function"
static void secp256k1_scalar_set_b32(uint32_t *r, const unsigned char *b32) {
    //int over;
    r[0] = (uint32_t)b32[31] | (uint32_t)b32[30] << 8 | (uint32_t)b32[29] << 16 | (uint32_t)b32[28] << 24;
    r[1] = (uint32_t)b32[27] | (uint32_t)b32[26] << 8 | (uint32_t)b32[25] << 16 | (uint32_t)b32[24] << 24;
    r[2] = (uint32_t)b32[23] | (uint32_t)b32[22] << 8 | (uint32_t)b32[21] << 16 | (uint32_t)b32[20] << 24;
    r[3] = (uint32_t)b32[19] | (uint32_t)b32[18] << 8 | (uint32_t)b32[17] << 16 | (uint32_t)b32[16] << 24;
    r[4] = (uint32_t)b32[15] | (uint32_t)b32[14] << 8 | (uint32_t)b32[13] << 16 | (uint32_t)b32[12] << 24;
    r[5] = (uint32_t)b32[11] | (uint32_t)b32[10] << 8 | (uint32_t)b32[9] << 16 | (uint32_t)b32[8] << 24;
    r[6] = (uint32_t)b32[7] | (uint32_t)b32[6] << 8 | (uint32_t)b32[5] << 16 | (uint32_t)b32[4] << 24;
    r[7] = (uint32_t)b32[3] | (uint32_t)b32[2] << 8 | (uint32_t)b32[1] << 16 | (uint32_t)b32[0] << 24;
}

static void secp256k1_scalar_get_b32(unsigned char *bin, const uint32_t* a) {
    bin[0] = a[7] >> 24; bin[1] = a[7] >> 16; bin[2] = a[7] >> 8; bin[3] = a[7];
    bin[4] = a[6] >> 24; bin[5] = a[6] >> 16; bin[6] = a[6] >> 8; bin[7] = a[6];
    bin[8] = a[5] >> 24; bin[9] = a[5] >> 16; bin[10] = a[5] >> 8; bin[11] = a[5];
    bin[12] = a[4] >> 24; bin[13] = a[4] >> 16; bin[14] = a[4] >> 8; bin[15] = a[4];
    bin[16] = a[3] >> 24; bin[17] = a[3] >> 16; bin[18] = a[3] >> 8; bin[19] = a[3];
    bin[20] = a[2] >> 24; bin[21] = a[2] >> 16; bin[22] = a[2] >> 8; bin[23] = a[2];
    bin[24] = a[1] >> 24; bin[25] = a[1] >> 16; bin[26] = a[1] >> 8; bin[27] = a[1];
    bin[28] = a[0] >> 24; bin[29] = a[0] >> 16; bin[30] = a[0] >> 8; bin[31] = a[0];
}
#pragma GCC diagnostic pop
#endif//__crypt0_secp256k1_h_included__


