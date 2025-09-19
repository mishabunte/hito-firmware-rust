
#include <ctype.h>
#include <stdint.h>
#include <string.h>
#include <stddef.h>
#include <stdbool.h>

#include <crypt0.h>
#include <crypt0_log.h>

#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Wunused-variable"
LOG_MODULE_REGISTER(crypt0_bech32, LOG_LEVEL_DBG);
#pragma GCC diagnostic pop

#include <crypt0_sha.h>
#include <crypt0_ripemd160.h>

#include <segwit_addr.h>

/**
 * By nullius <nullius@nym.zone>
 * PGP:		0xC2E91CD74A4C57A105F6C21B5A00591B2F307E0C
 * Bitcoin:	3NULL3ZCUXr7RDLxXeLPDMZDZYxuaYkCnG
 *		bc1qcash96s5jqppzsp8hy8swkggf7f6agex98an7h
 *
 * Copyright (c) 2017.  All rights reserved.
 *
 * The Antiviral License (AVL) v0.0.1, with added Bitcoin Consensus Clause:
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions
 * are met:
 *
 * 1. Redistributions of the source code must retain the above copyright
 *    and credit notices, this list of conditions, and the following
 *    disclaimer.
 * 2. Redistributions in binary form must reproduce the above copyright
 *    and credit notices, this list of conditions, and the following
 *    disclaimer in the documentation and/or other materials provided
 *    with the distribution.
 * 3. Derivative works hereof MUST NOT be redistributed under any license
 *    containing terms which require derivative works and/or usages to
 *    publish source code, viz. what is commonly known as a "copyleft"
 *    or "viral" license.
 * 4. Derivative works hereof which have any functionality related to
 *    digital money (so-called "cryptocurrency") MUST EITHER adhere to
 *    consensus rules fully compatible with Bitcoin Core, OR use a name
 *    which does not contain the word "Bitcoin".
 *
 * THIS SOFTWARE IS PROVIDED BY THE AUTHOR AND CONTRIBUTORS ``AS IS'' AND
 * ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
 * IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE
 * ARE DISCLAIMED.  IN NO EVENT SHALL THE AUTHOR OR CONTRIBUTORS BE LIABLE
 * FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
 * DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS
 * OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION)
 * HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT
 * LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY
 * OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF
 * SUCH DAMAGE.
 */
static int 
b32enc(unsigned char *b32, int b32buflen, const unsigned char *data, int datalen)
{
	unsigned bits = 0, b32char = 0;
	int b32data_len = 0;

	if (b32buflen < (datalen * 8 / 5 + !!(datalen * 8 % 5)))
		return (-1);

	do {
		b32char <<= 8, b32char |= *data++, bits += 8;

		while (bits >= 5) {
			*b32++ = (b32char >> (bits - 5)), ++b32data_len;
			b32char &= ~(0x1f << (bits -= 5));
		}
	} while (--datalen > 0);

	if (bits > 0)
		*b32 = b32char << (5 - bits), ++b32data_len;

	return (b32data_len);
}

int crypt0_bech32_encode(const uint8_t * data, int datalen, char * buf, int buflen)
{
	return b32enc((unsigned char *)buf, buflen, (const unsigned char *)data, datalen);
}

bool b58enc(char *b58, size_t *b58sz, const void *data, size_t binsz);

int crypt0_base58_encode(const uint8_t * data, int datalen, char * buf, int buflen)
{
	buf[0] = 0;
	size_t len = buflen;
	if (!b58enc(buf, &len, data, datalen)) {
		return -1;
	}
	return strlen(buf);
}

// Returns length of encoded
int crypt0_bech32_witness_v0_encode(const uint8_t * hash, int len, const char * hrp, char * buf, int buflen)
{
	if (len != 32 && len != 20) {
		LOG_ERR("hash len should be 20 or 32");
		return -1;
	}

	if (buflen < 73 + strlen(hrp)) {
		LOG_ERR("buf to small: %d, expected: %d", buflen, 73 + (int)strlen(hrp));
		return -1;
	}
	if (segwit_addr_encode(buf, hrp, 0, hash, len) != 1) {
		return -1;
	}
	return strlen(buf);
}

int crypt0_base58_checksum_hash(const uint8_t * hash, int hashlen, uint8_t prefix, char * buf, int buflen)
{
	uint8_t addr[37];
	addr[0] = prefix;
	if (hashlen != 20 && hashlen != 32) {
		return -1;
	}
	memcpy(&addr[1], hash, hashlen);

	uint8_t sha[32];
	uint8_t sha2[32];
	crypt0_sha256(addr, hashlen + 1, sha, 32);
	crypt0_sha256(sha, 32, sha2, 32);

	memcpy(&addr[hashlen + 1], sha2, 4);

	return crypt0_base58_encode(addr, hashlen + 5, buf, buflen);
}

int crypt0_base58_checksum_address(const uint8_t * pub, int publen, uint8_t prefix, char * buf, int buflen)
{
	uint8_t sha[32];
	uint8_t hash160[20];
	memset(sha, 0, 32);
	memset(hash160, 0, 20);

	crypt0_sha256(pub, publen, sha, 32);

	crypt0_ripemd160(sha, 32, hash160);

	return crypt0_base58_checksum_hash(hash160, 20, prefix, buf, buflen);

}

