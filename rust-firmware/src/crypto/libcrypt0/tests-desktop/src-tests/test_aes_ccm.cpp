#include <string.h>
#include <stddef.h>
#include <stdint.h>

#include <crypt0_aes_ccm.h>

#include <iostream>

#include <crypt0_log.h>
LOG_MODULE_REGISTER(test_aes_ccm, LOG_LEVEL_DBG);

const char * addresses[][2] = {
  {"foo", "bar"},
  {"foo", "bar"},
  {"foo", "bar"}
};

using namespace std;

// Test AES CCM Encryption and Decryption
bool test_aes_ccm_encrypt_decrypt() 
{
  // source data for encription
  auto key       = "kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk";
  auto nonce     = "nnnnnnn";
  auto aad       = "aaaaaaa";
  auto plaintext = "Hello, world!";
  auto plaintext_len = strlen(plaintext);

  // ciphertext and tag
  uint8_t ciphertext[128];
  uint8_t plaintext_decrypted[128];
  const int tag_len = 8;
  uint8_t tag[tag_len];

  // expected data
  auto tag_expected        = "\x0d\xdb\x10\x92\x16\xf0\x3d\x7b";
  auto ciphertext_expected = 
    "\x16\x97\xb3\x8e\xf1\x4d\xb6\xf4\x42\x4d\x00\x3e\xbc";

  // encrypt data
  int res = crypt0_encrypt_aes_ccm((const uint8_t *)plaintext, strlen(plaintext),
      (const uint8_t *)key, strlen(key), (const uint8_t *)nonce, strlen(nonce), 
      (const uint8_t *)aad, strlen(aad),
      ciphertext, tag, tag_len);

  // check result
  if (res != plaintext_len) {
    LOG_ERR("AES CCM encryption failed, wrong length of encrypted text");
    return false;
  }
  if (memcmp(ciphertext, ciphertext_expected, plaintext_len) != 0) {
    LOG_ERR("AES CCM encryption failed, wrong ciphertext");
    return false;
  }
  if (memcmp(tag, tag_expected, tag_len) != 0) {
    LOG_ERR("AES CCM encryption failed, wrong tag");
    return false;
  }

  // decrypt data
  res = crypt0_decrypt_aes_ccm((const uint8_t *)ciphertext, plaintext_len,
      (const uint8_t *)key, strlen(key), (const uint8_t *)nonce, strlen(nonce), 
      (const uint8_t *)aad, strlen(aad),
      tag, tag_len,
      plaintext_decrypted);
  if (res != plaintext_len) {
    LOG_ERR("AES CCM encryption failed, wrong length of encrypted text");
    return false;
  }
  if (memcmp(plaintext_decrypted, plaintext, plaintext_len) != 0) {
    LOG_ERR("AES CCM encryption failed, wrong ciphertext");
    return false;
  }

  return true;
}

int main(void) 
{
  if (!test_aes_ccm_encrypt_decrypt()) {
    return 1;
  }

  return 0;
}
