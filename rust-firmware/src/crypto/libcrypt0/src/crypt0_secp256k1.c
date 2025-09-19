#include "crypt0.h"
#include "crypt0_secp256k1.h"
#include <crypt0_log.h>

#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Wunused-variable"
LOG_MODULE_REGISTER(crypt0_secp256k1, LOG_LEVEL_DBG);
#pragma GCC diagnostic pop

// TODO: use non-deprecated functions
#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Wdeprecated-declarations"

#include <string.h>

#include <secp256k1.h>
#include <secp256k1_recovery.h>
#include <secp256k1_ecdh.h>

#ifdef __ZEPHYR__
#include <psa/crypto.h>
#else
#include <openssl/evp.h>
#include <openssl/ec.h>
#include <openssl/core_names.h>
#include <openssl/err.h>
#endif

//-----------------------------------------------------------------------------
int crypt0_secp256k1_public_key(const uint8_t * priv, size_t privlen, 
		uint8_t * pub, size_t publen)
{
	if (privlen != CRYPT0_SECP256_PRIVKEY_BYTES) {
		LOG_ERR("wrong private key len: %d, expected: %d", (int)privlen, CRYPT0_SECP256_PRIVKEY_BYTES);
		return CRYPT0_ERR_PRIVKEY_LEN;
	}
	if (publen != CRYPT0_SECP256_PUBKEY_BYTES) {
		LOG_ERR("wrong public key len: %d, expected: %d", (int)publen, CRYPT0_SECP256_PUBKEY_BYTES);
		return CRYPT0_ERR_PUBKEY_LEN;
	}
  crypt0_init();

#ifdef __ZEPHYR__
	static psa_key_attributes_t key_attributes = PSA_KEY_ATTRIBUTES_INIT;
	static psa_key_handle_t priv_key_handle;
	static psa_status_t status;
	size_t len;

  psa_set_key_usage_flags(&key_attributes, PSA_KEY_USAGE_VERIFY_HASH | PSA_KEY_USAGE_SIGN_HASH);
  psa_set_key_algorithm(&key_attributes, PSA_ALG_ECDSA(PSA_ALG_SHA_256));
  psa_set_key_type(&key_attributes, 
			PSA_KEY_TYPE_ECC_KEY_PAIR(PSA_ECC_FAMILY_SECP_K1));  
	psa_set_key_bits(&key_attributes, 256);


	status = psa_import_key(&key_attributes, priv,
			CRYPT0_SECP256_PRIVKEY_BYTES , &priv_key_handle);

	if (status != PSA_SUCCESS) {
		LOG_ERR("psa_import_key: %d", status);
		return CRYPT0_ERR_IMPORT_KEY;
	}

	memset(pub, 0xff, CRYPT0_SECP256_PUBKEY_BYTES);
	
	status = psa_export_public_key(priv_key_handle, pub, CRYPT0_SECP256_PUBKEY_BYTES, &len);
	if (status != PSA_SUCCESS) {
		LOG_ERR("psa_export_public_key: %d", status);
		return CRYPT0_ERR_EXPORT_KEY;
	}
	
  psa_destroy_key(priv_key_handle);

#else

	EC_KEY   *key      = NULL;
	BIGNUM   *priv_key = NULL;
	EC_POINT *pub_key  = NULL;
	
	//int ret;

	EC_GROUP *secp256k1_group = NULL;

	secp256k1_group = EC_GROUP_new_by_curve_name(NID_secp256k1);

	key = EC_KEY_new_by_curve_name(NID_secp256k1);
	//ret = EC_KEY_generate_key(key);

	EC_KEY_oct2priv(key, priv, privlen);

	priv_key = (BIGNUM *)EC_KEY_get0_private_key(key);

	pub_key = EC_POINT_new(secp256k1_group);
	EC_POINT_mul(secp256k1_group, pub_key, priv_key, NULL, NULL, NULL);
	EC_KEY_set_public_key(key, pub_key);

	EC_POINT_free(pub_key);

	pub_key            = (EC_POINT *)EC_KEY_get0_public_key(key);
	EC_POINT_point2oct(secp256k1_group, 
			pub_key, POINT_CONVERSION_UNCOMPRESSED, pub, publen, NULL);

	EC_KEY_free(key);
	EC_GROUP_free(secp256k1_group);

#endif

  return CRYPT0_OK;
}

