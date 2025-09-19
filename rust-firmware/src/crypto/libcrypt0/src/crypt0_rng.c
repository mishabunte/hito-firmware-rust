#include <crypt0_rng.h>

#ifdef __ZEPHYR__

#include <psa/crypto.h> 

bool crypt0_rng(uint8_t * data, int dataLen) 
{
  psa_crypto_init();
  psa_status_t status = psa_generate_random(data, dataLen);  
  if (status != PSA_SUCCESS) {                                                
    return false;
  } else {
    return true;
  }
}

#else

#include <openssl/rand.h>

bool crypt0_rng(uint8_t * data, int dataLen) 
{
  int ret = RAND_bytes(data, dataLen); 
  return ret == 1;
}

#endif
