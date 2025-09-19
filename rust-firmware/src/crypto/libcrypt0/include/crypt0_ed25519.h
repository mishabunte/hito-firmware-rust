#ifndef __crypt0_ed25519_h_included__
#define __crypt0_ed25519_h_included__

/** ed25519 privmitives */

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>


#define CRYPT0_ED25519_PRIVKEY_BYTES 32
#define CRYPT0_ED25519_PUBKEY_BYTES  32
#define CRYPT0_ED25519_SIG_BYTES     64 
#define CRYPT0_ED25519_HASH_BYTES_32 32
#define CRYPT0_ED25519_HASH_BYTES_64 64

/** convert secret key to public */
int crypt0_ed25519_public_key(const uint8_t * priv, size_t privlen, 
        uint8_t * pub, size_t publen);

/** sign sha256 hash 
 * @param message - message to sign
 * @param messagelen - message length
 * @param priv - private key
 * @param privlen - private key length (32 bytes)
 * @param pub - public key
 * @param publen - public key length (32 bytes)
 * @param sig - signature
 * @param siglen - signature length (64 bytes)
*/
int crypt0_ed25519_sign(
        const uint8_t * message, size_t messagelen, 
        const uint8_t * priv, size_t privlen, 
        const uint8_t * pub,  size_t publen, 
        uint8_t * sig, uint8_t siglen);

int crypt0_ed25519_derive_secret_index(uint8_t * secret, uint32_t index);

#endif//__crypt0_ed25519_h_included__
