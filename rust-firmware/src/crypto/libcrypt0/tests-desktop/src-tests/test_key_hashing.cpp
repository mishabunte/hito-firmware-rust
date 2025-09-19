#include <string.h>
#include <stddef.h>
#include <stdint.h>

#include <crypt0.h>
#include <crypt0_sha.h>
#include <crypt0_ripemd160.h>
#include <crypt0_secp256k1.h>

#include <iostream>

#include <crypt0_log.h>
LOG_MODULE_REGISTER(test_key_hashing, LOG_LEVEL_DBG);

using namespace std;

const char * entropy_src = "correct horse battery staple";
const char * priv_expected = "c4bbcb1fbec99d65bf59d85c8cb62ee2db963f0fe106f483d9afa73bd4e39a8a";
const char * pub_expected_compressed   = "0378d430274f8c5ec1321338151e9f27f4c676a008bdf8638d07c0b6be9ab35c71";

const char * pub_expected_uncompressed = "0478d430274f8c5ec1321338151e9f27f4c676a008bdf8638d07c0b6be9ab35c71"
                                         "a1518063243acd4dfe96b66e3f2ec8013c8e072cd09b3834a19f81f659cc3455";

const char * hash160_expected = "79fbfc3f34e7745860d76137da68f362380c606c";
const char * hash160_expected_uncompressed = "c4c5d791fcb4654a1ef5e03fe0ad3d9c598f9827";

const char * eth_expected = "0x17B35f044A163541f076ECfF24F373337621Fc9D";
const char * btc_expected = "1C7zdTfnkzmr13HfA2vNm5SJYRK6nEKyq8";
const char * btc_expected_segwit = "bc1q08alc0e5ua69scxhvyma568nvguqccrv4cc9n4";
const char * btc_expected_p2sh_segwit = "3KToBU4ykTWfjnu4kAUV1q8QosnxT61sbf";

const char * btct_expected = "mrdwvWkma2D6n9mGsbtkazedQQuoksnqJV";
const char * btct_expected_segwit = "tb1q08alc0e5ua69scxhvyma568nvguqccrvl7rkgx";
const char * btct_expected_p2sh_segwit = "2NB21FD11Mv21waXcRJ6Mdn7g2E18HRwZhS";

const char * ltc_expected = "LWLwtfycqf1uFqypLAug36W4kdgNwrZdNs";
const char * ltc_expected_segwit = "ltc1q08alc0e5ua69scxhvyma568nvguqccrv3yzpt9";
const char * ltc_expected_p2sh_segwit = "MRfwVMUwhaN6YJAxr3TpqUNp8aPQWYHLZk";

const char * doge_expected  = "DGG6AicS4Qg8Y3UFtcuwJqbuRZ3Q7WtYXv";
const char * doget_expected = "nfK9tjMLzP8rR23SvSZPZFCCfRRhAhtwDo";

bool test_key_hashing() {
  // Create the (in)famous correct brainwallet secret key.
  uint8_t priv[32];
  uint8_t sha[32];
  uint8_t hash160[32];
  char buf[1024];

  uint8_t pub_compressed[65];
  uint8_t pub_uncompressed[65];


  crypt0_sha256((const uint8_t *)entropy_src, strlen(entropy_src), priv, 32);
  bzero(buf, sizeof(buf));
  crypt0_bin2hex(priv, 32, buf, 65);

  if (memcmp(buf, priv_expected, 64) != 0) {
    LOG_ERR("priv expected %s, got %s", priv_expected, buf);
    return false;
  }
  LOG_DBG("privkey         : %s", buf);

  // compressed key
  crypt0_secp256k1_public_key_compressed(priv, 32, pub_compressed, 33);
  bzero(buf, sizeof(buf));
  crypt0_bin2hex(pub_compressed, 33, buf, sizeof(buf));

  LOG_DBG("pubkey          : %s", buf);
  if (strcmp(pub_expected_compressed, buf) != 0) {
    LOG_ERR("expected          : %s", pub_expected_compressed);
    return false;
  }

  // uncompressed key
  crypt0_secp256k1_public_key(priv, 32, pub_uncompressed, 65);
  bzero(buf, sizeof(buf));
  crypt0_bin2hex(pub_uncompressed, 65, buf, sizeof(buf));

  LOG_DBG("  uncompressed  : %s", buf);
  if (strcmp(pub_expected_uncompressed, buf) != 0) {
    LOG_ERR("expected          : %s", pub_expected_uncompressed);
    return false;
  }

  crypt0_sha256(pub_compressed, 33, sha, 32);
  crypt0_ripemd160(sha, 32, hash160);
  crypt0_bin2hex(hash160, 20, buf, sizeof(buf));
  LOG_DBG("hash160         : %s", buf);
  if (strcmp(hash160_expected, buf) != 0) {
    LOG_ERR("expected          : %s", hash160_expected);
    return false;
  }

  crypt0_sha256(pub_uncompressed, 65, sha, 32);
  crypt0_ripemd160(sha, 32, hash160);
  crypt0_bin2hex(hash160, 20, buf, sizeof(buf));
  LOG_DBG("  uncompressed  : %s", buf);
  if (strcmp(hash160_expected_uncompressed, buf) != 0) {
    LOG_ERR("expected          : %s", hash160_expected_uncompressed);
    return false;
  }

  return true;
}