//-----------------------------------------------------------------------------
int crypt0_secp256k1_public_key_compressed(const uint8_t * priv, size_t privlen, 
		uint8_t * pub, size_t publen)
{
	if (publen != CRYPT0_SECP256_PUBKEY_COMPRESSED_BYTES) {
		LOG_ERR("wrong public key len: %d, expected: %d", (int)publen, 
				CRYPT0_SECP256_PUBKEY_COMPRESSED_BYTES);
		return CRYPT0_ERR_PUBKEY_LEN;
	}
	uint8_t pub_uncompressed[CRYPT0_SECP256_PUBKEY_BYTES];

	int ret = crypt0_secp256k1_public_key(priv, privlen, pub_uncompressed, sizeof(pub_uncompressed));
	if (ret != CRYPT0_OK) {
		return ret;
	}

	memcpy(pub, pub_uncompressed, CRYPT0_SECP256_PUBKEY_COMPRESSED_BYTES);
	pub[0] = 0x02 + (pub_uncompressed[CRYPT0_SECP256_PUBKEY_BYTES - 1] & 0x01);

  return CRYPT0_OK;
}

//-----------------------------------------------------------------------------
/** sign sha256 hash */
int crypt0_secp256k1_sign_recoverable(const uint8_t * hash, size_t hashlen, 
        const uint8_t * priv, size_t privlen, uint8_t * sig, uint8_t siglen)
{
	if (hashlen != CRYPT0_SHA256_BYTES) {
		return CRYPT0_ERR_HASH_LEN;
	}
	if (privlen != CRYPT0_SECP256_PRIVKEY_BYTES) {
		return CRYPT0_ERR_PRIVKEY_LEN;
	}
	if (siglen != CRYPT0_SECP256_SIG_RECOVERABLE_BYTES) {
		return CRYPT0_ERR_SIG_LEN;
	}
  secp256k1_ecdsa_recoverable_signature sigr;
  secp256k1_context * ctx = 
		secp256k1_context_create(SECP256K1_CONTEXT_SIGN | SECP256K1_CONTEXT_VERIFY);
	if (ctx == NULL) {
		LOG_ERR("err create secp256k1 context");
		return CRYPT0_ERR_INIT_SECP256K1;
	}
  secp256k1_ecdsa_sign_recoverable(ctx, &sigr, hash, priv, NULL, NULL);

  int recid;
  secp256k1_ecdsa_recoverable_signature_serialize_compact(ctx, sig, &recid, &sigr);
	LOG_DBG("recid %d", recid);
	secp256k1_context_destroy(ctx);
	sig[siglen - 1] = sigr.data[64];

	return CRYPT0_OK;
}

int crypt0_secp256k1_verify_signature(const secp256k1_ecdsa_signature *sig, const uint8_t * hash, size_t hashlen)
{
  secp256k1_context * ctx = secp256k1_context_create(SECP256K1_CONTEXT_VERIFY);
  
  //secp256k1_ecdsa_signature_parse_compact(ctx, secp256k1_ecdsa_signature* sig, const unsigned char *input64);

  const secp256k1_pubkey pubkey;
  if (!secp256k1_ecdsa_verify(ctx, sig, hash, &pubkey))
  {
    LOG_DBG("Signature not verified");
    return -1;
  }

  return 1;
} //TODO Complete function

int crypt0_secp256k1_ecdh_secret(uint8_t * priv, int privlen, uint8_t * pub, int publen, uint8_t *secret, int secretlen)
{
  if (secretlen != CRYPT0_SHA256_BYTES) 
  {
		return CRYPT0_ERR_HASH_LEN;
	}
  if (publen != CRYPT0_SECP256_PUBKEY_COMPRESSED_BYTES) 
  {
		return CRYPT0_ERR_SIG_LEN;
	}
  if (privlen != CRYPT0_SHA256_BYTES) 
  {
		return CRYPT0_ERR_SIG_LEN;
	}

  secp256k1_context * ctx = secp256k1_context_create(SECP256K1_CONTEXT_SIGN | SECP256K1_CONTEXT_VERIFY);
  if (ctx == NULL) 
  {
		LOG_ERR("err create secp256k1 context");
		return CRYPT0_ERR_INIT_SECP256K1;
	}
  
  secp256k1_pubkey pubkey;
  if (!secp256k1_ec_pubkey_parse(ctx, &pubkey, pub, publen))
  {
		LOG_ERR("err parsing secp256k1 pubkey");
		return CRYPT0_ERR_IMPORT_KEY;
	}
  if (!secp256k1_ecdh(ctx, secret, &pubkey, priv, NULL, NULL))
  {
    LOG_ERR("secp256k1_ecdh failed");
    return -1;
  }
  secp256k1_context_destroy(ctx);
  
  return CRYPT0_OK;
}

