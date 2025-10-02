
#ifndef CRC_16_CCITT_FALSE_H_
#define CRC_16_CCITT_FALSE_H_

#ifdef __cplusplus
extern "C"
{
#endif

#include <stdint.h>
#include <string.h>

#define INIT_VALUE (0xFFFFu)
#define POLY_VALUE (0x1021u)
#define LOOK_UP_TABLE_SIZE (256u)

enum algorithmType {
    ALG_FAST = 0,
    ALG_SLOW = 1,
    AMOUNT_OF_ALG = 2
};

uint16_t calc_crc16_ccitt_false (const uint8_t pData [], uint16_t const length, const uint8_t algType);

#ifdef __cplusplus
}
#endif

#endif /* CRC_16_CCITT_FALSE_H_ */
