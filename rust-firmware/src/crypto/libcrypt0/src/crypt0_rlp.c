#include "crypt0_rlp.h"
#include "crypt0.h"
#include "crypt0_byteorder.h"
#include "crypt0_log.h"

#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Wunused-variable"
LOG_MODULE_REGISTER(crypt0_rlp, LOG_LEVEL_DBG);
#pragma GCC diagnostic pop


#include <string.h>

//-----------------------------------------------------------------------------
/**
 * Returns length of rlp-encoded value
 */
int crypt0_rlp_encode_val_len(const uint8_t * buf, size_t buflen)
{
	return crypt0_rlp_encode_val(buf, buflen, NULL, 0);
}

int crypt0_rlp_encode_list(uint8_t * out, size_t outlen, const char * fmt, ...)
{
	va_list ap;
	va_start(ap, fmt);

	int list_total_len = crypt0_rlp_encode_list_len_v(fmt, ap) - 1;
	if (list_total_len < 0) {
		return list_total_len;
	}
	if (outlen < list_total_len) {
		LOG_ERR("outlen < list_total_len: %zu < %d", outlen, list_total_len);
		return CRYPT0_ERR_OUTBUF_LEN;
	}

	if (list_total_len > 0xfffffff9) {
		LOG_ERR("list_total_len > 0xfffffff9: %d", list_total_len);
		return CRYPT0_ERR_OUTBUF_LEN;
	}

	uint8_t * outbuf; // pointer to output buf
	if (list_total_len < 55) {
		out[0] = 0xc0 + list_total_len;
		outbuf = &out[1];
	} else {
		int lenlenbytes;
		if (list_total_len >> 24) {
			lenlenbytes = 4;
		} else if (list_total_len >> 16) {
			lenlenbytes = 3;
		} else if (list_total_len >> 8) {
			lenlenbytes = 2;
		} else {
			lenlenbytes = 1;
		}
		if (outlen < list_total_len + lenlenbytes + 2) {
			return CRYPT0_ERR_OUTBUF_LEN;
		}
		uint32_t lenlen = crypt0_cpu_to_be32(list_total_len - lenlenbytes);
		out[0] = 0xf7 + lenlenbytes;
		lenlen >>= (4 - lenlenbytes) * 8;
		memcpy(&out[1], &lenlen, lenlenbytes);
		outbuf = &out[1 + lenlenbytes];
	}

	va_start(ap, fmt);
	int i = 0;
	while(*fmt != '\0') {
		int ret;
		if (*fmt == 'd') {        // uint32
		  ret = crypt0_rlp_encode_val_uint(va_arg(ap, uint32_t), outbuf, list_total_len + 1 - (outbuf - out));
		} else if (*fmt == 'l') { // uint64
		  ret = crypt0_rlp_encode_val_uint(va_arg(ap, uint64_t), outbuf, list_total_len + 1 - (outbuf - out));
		} else if (*fmt == 'v') { // uint256
		  ret = crypt0_rlp_encode_val_uint256(va_arg(ap, intc_u256*), outbuf, list_total_len + 1 - (outbuf - out));
		} else if (*fmt == 's') { // ----------- string
			const char * s = va_arg(ap, const char *);
		  ret = crypt0_rlp_encode_val((const uint8_t *)s, strlen(s), outbuf, list_total_len + 1 - (outbuf - out));
		} else if (*fmt == 'b') {  // ---------- bytes
		  ret = crypt0_rlp_encode_val(va_arg(ap, const uint8_t *), va_arg(ap, int), outbuf, list_total_len + 1 - (outbuf - out));
		} else if (*fmt == 't') { // ----------- empty list
      *outbuf = 0xc0;
			ret = va_arg(ap, int) + 1;
			//ret = 1;
		} else {
			LOG_ERR("wrong fmt char: %x, expected 'd', 'l', 'v', 'b', 't' or 's'", *fmt);
			return CRYPT0_ERR_FMT;
	  }
		if (ret < 0) {
			LOG_ERR("error encoding arg #%d: %d", i, ret);
			return ret;
		}
		outbuf += ret;
		i++;
		fmt++;
	}
	va_end(ap);
	
	return list_total_len + 1;
}

