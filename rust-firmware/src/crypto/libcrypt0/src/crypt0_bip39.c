#include "crypt0.h"
#include "crypt0_bip39.h"
#include "crypt0_sha.h"
#include "crypt0_hmac.h"
#include "crypt0_pbkdf2.h"

#include "crypt0_log.h"

#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Wunused-variable"
LOG_MODULE_REGISTER(crypt0_bip39, LOG_LEVEL_DBG);
#pragma GCC diagnostic pop

#include <string.h>

#include "crypt0_bip39_english.c"
//extern static const char * crypt0_bip39_english[];

#define CRYPT0_BIP39_MNEMONIC_WORDS 2048

//-----------------------------------------------------------------------------
/** 
 * Converts entropy to mnemonic english
 * returns mnemonic length on success, negative error code otherwise 
 */
int crypt0_bip39_entropy_to_mnemonic_en(const uint8_t * entropy, 
    uint8_t entropy_len, uint8_t * mnemonic, size_t mnemonic_len)
{
  if (entropy_len != 32 && entropy_len != 16 && entropy_len != 24) {
    LOG_ERR("wrong entropy len: %d, only 16/24/32 bytes accepted", entropy_len);
    return CRYPT0_ERR_ENTROPY_LEN;
  }
  memset(mnemonic, 0, mnemonic_len);

  uint8_t buf[33];

  // build checksum 
  crypt0_sha256(entropy, entropy_len, buf, CRYPT0_SHA256_BYTES);
  buf[entropy_len] = buf[0];
  memcpy(buf, entropy, entropy_len);

  for(int i = 0; i < (entropy_len * 8 + 8) / 11; i++) {
    uint32_t word_index;
    const uint8_t * p = buf + (i * 11) / 8;
    word_index = (uint32_t)*p++ << 16;
    word_index += (uint32_t)*p++ << 8;
    word_index += (uint32_t)*p++;

    uint32_t shift = 8 - (i + 1) * 11 % 8;
    if (shift == 8) {
      shift = 8;
    } else if (shift <= 5) {
      shift += 8;
    }
    word_index >>= shift;
    word_index &= 0x7ff;

    if (word_index >= CRYPT0_BIP39_MNEMONIC_WORDS) {
      LOG_ERR("word_index is too big: %d, max: %d", (int)word_index,
          CRYPT0_BIP39_MNEMONIC_WORDS);
      return CRYPT0_ERR;
    }
    const char * word = crypt0_bip39_english[word_index];
    if (strlen(word) + strlen((char *)mnemonic) + 1 > mnemonic_len) {
      LOG_ERR("mnemonic len is too small %d, need %d bytes",
          (int)mnemonic_len, (int)(strlen((char *)mnemonic) + strlen(word)));
      return CRYPT0_ERR;
    }

    if (i != 0) {
      strncat((char *)mnemonic, " ", mnemonic_len);
    }
    strncat((char *)mnemonic, word, mnemonic_len);
  }
  
  return strlen((char *)mnemonic);

}

//-----------------------------------------------------------------------------
/** 
 * Converts mnemonic to seed, 
 * returns CRYPT0_OK on success, negative error code otherwise
 */
int crypt0_bip39_mnemonic_to_seed(const uint8_t * mnemonic, 
    uint16_t mnemonic_len, uint8_t * seed, uint16_t seed_len)
{
  if (seed_len != 64) {
    LOG_ERR("seed_len is wrong: %d", (int)seed_len);
    return CRYPT0_ERR_FMT;
  }
	int ret = crypt0_pbkdf2_hmac_sha512(2048, mnemonic, mnemonic_len,
			(unsigned char *)"mnemonic", strlen("mnemonic"), seed, seed_len);
  return ret;
}

bool crypt0_bip39_ton_mnemonic_to_entropy(const uint8_t *mnemonic, uint16_t mnemonic_len, uint8_t *entropy, uint16_t entropy_len) {
  if (!mnemonic || !entropy) {
    return false;
  }
  if (crypt0_hmac_sha512(mnemonic, mnemonic_len, (const uint8_t *)"", 0, entropy) != 0) {
    LOG_ERR("HMAC SHA512 failed");
    return false;
  }

  return true;
}

