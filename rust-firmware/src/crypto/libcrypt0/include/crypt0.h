#ifndef __crypt0_h_included__
#define __crypt0_h_included__
#ifdef __cplusplus
extern "C" {
#endif

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>

#include <intc.h> // int128
//typedef FStar_UInt128_uint128 uint128_t;

int crypt0_bech32_encode(const uint8_t * data, int datalen, char * buf, int buflen);

int crypt0_base58_encode(const uint8_t * data, int datalen, char * buf, int buflen);

int crypt0_base58_checksum_address(const uint8_t * pub, int publen, uint8_t prefix, char * buf, int buflen);
int crypt0_base58_checksum_hash(const uint8_t * hash, int hashlen, uint8_t prefix, char * buf, int buflen);

int crypt0_bech32_witness_v0_encode(const uint8_t * hash, int len, const char * hrp, char * buf, int buflen);


#ifndef __ZEPHYR__
bool crypt0_bin2hex(const uint8_t * bin, size_t binlen, 
                    char * hex, size_t hexlen);
bool crypt0_hex2bin(const char * hex, size_t hexlen, 
                    uint8_t * bin, size_t binlen);
#else
#include <sys/util.h>
#define crypt0_bin2hex bin2hex
#define crypt0_hex2bin hex2bin
#endif

// crypto ber2raw error codes
enum {

  // ber2raw error codes
  CRYPT0_ERR_SIGNATURE_HEADER = -1,
  CRYPT0_ERR_SIGNATURE_LENGTH = -2,
  CRYPT0_ERR_SIGNATURE_R_LENGTH = -3,
  CRYPT0_ERR_SIGNATURE_S_LENGTH = -4,

  CRYPT0_ERR_IMPORT_HMAC_KEY    = -5,
  CRYPT0_ERR_HMAC_DIGEST        = -6,
  CRYPT0_ERR_HMAC_LEN           = -7,

  CRYPT0_ERR_INIT_MBETLS        = -8,
  CRYPT0_ERR_INIT_PSA           = -9,
  CRYPT0_ERR_INIT_SECP256K1     = -10,

  CRYPT0_ERR_IMPORT_KEY         = -11,
  CRYPT0_ERR_EXPORT_KEY         = -12,

  CRYPT0_ERR_PUBKEY_LEN         = -13,
  CRYPT0_ERR_OUTBUF_LEN         = -14,
  CRYPT0_ERR_PRIVKEY_LEN        = -15,

  CRYPT0_ERR_FMT                = -16,
  CRYPT0_ERR_NOT_IMPLEMENTED    = -17,

  CRYPT0_ERR_SIG_LEN            = -18,
  CRYPT0_ERR_HASH_LEN           = -19,

  CRYPT0_ERR_SHA3               = -20,

  CRYPT0_ERR_RLP_LIST_HEADER    = -21,
  CRYPT0_ERR_RLP_BUFLEN         = -22,

  CRYPT0_ERR_RNG                = -23,
  CRYPT0_ERR_ENTROPY_LEN        = -24,

  CRYPT0_ERR                    = -25,
  CRYPT0_ERR_SALTLEN            = -26,

  CRYPT0_SHA256_BYTES           = 32,

  CRYPT0_BIP32_ENTROPY_BYTES = 16,
  CRYPT0_BIP32_SEED_BYTES    = 64,
  CRYPT0_BIP32_SECRET_BYTES  = 64,

  // ok
  CRYPT0_OK = 0
};

//
int crypt0_init();

/**
 * convert crypto signature from ber to raw format, 
 * returns CRYPT0_OK on success, raw buffer should be 0x40 bytes
 */
int crypt0_sig_ber2raw(const uint8_t * ber, uint8_t * raw);

//int crypt0_rng(uint8_t * rng, size_t rnglen);

#ifdef __cplusplus
}
#endif
#endif//__crypt0_h_included__