int crypt0_rlp_decode_list(const uint8_t * buf, size_t buflen, 
		size_t items_count, crypt0_rlp_decode_handler_t handler)
{
	LOG_DBG("buf[0] %x", buf[0]);
	// get overall list length
	if (buflen < 1) {
		LOG_ERR("rlp list less than 1 byte");
		return CRYPT0_ERR_RLP_LIST_HEADER;
	}
	uint32_t payloadlen = 0;
	if (buf[0] < 0xc0) {
		LOG_ERR("rlp list less than 1 byte");
		return CRYPT0_ERR_RLP_LIST_HEADER;
	} else if (buf[0] < 0xf8) {
		payloadlen = buf[0] - 0xc0;
	  if (payloadlen + 1 != buflen) {
		  LOG_ERR("buflen is not enough for payload, expected: %d (%x)", 
					(int)payloadlen, (int)payloadlen);
			return CRYPT0_ERR_RLP_BUFLEN;
		}
	} else if (buf[0] <= 0xf8 + 4) {
		memcpy(&payloadlen, &buf[1], buf[0] - 0xf7);
		payloadlen <<= (4 - (buf[0] - 0xf7)) * 8;
    payloadlen = crypt0_be32_to_cpu(payloadlen);

    if (payloadlen != buflen - (buf[0] - 0xf7 + 1)) {
		  LOG_ERR("buflen is not enough for payload, expected: %d (%x)", 
				(int)payloadlen + buf[0] - 0xf7 + 1, (int)payloadlen + buf[0] - 0xf7 + 1);
			return CRYPT0_ERR_RLP_BUFLEN;
		}
	}

	if (payloadlen == 0) {
		if (items_count != 0) {
			LOG_ERR("payload is zero, items count %zu", items_count);
			return CRYPT0_ERR_FMT;
		}
		return 0;
	}

	int ret;

	const uint8_t * p = &buf[buflen - payloadlen];
	size_t bytes_remaining = payloadlen;
	int i = 0;
	for(; i < (int)items_count; i++) {
		if (*p <= 0x7f) {
			LOG_DBG("*p <= 0x7f");
			if (bytes_remaining < 1) {
				LOG_ERR("not enough buf len for a byte (item #%d)", i);
				return CRYPT0_ERR_FMT;
			}
			ret = (*handler)(p, 1, i);
			if (ret != CRYPT0_OK) {
				return ret;
			}
			bytes_remaining--;
			p++;
		} else if (*p < 0xb7) {
			LOG_DBG("*p < 0xb7");
			if (bytes_remaining < *p - 0x80) {
				LOG_ERR("not enough buf for item len #%d", i);
				return CRYPT0_ERR_FMT;
			}
			ret = (*handler)(p + 1, *p - 0x80, i);
			if (ret != CRYPT0_OK) {
				return ret;
			}
			bytes_remaining -= *p - 0x80 + 1;
			p += *p - 0x80 + 1;
		} else if (*p == 0xc0) { // ------------------------------------ emtpy list
			ret = (*handler)(p, 0, i);
			bytes_remaining--;
			p++;
		} else if (*p > 0xb7 + 4) {
			LOG_ERR("lenlen for item #%d is more than 4 bytes", i);
			return CRYPT0_ERR_FMT;
		} else {
			LOG_DBG("*p > 0xb8 %x %x", *p, *(p + 1));
			uint32_t lenlen; 
			if (bytes_remaining < *p - 0xb8 + 1) {
				LOG_ERR("bytes_remaining for item #%d is not enough: %d", 
						i, (int)bytes_remaining);
				return CRYPT0_ERR_FMT;
			}
			bytes_remaining -= *p - 0xb8 + 1;

			lenlen = 0;
			memcpy(&lenlen, p + 1, *p - 0xb8 + 1);
			lenlen <<= (4 - (*p - 0xb8 + 1)) * 8;
			p += *p - 0xb8 + 2;
			LOG_DBG("lenlen 1: %x", lenlen);
			lenlen = crypt0_be32_to_cpu(lenlen);
			LOG_DBG("lenlen 1: %x", lenlen);
			if (bytes_remaining < lenlen) {
				LOG_ERR("bytes_remaining for item #%d is not enough: %d, expected %d", 
						i, (int)bytes_remaining, (int)lenlen);
				return CRYPT0_ERR_FMT;
			}
			ret = (*handler)(p, lenlen, i);
			if (ret != CRYPT0_OK) {
				return ret;
			}
			bytes_remaining -= lenlen + 1;
			p += lenlen;
		}
	}

	if (bytes_remaining != 0) {
		LOG_ERR("bytes_remaining is %zu, expected 0", bytes_remaining);
		return CRYPT0_ERR_FMT;
	}

	return i;
}