bool test_key_base58() 
{
  char buf[1024];
  uint8_t priv[32];

  uint8_t pub_compressed[65];

  crypt0_sha256((const uint8_t *)entropy_src, strlen(entropy_src), priv, 32);
  bzero(buf, sizeof(buf));
  crypt0_bin2hex(priv, 32, buf, 65);

  if (memcmp(buf, priv_expected, 64) != 0) {
    LOG_ERR("priv expected %s, got %s", priv_expected, buf);
    return false;
  }
  LOG_DBG("privkey         : %s", buf);

  // compressed key
  crypt0_secp256k1_public_key_compressed(priv, 32, pub_compressed, 33);
  bzero(buf, sizeof(buf));
  crypt0_bin2hex(pub_compressed, 33, buf, sizeof(buf));

  LOG_DBG("pubkey          : %s", buf);
  if (strcmp(pub_expected_compressed, buf) != 0) {
    LOG_ERR("expected          : %s", pub_expected_compressed);
    return false;
  }
  crypt0_base58_checksum_address(pub_compressed, 33, 0, buf, sizeof(buf));
  LOG_DBG("btc  address    : %s", buf);
  if (strcmp(btc_expected, buf) != 0) {
    LOG_ERR("expected          : %s", btc_expected);
    return false;
  }

  crypt0_base58_checksum_address(pub_compressed, 33, 0x6f, buf, sizeof(buf));
  LOG_DBG("btct address    : %s", buf);
  if (strcmp(btct_expected, buf) != 0) {
    LOG_ERR("expected          : %s", btct_expected);
    return false;
  }

  crypt0_base58_checksum_address(pub_compressed, 33, 0x30, buf, sizeof(buf));
  LOG_DBG("ltc  address    : %s", buf);
  if (strcmp(ltc_expected, buf) != 0) {
    LOG_ERR("expected          : %s", ltc_expected);
    return false;
  }

  crypt0_base58_checksum_address(pub_compressed, 33, 0x1e, buf, sizeof(buf));
  LOG_DBG("doge  address   : %s", buf);
  if (strcmp(doge_expected, buf) != 0) {
    LOG_ERR("expected          : %s", doge_expected);
    return false;
  }

  crypt0_base58_checksum_address(pub_compressed, 33, 0x71, buf, sizeof(buf));
  LOG_DBG("doget address   : %s", buf);
  if (strcmp(doget_expected, buf) != 0) {
    LOG_ERR("expected          : %s", doget_expected);
    return false;
  }

  return true;
}

bool test_key_bech32() 
{
  char buf[1024];

  uint8_t hash160[20];
  bzero(hash160, 20);
  crypt0_hex2bin(hash160_expected, strlen(hash160_expected), hash160, 20);

  bzero(buf, 1024);
  crypt0_bech32_witness_v0_encode(hash160, 20, "bc", buf, 1024);

  LOG_DBG("btc segwit :   %s", buf);
  if (strcmp(btc_expected_segwit, buf) != 0) {
    LOG_ERR("expected          : %s", btc_expected_segwit);
    return false;
  }

  crypt0_bech32_witness_v0_encode(hash160, 20, "tb", buf, 1024);
  LOG_DBG("btct segwit :  %s", buf);
  if (strcmp(btct_expected_segwit, buf) != 0) {
    LOG_ERR("expected          : %s", btct_expected_segwit);
    return false;
  }

  crypt0_bech32_witness_v0_encode(hash160, 20, "ltc", buf, 1024);
  LOG_DBG("ltc segwit :  %s", buf);
  if (strcmp(ltc_expected_segwit, buf) != 0) {
    LOG_ERR("expected          : %s", ltc_expected_segwit);
    return false;
  }

  return true;
}

int main(void) 
{

  if (!test_key_hashing()) {
    return 1;
  }

  if (!test_key_base58()) {
    return 1;
  }

  if (!test_key_bech32()) {
    return 1;
  }

  return 0;
}


