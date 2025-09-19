#include <crypt0_aes_ccm.h>

#ifdef __ZEPHYR__
#include <ocrypto_aes_ccm.h>
#else
#include <openssl/evp.h>
#include <openssl/rand.h>
#endif

#include <string.h>
#include <stdint.h>

#define KEY_SIZE 32
#define NONCE_SIZE 14
#define TAG_SIZE 8

#ifdef __ZEPHYR__
#define handleErrors() { hito_reboot(); }
#else
#define handleErrors() \
{ \
    printf("An error occurred\n"); \
    exit(1); \
}
#endif

#ifndef __ZEPHYR__

int crypt0_encrypt_aes_ccm_openssl(
  const uint8_t *plaintext, int plaintext_len, 
  const uint8_t *key, int key_len, 
  const uint8_t *nonce, int nonce_len,
  const uint8_t *aad, int aad_len,
  uint8_t *ciphertext, uint8_t *tag, int tag_len)
{
    EVP_CIPHER_CTX *ctx;

    int len;
    int ciphertext_len;

    // Create and initialize the context
    if(!(ctx = EVP_CIPHER_CTX_new())) {
        handleErrors();
    }

    // Initialize the encryption operation
    if(1 != EVP_EncryptInit_ex(ctx, EVP_aes_256_ccm(), NULL, NULL, NULL))
        handleErrors();

    // Set nonce len
    if(1 != EVP_CIPHER_CTX_ctrl(ctx, EVP_CTRL_CCM_SET_IVLEN, nonce_len, NULL))
        handleErrors();
    // Set tag len
    if(1 != EVP_CIPHER_CTX_ctrl(ctx, EVP_CTRL_CCM_SET_TAG, tag_len, NULL))
        handleErrors();

    // Set the key and nonce
    if(1 != EVP_EncryptInit_ex(ctx, NULL, NULL, key, nonce))
        handleErrors();

    /* Provide the total plaintext length */
    if(1 != EVP_EncryptUpdate(ctx, NULL, &len, NULL, plaintext_len))
        handleErrors();

    /* Provide any AAD data. This can be called zero or one times as required */
    if(1 != EVP_EncryptUpdate(ctx, NULL, &len, aad, aad_len))
        handleErrors();

    /*
     * Provide the message to be encrypted, and obtain the encrypted output.
     * EVP_EncryptUpdate can only be called once for this.
     */
    if(1 != EVP_EncryptUpdate(ctx, ciphertext, &len, plaintext, plaintext_len))
        handleErrors();
    ciphertext_len = len;

    /*
     * Finalise the encryption. Normally ciphertext bytes may be written at
     * this stage, but this does not occur in CCM mode.
     */
    if(1 != EVP_EncryptFinal_ex(ctx, ciphertext + len, &len))
        handleErrors();
    ciphertext_len += len;

    /* Get the tag */
    if(1 != EVP_CIPHER_CTX_ctrl(ctx, EVP_CTRL_CCM_GET_TAG, tag_len, tag))
        handleErrors();

    EVP_CIPHER_CTX_free(ctx);

    return ciphertext_len;
}

int crypt0_decrypt_aes_ccm_openssl(const uint8_t * ciphertext, int ciphertext_len, 
                                   const uint8_t * key,   int key_len,
                                   const uint8_t * nonce, int nonce_len,
                                   const uint8_t * aad,   int aad_len,
                                   const uint8_t * tag,   int tag_len,
                                   uint8_t *plaintext)
{
    EVP_CIPHER_CTX *ctx;

    int len;
    int plaintext_len;
    int ret;

    /* Create and initialise the context */
    if(!(ctx = EVP_CIPHER_CTX_new()))
        handleErrors();

    /* Initialise the decryption operation. */
    if(1 != EVP_DecryptInit_ex(ctx, EVP_aes_256_ccm(), NULL, NULL, NULL))
        handleErrors();

    /* Setting IV len to 7. Not strictly necessary as this is the default
     * but shown here for the purposes of this example */
    if(1 != EVP_CIPHER_CTX_ctrl(ctx, EVP_CTRL_CCM_SET_IVLEN, nonce_len, NULL))
        handleErrors();

    /* Set expected tag value. */
    if(1 != EVP_CIPHER_CTX_ctrl(ctx, EVP_CTRL_CCM_SET_TAG, tag_len, (uint8_t *)tag))
        handleErrors();

    /* Initialise key and IV */
    if(1 != EVP_DecryptInit_ex(ctx, NULL, NULL, key, nonce))
        handleErrors();


    /* Provide the total ciphertext length */
    if(1 != EVP_DecryptUpdate(ctx, NULL, &len, NULL, ciphertext_len))
        handleErrors();

    /* Provide any AAD data. This can be called zero or more times as required */
    if(1 != EVP_DecryptUpdate(ctx, NULL, &len, aad, aad_len))
        handleErrors();

    /*
     * Provide the message to be decrypted, and obtain the plaintext output.
     * EVP_DecryptUpdate can be called multiple times if necessary
     */
    ret = EVP_DecryptUpdate(ctx, plaintext, &len, ciphertext, ciphertext_len);

    plaintext_len = len;

    /* Clean up */
    EVP_CIPHER_CTX_free(ctx);

    if(ret > 0) {
        /* Success */
        return plaintext_len;
    } else {
        /* Verify failed */
        return -1;
    }

}
#endif

int crypt0_encrypt_aes_ccm(const uint8_t *plaintext, int plaintext_len, 
                           const uint8_t *key,   int key_len,
                           const uint8_t *nonce, int nonce_len,
                           const uint8_t *aad,   int aad_len,
                           uint8_t *ciphertext, uint8_t *tag, int tag_len) 
{
    //retu2rn 1;
  #ifdef __ZEPHYR__
    ocrypto_aes_ccm_encrypt(ciphertext,
            tag, tag_len,
            plaintext, plaintext_len,
            key, key_len,
            nonce, nonce_len,
            aad, aad_len);

    // TODO: verify tag

    return plaintext_len;
  #else
    return crypt0_encrypt_aes_ccm_openssl(plaintext, plaintext_len, key, key_len, 
           nonce, nonce_len, aad, aad_len, ciphertext, tag, tag_len);
  #endif

}

// return decrypted len on success
int crypt0_decrypt_aes_ccm(const uint8_t *ciphertext, int ciphertext_len, 
                           const uint8_t *key,   int key_len,
                           const uint8_t *nonce, int nonce_len,
                           const uint8_t *aad,   int aad_len,
                           const uint8_t *tag, int tag_len,
                           uint8_t *plaintext)
{
    //retu2rn 1;
  #ifdef __ZEPHYR__
    int res = ocrypto_aes_ccm_decrypt(plaintext,
            tag, tag_len, ciphertext, ciphertext_len,
            key, key_len,
            nonce, nonce_len,
            aad, aad_len);
    return res == 0 ? ciphertext_len : -1;
  #else
    return crypt0_decrypt_aes_ccm_openssl(ciphertext, ciphertext_len, key, key_len, 
           nonce, nonce_len, aad, aad_len, tag, tag_len, plaintext);
  #endif
}