int crypt0_rlp_encode_list_len(const char * fmt, ...)
{
	va_list ap;

	va_start(ap, fmt);
	int len = crypt0_rlp_encode_list_len_v(fmt, ap);
	va_end(ap);

	return len;
}

int crypt0_rlp_encode_list_len_v(const char * fmt, va_list ap)
{
	// calculate total length
	int list_total_len = 0;
	int i = 0;
	while(*fmt != '\0') {
		int ret;
		if (*fmt == 'd') {
		  ret = crypt0_rlp_encode_val_uint_len(va_arg(ap, uint32_t));
			LOG_DBG("ret d %d", ret);
		} else if (*fmt == 'l') {
		  ret = crypt0_rlp_encode_val_uint_len(va_arg(ap, uint64_t));
			LOG_DBG("ret l %d", ret);
		} else if (*fmt == 'v') {
		  ret = crypt0_rlp_encode_val_uint256_len(va_arg(ap, intc_u256*));
			LOG_DBG("ret v %d", ret);
		} else if (*fmt == 's') {
			const char * s = va_arg(ap, const char *);
		  ret = crypt0_rlp_encode_val_len((const uint8_t *)s, strlen(s));
			LOG_DBG("ret s %d", ret);
		} else if (*fmt == 'b') {
		  ret = crypt0_rlp_encode_val_len(va_arg(ap, const uint8_t *), va_arg(ap, int));
			LOG_DBG("ret b %d", ret);
		} else if (*fmt == 't') { // ----------- empty list
			ret = va_arg(ap, int) + 1;
			//ret = 1;
			LOG_DBG("ret t %d", ret);
		} else {
			LOG_ERR("wrong fmt char: %x, expected 'd', 'l', 'b', 't' or 's'", *fmt);
			return CRYPT0_ERR_FMT;
	  }

		if (ret < 0) {
			LOG_ERR("error encode_len for arg #%d: %d", i, ret);
			return ret;
		}
		list_total_len += ret;
		i++;
		fmt++;
	}

	if (list_total_len < 55) {
		return list_total_len + 1;
	} 

	if (list_total_len > 0xfffffff9) {
		return CRYPT0_ERR_OUTBUF_LEN;
	}

	int lenlenbytes;
	if (list_total_len >> 24) {
		lenlenbytes = 4;
	} else if (list_total_len >> 16) {
		lenlenbytes = 3;
	} else if (list_total_len >> 8) {
		lenlenbytes = 2;
	} else {
		lenlenbytes = 1;
	}
	return list_total_len + 1 + lenlenbytes;
}


//-----------------------------------------------------------------------------
/**
 * RLP encode value and return encoded length of it
 */