int count_words(const char *mnemonic) {
    int count = 0;
    const char *p = mnemonic;
    int in_word = 0;

    while (*p) {
        if (*p != ' ' && !in_word) {
            in_word = 1;
            count++;
        } else if (*p == ' ') {
            in_word = 0;
        }
        p++;
    }
    return count;
  }

bool crypt0_bip39_mnemonic_to_entropy(const uint16_t *mnemonic, uint16_t mnemonic_len, uint8_t *entropy, uint16_t entropy_len)
{
  if (mnemonic_len == 12) {
    if (entropy_len != 16) {
      LOG_ERR("wrong entropy len: %d, 16 bytes expected", entropy_len);
      return false;
    }
  } else if (mnemonic_len == 18) {
    if (entropy_len != 24) {
      LOG_ERR("wrong entropy len: %d, 24 bytes expected", entropy_len);
      return false;
    }
  } else if (mnemonic_len == 24) {
    if (entropy_len != 32) {
      LOG_ERR("wrong entropy len: %d, 32 bytes expected", entropy_len);
      return false;
    }
  } else {
    LOG_ERR("wrong len: %d, only 12/18/24 words accepted", mnemonic_len);
    return false;
  }

  uint8_t entropy_with_checksum[33];
  memset(entropy_with_checksum, 0, 33);

  // iterate over words
  for(int i = 0; i < mnemonic_len; i++) {
    //LOG_DBG("mnemonic #%02d: %x", i, mnemonic[i]);
    // iterate over 11 bits in each word
    for(int j = 0; j < 11; j++) {
      if ((mnemonic[i] & (1 << (10 - j))) != 0) {
        //LOG_DBG("i: %d, j: %d", i, j);
        // set the corresponding bit in the entropy
        int bitPosition = i * 11 + j;
        int bytePosition = bitPosition / 8;
        int bit = bitPosition % 8;
        entropy_with_checksum[bytePosition] |= (1u << (7 - bit));
      }
    }
  }

  //char ee[67];
  //memset(ee, 0, 67);
  //ee[0] = 0x30;
  //crypt0_bin2hex(entropy_with_checksum, 33, ee, 67);
  //LOG_DBG("entropy_with_checksum: [\n%s]", ee);

  // check the checksum of the entropy
  uint8_t sha256[32];
  memset(sha256, 0, 32);
  crypt0_sha256(entropy_with_checksum, entropy_len, sha256, 32);

  // calculate the number of bits in the checksum and trim it
  int checksum_bits = mnemonic_len * 11 - entropy_len * 8;

  //LOG_DBG("checksum_bits: %d", checksum_bits);

  uint8_t checksum_byte = 0;
  for(int i = 0; i < checksum_bits; i++) {
    if ((sha256[0] & (1 << (7 - i))) != 0) {
      checksum_byte |= (1 << (7 - i));
    }
  }

  if (checksum_byte != entropy_with_checksum[entropy_len]) {
    LOG_ERR("wrong checksum: %x, expected: %x", 
        entropy_with_checksum[entropy_len], checksum_byte);
    return false;
  }

  memcpy(entropy, entropy_with_checksum, entropy_len);

  return true;
}

int crypt0_ton_entropy_to_seed(const uint8_t *entropy, uint16_t entropy_len, uint8_t *seed_out) {
  if (!entropy || !seed_out || entropy_len != 64) return CRYPT0_ERR_FMT;
  int ret = crypt0_pbkdf2_hmac_sha512(100000, entropy, entropy_len,
      (unsigned char *)"TON default seed", strlen("TON default seed"), seed_out, 64);
  return ret;
}

