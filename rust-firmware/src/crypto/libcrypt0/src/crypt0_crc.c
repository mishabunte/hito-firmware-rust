#include <crypt0_crc.h>

#ifdef __ZEPHYR__
#include <sys/crc.h>
#else
#include <crc16_ccitt.h>
#endif

uint16_t crypt0_crc16_xmodem(const uint8_t *data, size_t len)
{
    uint16_t crc = 0x0000;
    for (size_t i = 0; i < len; ++i) {
        crc ^= ((uint16_t)data[i] << 8);
        for (int j = 0; j < 8; ++j) {
            if (crc & 0x8000)
                crc = (crc << 1) ^ 0x1021;
            else
                crc <<= 1;
        }
    }
    return crc;
}

int crypt0_crc16_ccitt(const void * data, int dataLen)
{
	int crc;
#ifdef __ZEPHYR__
    // crc = crc16_ccitt(0xffff, data, dataLen);
		crc = crc16_ccitt(0x0000, data, dataLen);
#else
		// crc = calc_crc16_ccitt_false(data, dataLen, ALG_SLOW);
		crc = crypt0_crc16_xmodem((const uint8_t *)data, dataLen);
#endif
	return crc;
}