//-----------------------------------------------------------------------------
/** sign sha256 hash, generates compact signature if siglen is 64 */
int crypt0_secp256k1_sign(const uint8_t * hash, size_t hashlen, 
        const uint8_t * priv, size_t privlen, uint8_t * sig, uint8_t siglen)
{
#ifdef __ZEPHYR__

	if (hashlen != CRYPT0_SHA256_BYTES) {
		return CRYPT0_ERR_HASH_LEN;
	}
	if (privlen != CRYPT0_SECP256_PRIVKEY_BYTES) {
		return CRYPT0_ERR_PRIVKEY_LEN;
	}
	if (siglen != CRYPT0_SECP256_SIG_COMPACT_BYTES && siglen != 0x49) {
		return CRYPT0_ERR_SIG_LEN;
	}
  secp256k1_ecdsa_signature ecdsa_sig;
  secp256k1_context * ctx = 
		secp256k1_context_create(SECP256K1_CONTEXT_SIGN | SECP256K1_CONTEXT_VERIFY);
	if (ctx == NULL) {
		LOG_ERR("err create secp256k1 context");
		return CRYPT0_ERR_INIT_SECP256K1;
	}
  secp256k1_ecdsa_sign(ctx, &ecdsa_sig, hash, priv, NULL, NULL);

	if (siglen == 64) {
		secp256k1_ecdsa_signature_serialize_compact(ctx, sig, &ecdsa_sig);
	} else {
		if (siglen < 0x48) {
			siglen = -1;
		} else {
			size_t len = siglen;
			secp256k1_ecdsa_signature_serialize_der(ctx, sig, &len, &ecdsa_sig);
			siglen = len;
		}
	}
	secp256k1_context_destroy(ctx);

#else

	//---------------------------------------------------------------------
	// OpenSSL implementation
	
	EC_GROUP * ec_group = EC_GROUP_new_by_curve_name(NID_secp256k1);
	EC_KEY   * key = EC_KEY_new_by_curve_name(NID_secp256k1);

	EC_KEY_generate_key(key);
	EC_KEY_oct2priv(key, priv, privlen);

	BIGNUM * priv_key = (BIGNUM *)EC_KEY_get0_private_key(key);
	EC_POINT * pub_key = EC_POINT_new(ec_group);
	EC_POINT_mul(ec_group, pub_key, priv_key, NULL, NULL, NULL);
	EC_KEY_set_public_key(key, pub_key);

	ECDSA_SIG *ec_sig = ECDSA_do_sign(hash, hashlen, key);

	// normalize signature to use with bitcoin
	BIGNUM *order = BN_new();
  	BIGNUM *half_order = BN_new();
	BN_CTX *ctx = BN_CTX_new();

  EC_GROUP_get_order(ec_group, order, ctx);
  BN_rshift1(half_order, order); // half_order = order / 2

	// Check if S > N/2 and normalize if necessary
	BIGNUM *r = (BIGNUM*)ECDSA_SIG_get0_r(ec_sig);
	BIGNUM *s = (BIGNUM*)ECDSA_SIG_get0_s(ec_sig);
	if (BN_cmp(s, half_order) > 0) {
		LOG_DBG("--------------- normalize");
		BN_sub(s, order, s); // s = order - s

		// Reassemble the signature
		ec_sig = ECDSA_SIG_new();
		ECDSA_SIG_set0(ec_sig, r, s);
	}

	if (siglen == 64) {

		BN_bn2bin(r, sig);
		BN_bn2bin(s, &sig[32]);

	} else {
		if (siglen < 0x48) {
			siglen = -1;
		} else {
		  siglen = i2d_ECDSA_SIG(ec_sig, &sig);
		}
	}

	BN_free(order);
	BN_free(half_order);
	BN_CTX_free(ctx);
	EC_KEY_free(key);
	EC_GROUP_free(ec_group);
	EC_POINT_free(pub_key);
	ECDSA_SIG_free(ec_sig);

#endif
	return siglen;
}
// eof
#pragma GCC diagnostic pop