int crypt0_ton_mnemonic_to_seed(const uint8_t *mnemonic_str, uint16_t mnemonic_len, uint8_t *seed_out, uint32_t seed_len) {
    if (!mnemonic_str || !seed_out || seed_len != 64) {
        LOG_ERR("Invalid input parameters");
        return CRYPT0_ERR_FMT;
    }
    // Count the number of words in the mnemonic
    int word_count = count_words((const char *)mnemonic_str);
    if (word_count != 12 && word_count != 18 && word_count != 24) {
        LOG_ERR("Unsupported word count: %d", word_count);
        return CRYPT0_ERR_FMT;
    }
    if (word_count != 24) {
      return crypt0_bip39_mnemonic_to_seed(mnemonic_str, mnemonic_len, seed_out, seed_len);
    }
    uint8_t entropy[64];
    memset(entropy, 0, sizeof(entropy));
    LOG_DBG("mnemonic_len: %d", mnemonic_len);
    bool ret = crypt0_bip39_ton_mnemonic_to_entropy(mnemonic_str, mnemonic_len, entropy, 64);
    if (!ret) return -1;


    return crypt0_ton_entropy_to_seed(entropy, 64, seed_out);
}

// int crypt0_bip39_mnemonic_to_ton_seed(const uint8_t *mnemonic, uint16_t mnemonic_len, uint8_t *seed, uint16_t seed_len) {
//   if (seed_len != 64) {
//     LOG_ERR("seed_len is wrong: %d", (int)seed_len);
//     return CRYPT0_ERR_FMT;
//   }
// 	int ret = crypt0_pbkdf2_hmac_sha512(2048, mnemonic, mnemonic_len,
// 			(unsigned char *)"TON default seed", strlen("TON default seed"), seed, seed_len);
//   return ret;
// }

// bool crypt0_bip39_mnemonic_to_entropy(const uint8_t *mnemonic, uint16_t mnemonic_len,
//   uint8_t *entropy, uint16_t entropy_len) {
//   if (!mnemonic || !entropy) {
//     return false;
//   }

//   // Convert input buffer to null-terminated string for word parsing
//   char mnemonic_str[256];
//   if (mnemonic_len >= sizeof(mnemonic_str)) {
//     LOG_ERR("mnemonic too long");
//     return false;
//   }
//   memcpy(mnemonic_str, mnemonic, mnemonic_len);
//   mnemonic_str[mnemonic_len] = '\0';

//   // Split mnemonic string into words
//   char *words[24] = {0};
//   int word_count = 0;
//   char *token = strtok(mnemonic_str, " ");
//   while (token && word_count < 24) {
//     words[word_count++] = token;
//     token = strtok(NULL, " ");
//   }

//   if (word_count != 12 && word_count != 18 && word_count != 24) {
//     LOG_ERR("unsupported word count: %d", word_count);
//     return false;
//   }
//   LOG_DBG("word count: %d", word_count);

//   // Check expected entropy length
//   uint16_t expected_entropy_len = (word_count * 11) / 33 * 4;
//   if (entropy_len != expected_entropy_len) {
//     LOG_ERR("unexpected entropy len: %d, expected: %d", entropy_len, expected_entropy_len);
//     return false;
//   }

//   // Convert words to indices
//   uint16_t indices[24] = {0};
//   for (int i = 0; i < word_count; i++) {
//     int idx = bip39_get_word_index(words[i]);
//     if (idx < 0) {
//       LOG_ERR("invalid word: %s", words[i]);
//       return false;
//     }
//     indices[i] = (uint16_t)idx;
//   }

//   // Pack indices into bitstream
//   uint8_t bitstream[33] = {0};  // max 24*11=264 bits
//   int bit_pos = 0;
//   for (int i = 0; i < word_count; i++) {
//     for (int j = 10; j >= 0; j--) {
//       if ((indices[i] >> j) & 1) {
//         bitstream[bit_pos / 8] |= 1 << (7 - (bit_pos % 8));
//       }
//       bit_pos++;
//     }
//   }

//   // Extract entropy and checksum
//   memcpy(entropy, bitstream, entropy_len);
//   int checksum_bits = word_count * 11 - entropy_len * 8;

