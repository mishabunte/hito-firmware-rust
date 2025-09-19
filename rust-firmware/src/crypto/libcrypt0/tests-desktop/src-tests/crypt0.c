#include "crypt0.h"

#include <string.h>

#ifdef __ZEPHYR__
#include <psa/crypto.h>
#include <mbedtls/platform.h>
#endif//__ZEPHYR__

#include "crypt0_log.h"
LOG_MODULE_REGISTER(crypt0, LOG_LEVEL_DBG); 

static bool m_crypt0_inited = false;

//-----------------------------------------------------------------------------
// Convert ber signature to raw format
int crypt0_sig_ber2raw(const uint8_t * ber, uint8_t * raw) 
{
  // check valid header buffer[0] == 0x30
  if (ber[0] != 0x30 || ber[2] != 0x02) {
    return CRYPT0_ERR_SIGNATURE_HEADER;
  }

  // check signature encoded length
  if (ber[1] < 0x44 || ber[1] > 0x46) { 
    return CRYPT0_ERR_SIGNATURE_LENGTH;
  }

  // check r encoded length
  if (ber[3] != 0x20 && ber[3] != 0x21) {
    return CRYPT0_ERR_SIGNATURE_R_LENGTH;
  }

  // check s encoded length
  uint8_t s_len = ber[4 + ber[3] + 1];
  if (s_len != 0x20 && s_len != 0x21) {
    return CRYPT0_ERR_SIGNATURE_S_LENGTH;
  }

  // copy r
  memcpy(raw, &ber[4 + (ber[3] == 0x21 ? 1 : 0)], 0x20);

  // copy s
  memcpy(&raw[0x20], &ber[4 + ber[3] + 2 + (s_len == 0x21 ? 1 : 0)], 0x20);

  return CRYPT0_OK;
}

// init crypt0 library
int crypt0_init()
{
  if (m_crypt0_inited) {
    return CRYPT0_OK;
  }

#ifdef __ZEPHYR__
	int ret;
  static mbedtls_platform_context platform_context = {0};

  ret = mbedtls_platform_setup(&platform_context);
  if (ret != 0) {
	  LOG_ERR("Failed to initialize nrf_cc3xx_mbedcrypto platform: %d", ret);
    return CRYPT0_ERR_INIT_MBETLS;
  }

  ret = psa_crypto_init();
  if (ret != PSA_SUCCESS) {
	  LOG_ERR("Failed to initialize mbedtls: %d", ret);
    return CRYPT0_ERR_INIT_PSA;
  }
#endif 

  m_crypt0_inited = true;

  return CRYPT0_OK;
}

int crypt0_rng(uint8_t * rng, size_t rnglen)
{
  crypt0_init();

#ifdef __ZEPHYR__
  int status = psa_generate_random(rng, rnglen);
  if (status != PSA_SUCCESS) {
    LOG_ERR("Failed nrf_crypto_rng_vector_generate: %d", (int)status);
    return CRYPT0_ERR_RNG;
  }
  return CRYPT0_OK;
#else
  return CRYPT0_ERR_NOT_IMPLEMENTED;
#endif
}

#ifndef __ZEPHYR__
bool crypt0_bin2hex(const uint8_t * bin, size_t binlen, char * hex, size_t hexlen)
{
		if (hexlen < binlen * 2 + 1) {
			return false;
		}

    size_t i, j;
    int b = 0;

    for (i = j = 0; i < binlen; i++) {
        b = bin[i] >> 4;
        hex[j++] = (char) (87 + b + (((b - 10) >> 31) & -39));
        b = bin[i] & 0xf;
        hex[j++] = (char) (87 + b + (((b - 10) >> 31) & -39));
    }
    hex[j] = '\0';
    return true;
}

int crypt0_hexchr2bin(const char hex, char *out)
{
	if (out == NULL)
		return 0;

	if (hex >= '0' && hex <= '9') {
		*out = hex - '0';
	} else if (hex >= 'A' && hex <= 'F') {
		*out = hex - 'A' + 10;
	} else if (hex >= 'a' && hex <= 'f') {
		*out = hex - 'a' + 10;
	} else {
		return 0;
	}

	return 1;
}

bool crypt0_hex2bin(const char * hex, size_t hexlen, uint8_t * bin, size_t binlen)
{
	size_t len;
	char   b1;
	char   b2;
	size_t i;

	if (hex == NULL || *hex == '\0' || bin == NULL) {
    LOG_ERR("hex == NULL || *hex == '\0' || bin == NULL");
		return 0;
  }

	len = hexlen;
	if (len % 2 != 0) {
    LOG_ERR("hexlen %d is not even", (int)len);
		return false;
  }
	len /= 2;

  if (len > binlen) {
    LOG_ERR("binlen %d is not enough to fit %d", (int)binlen, (int)len);
    return false;
  }

	memset(bin, 'A', len);
	for (i=0; i<len; i++) {
		if (!crypt0_hexchr2bin(hex[i*2], &b1) || !crypt0_hexchr2bin(hex[i*2+1], &b2)) {
			return false;
		}
		bin[i] = (b1 << 4) | b2;
	}
	return true;
}

#endif

