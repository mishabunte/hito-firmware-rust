
extern "C" 
{
#include "crypt0_bip32.h"
#include "crypt0_bip39.h"
#include "crypt0.h"
#include "crypt0_sha.h"

#include <crypt0_log.h>

LOG_MODULE_REGISTER(extended_pubkey, LOG_LEVEL_DBG);
}

void error_exit(const char * function, const char * message) {
  fprintf(stderr, "..... Error in %s: %s\n", function, message);
  exit(1);
}

char * test_seed_text = "wild casual icon cream oven boil";
char * pubkey_exp_hex = "0488b21e03b88f586e0000000086d7c38e5d268df917c72a3bb79c24273b92aa98c9c5b5f1c3bf76f4cb7699340295f1e62f3d1ff2e431c30f035c44fd1ede65c0318721b1024a2279782c82feba9549eb05";
char * seed_exp = "1529b43e38c0ff3d6561a64b2464994b9ac888d2e85b32aec554d224fe85b439f6d1fe1ac75ec1c24c26e93eaa49d7135176538b598b652d3203239bd15f4b5c";

bool test__seed_to_extended_pubkey(){
    uint8_t test_seed[64];
    bzero(test_seed, 64);

    int res = crypt0_bip39_mnemonic_to_seed((const uint8_t *)test_seed_text, strlen(test_seed_text), test_seed, 64);
    if (res != CRYPT0_OK) {
      LOG_ERR("crypt0_bip39_mnemonic_to_seed failed: %d", res);
      return false;
    }

    char seed_hex[129];
    bzero(seed_hex, 129);
    crypt0_bin2hex(test_seed, 64, seed_hex, 129);
    if (memcmp(seed_hex, seed_exp, 129))
    {
        LOG_ERR("seed do not match");
        LOG_ERR("expected seed: %s", seed_exp);
        LOG_ERR("seed got: %s", seed_hex);
        return false;
    }

    uint8_t pubkey[78];
    if (crypt0_bip32_seed_to_account_pubkey(test_seed, pubkey, 44, 60, 0) == -1)
      return false;

    uint8_t pubkey_exp[78];
    crypt0_hex2bin(pubkey_exp_hex, 156, pubkey_exp, 78);

    if (memcmp(pubkey_exp, pubkey, 78))
    {
        LOG_ERR("seed to extended pubkey error");
        LOG_HEXDUMP_DBG(pubkey_exp, 78, "Pubkey expected");
        LOG_HEXDUMP_DBG(pubkey, 78, "Pubkey result");
        return false;
    }

    return true;
}

char * test_seed_text_hardened = "wrap ensure cannon foam common save another embark lobster inflict flavor soldier";
char * pubkey_exp_hex_hardened = "0488b21e03de3a803b80000000d5b0697db44d0c168fe09e0af26d4cc7c1d789b8534217100fc4bb298e3051c00374af29b30c0c678695d41f8a4646dae0e6a3431cc9b2a9f0cb07eadb0836cd17fc0ab241";
char * seed_exp_hardened = "a496d7d0f2f7ae7115a8512eea3ba8bfaba83133692174b76e3b5f0e4c473666fe58fddb0544ec19811ddfbcc88ee5e2c8a9bf8dcb1b1ed2a8668f9134a72f47";

bool test__seed_to_extended_pubkey_hardened(){
    uint8_t test_seed[64];
    bzero(test_seed, 64);

    int res = crypt0_bip39_mnemonic_to_seed((const uint8_t *)test_seed_text_hardened, strlen(test_seed_text_hardened), test_seed, 64);
    if (res != CRYPT0_OK) {
      LOG_ERR("crypt0_bip39_mnemonic_to_seed failed: %d", res);
      return false;
    }

    char seed_hex[129];
    bzero(seed_hex, 129);
    crypt0_bin2hex(test_seed, 64, seed_hex, 129);
    if (memcmp(seed_hex, seed_exp_hardened, 129))
    {
        LOG_ERR("seed do not match");
        LOG_ERR("expected seed: %s", seed_exp_hardened);
        LOG_ERR("seed got: %s", seed_hex);
        return false;
    }
    
    crypt0_hex2bin(seed_hex, 129, test_seed, 64);

    uint8_t pubkey[78];
    if (crypt0_bip32_seed_to_account_pubkey(test_seed, pubkey, 44 + CRYPT0_BIP32_INDEX_HARDENED, 56 + CRYPT0_BIP32_INDEX_HARDENED, 0 + CRYPT0_BIP32_INDEX_HARDENED) == -1)
      return false;

    uint8_t pubkey_exp[78];
    crypt0_hex2bin(pubkey_exp_hex_hardened, 156, pubkey_exp, 78);

    if (memcmp(pubkey_exp, pubkey, 78))
    {
        LOG_ERR("seed to extended pubkey error");
        LOG_HEXDUMP_DBG(pubkey_exp, 78, "Pubkey expected");
        LOG_HEXDUMP_DBG(pubkey, 78, "Pubkey result");
        return false;
    }

    return true;
}

char * test_seed_base58 = "occur wrap divert rely write poverty name slush color chief amused tiny cup purity sheriff dignity mix night joy cheese earth friend drift trouble";

char * exp_base58 = "vpub5ZYDAJzy6iuDK7nxnBZjeeeUauM1Zn5kUKj35Bs3KByn6RgDWQK1KocVADiDHrSqMnyNmdY33MESTEjSykhZ4Hbp3KtQtFNX7h132y2nq3o";

bool test__seed_to_extended_pubkey_base58()
{
  uint8_t test_seed[64];
  bzero(test_seed, 64);

  int res = crypt0_bip39_mnemonic_to_seed((const uint8_t *)test_seed_base58, strlen(test_seed_base58), test_seed, 64);
  if (res != CRYPT0_OK)
  {
    LOG_ERR("crypt0_bip39_mnemonic_to_seed failed: %d", res);
    return false;
  }

  uint8_t pubkey[78 + 4];
  if (crypt0_bip32_seed_to_account_pubkey(test_seed, pubkey, 84 + CRYPT0_BIP32_INDEX_HARDENED, 1 + CRYPT0_BIP32_INDEX_HARDENED, 0 + CRYPT0_BIP32_INDEX_HARDENED) == -1)
    return false;

  char base58_pubkey[112];
  
  uint8_t sha[32];
	uint8_t sha2[32];
	crypt0_sha256(pubkey, 78, sha, 32);
	crypt0_sha256(sha, 32, sha2, 32);

	memcpy(&pubkey[78], sha2, 4);
  int ret = crypt0_base58_encode(pubkey, 78 + 4, base58_pubkey, 112);
  if (ret != strlen(exp_base58))
    {
      LOG_DBG("Base58 error, length got: %d", ret);
      return false;
    }

  if (strcmp(base58_pubkey, exp_base58))
  {
    LOG_ERR("seed to extended pubkey error");
    LOG_DBG("Pubkey expected %s", exp_base58);
    LOG_DBG("Pubkey result %s", base58_pubkey);
    return false;
  }

  return true;
}

int main()
{
  if (test__seed_to_extended_pubkey() == false)
    return 1;
  if (test__seed_to_extended_pubkey_hardened() == false)
    return 1;
  if (test__seed_to_extended_pubkey_base58() == false)
    return 1;
}