//   // Calculate SHA-256 of entropy
//   uint8_t hash[32];
//   crypt0_sha256(entropy, entropy_len, hash, sizeof(hash));

//   // Compare checksums
//   uint8_t expected_checksum = hash[0] >> (8 - checksum_bits);
//   uint8_t actual_checksum = bitstream[entropy_len] >> (8 - checksum_bits);

//   if (expected_checksum != actual_checksum) {
//     LOG_ERR("checksum mismatch: got %02x, expected %02x",
//             actual_checksum, expected_checksum);
//     return false;
//   }

//   return true;
// }



// // convert mnemonic provided as array of words indexes to entropy
// bool crypt0_bip39_mnemonic_to_entropy(
//     const uint8_t * mnemonic, uint16_t mnemonic_len,
//     uint8_t * entropy, uint16_t entropy_len) 
// {
//   if (mnemonic_len == 12) {
//     if (entropy_len != 16) {
//       LOG_ERR("wrong entropy len: %d, 16 bytes expected", entropy_len);
//       return false;
//     }
//   } else if (mnemonic_len == 18) {
//     if (entropy_len != 24) {
//       LOG_ERR("wrong entropy len: %d, 24 bytes expected", entropy_len);
//       return false;
//     }
//   } else if (mnemonic_len == 24) {
//     if (entropy_len != 32) {
//       LOG_ERR("wrong entropy len: %d, 32 bytes expected", entropy_len);
//       return false;
//     }
//   } else {
//     LOG_ERR("wrong len: %d, only 12/18/24 words accepted", mnemonic_len);
//     return false;
//   }

//   uint8_t entropy_with_checksum[33];
//   memset(entropy_with_checksum, 0, 33);

//   // iterate over words
//   for(int i = 0; i < mnemonic_len; i++) {
//     // iterate over 11 bits in each word
//     for(int j = 0; j < 11; j++) {
//       if ((mnemonic[i] & (1 << (10 - j))) != 0) {
//         // set the corresponding bit in the entropy
//         int bitPosition = i * 11 + j;
//         int bytePosition = bitPosition / 8;
//         int bit = bitPosition % 8;
//         entropy_with_checksum[bytePosition] |= (1u << (7 - bit));
//       }
//     }
//   }

//   // check the checksum of the entropy
//   uint8_t sha256[32];
//   memset(sha256, 0, 32);
//   crypt0_sha256(entropy_with_checksum, entropy_len, sha256, 32);

//   // calculate the number of bits in the checksum and trim it
//   int checksum_bits = mnemonic_len * 11 - entropy_len * 8;

//   uint8_t checksum_byte = 0;
//   for(int i = 0; i < checksum_bits; i++) {
//     if ((sha256[0] & (1 << (7 - i))) != 0) {
//       checksum_byte |= (1 << (7 - i));
//     }
//   }

//   if (checksum_byte != entropy_with_checksum[entropy_len]) {
//     LOG_ERR("wrong checksum: %x, expected: %x", 
//         entropy_with_checksum[entropy_len], checksum_byte);
//     return false;
//   }

//   memcpy(entropy, entropy_with_checksum, entropy_len);

//   return true;
// }

/**
 * Convert entropy to seed using english mnemonic
 * returns CRYPT0_OK on success, negative error code otherwise
 */
int crypt0_bip39_entropy_to_seed_en(const uint8_t * entropy, 
    uint16_t entropyLen, uint8_t * seed, uint16_t seedLen) 
{
  uint8_t mnemonic[CRYPT0_BIP39_MNEMONIC_MAXBYTES];

  int mnemonicLen = crypt0_bip39_entropy_to_mnemonic_en(
      entropy, entropyLen, mnemonic, CRYPT0_BIP39_MNEMONIC_MAXBYTES);

  if (mnemonicLen <= 0) {
    return CRYPT0_ERR;
  }

  return crypt0_bip39_mnemonic_to_seed(mnemonic, mnemonicLen, seed, seedLen);
}

