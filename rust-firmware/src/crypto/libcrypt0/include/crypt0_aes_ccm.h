#ifndef __crypt0_aes_ccm_h_included__
#define __crypt0_aes_ccm_h_included__
#ifdef __cplusplus
extern "C" {
#endif

#include <stdint.h>

int crypt0_encrypt_aes_ccm(const uint8_t * plaintext, int plaintext_len, 
                           const uint8_t * key,   int key_len,
                           const uint8_t * nonce, int nonce_len,
                           const uint8_t * aad,   int aad_len,
                           uint8_t * ciphertext, uint8_t * tag, int tag_len);

int crypt0_decrypt_aes_ccm(const uint8_t *ciphertext, int ciphertext_len, 
                           const uint8_t *key,   int key_len,
                           const uint8_t *nonce, int nonce_len,
                           const uint8_t *aad,   int aad_len,
                           const uint8_t *tag, int tag_len,
                           uint8_t *plaintext);

#ifdef __cplusplus
}
#endif
#endif//__crypt0_aes_ccm_h_included__