int crypt0_rlp_encode_val(const uint8_t * buf, size_t buflen, 
		uint8_t * out, size_t outlen)
{
	if (buflen == 1 && buf[0] <= 0x7f) {

		// For a single byte whose value is in the [0x00, 0x7f] (decimal [0, 127]) 
		//   range, that byte is its own RLP encoding.
		if (outlen == 0 && out != NULL) {
      return CRYPT0_ERR_OUTBUF_LEN;
		}
		if (out != NULL) {
			out[0] = buf[0];
		}
		return 1;

	} else if (buflen == 0 || buflen < 55) {

		// Otherwise, if a string is 0-55 bytes long, the RLP encoding consists of a 
		//   single byte with value 0x80 (dec. 128) plus the length of the string 
		//   followed by the string. The range of the first byte is thus [0x80, 0xb7] 
		//   (dec. [128, 183]).
		if (outlen < buflen + 1 && out != NULL) {
			return CRYPT0_ERR_OUTBUF_LEN;
		}
		if (out != NULL) {
			out[0] = 0x80 + buflen;
			memcpy(&out[1], buf, buflen);
		}
		return buflen + 1;

	} else {
		// If a string is more than 55 bytes long, the RLP encoding consists of a 
		//   single byte with value 0xb7 (dec. 183) plus the length in bytes of 
		//   the length of the string in binary form, followed by the length of 
		//   the string, followed by the string.
		if (buflen > 0xfffffff9) {
			return CRYPT0_ERR_OUTBUF_LEN;
		}
		int lenlenbytes;
		if (buflen >> 24) {
			lenlenbytes = 4;
		} else if (buflen >> 16) {
			lenlenbytes = 3;
		} else if (buflen >> 8) {
			lenlenbytes = 2;
		} else {
			lenlenbytes = 1;
		}
		// TODO: add support for 64 bit length
		if (outlen < buflen + lenlenbytes + 1 && out != NULL) {
			return CRYPT0_ERR_OUTBUF_LEN;
		}
		if (out != NULL) {
			uint32_t lenlen = crypt0_cpu_to_be32(buflen);
			out[0] = 0xb7 + lenlenbytes;
			lenlen >>= (4 - lenlenbytes) * 8;
			memcpy(&out[1], &lenlen, lenlenbytes);
			memcpy(&out[lenlenbytes + 1], buf, buflen);
		}

		return lenlenbytes + buflen + 1;
	}
}

int crypt0_rlp_encode_val_uint_len(uint64_t val)
{
	return crypt0_rlp_encode_val_uint(val, NULL, 0);
}

//
int crypt0_rlp_encode_val_uint(uint64_t val, uint8_t * out, size_t outlen) 
{
  uint64_t valbe = crypt0_cpu_to_be64(val);

	size_t lenlen = 8;
	while(lenlen > 0 && (valbe & 0xff) == 0) {
		valbe >>= 8;
		lenlen--;
	}

  valbe = crypt0_cpu_to_be64(val);
	const uint8_t * valbe_buf = (void *)&valbe;

	return crypt0_rlp_encode_val(&valbe_buf[8 - lenlen], lenlen, out, outlen);
}

int crypt0_rlp_encode_val_uint256_len(intc_u256 *val)
{
	return crypt0_rlp_encode_val_uint256(val, NULL, 0);
}

int crypt0_rlp_encode_val_uint256(intc_u256 *val, uint8_t * out, size_t outlen) 
{
  intc_u256_string val_str = intc_u256_str(*val, 16, 0, 0);
  size_t buflen = val_str.size / 2 + val_str.size % 2;
  uint8_t buf[buflen];
  crypt0_hex2bin(val_str.data, val_str.size, buf, buflen);

	if (buflen == 1 && buf[0] == 0x0) {
		buflen = 0;
	}

	return crypt0_rlp_encode_val(buf, buflen, out, outlen);
}

// eof
