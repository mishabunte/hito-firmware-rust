#ifndef __crypt0_byteorder_h_included__
#define __crypt0_byteorder_h_included__

#ifdef __ZEPHYR__

#include <sys/byteorder.h>

#define crypt0_cpu_to_le32(val) sys_cpu_to_le32(val)
#define crypt0_cpu_to_le64(val) sys_cpu_to_le64(val)
#define crypt0_cpu_to_be32(val) sys_cpu_to_be32(val)
#define crypt0_cpu_to_be64(val) sys_cpu_to_be64(val)
#define crypt0_be32_to_cpu(val) sys_be32_to_cpu(val)
#define crypt0_le32_to_cpu(val) sys_le32_to_cpu(val)
#define crypt0_be64_to_cpu(val) sys_be64_to_cpu(val)
#define crypt0_le64_to_cpu(val) sys_le64_to_cpu(val)

#else

#ifdef __APPLE__
#include <libkern/OSByteOrder.h>
#define crypt0_le32_to_cpu(x) OSSwapLittleToHostInt32(x)
#define crypt0_cpu_to_le32(x) OSSwapHostToLittleInt32(x)
#define crypt0_le64_to_cpu(x) OSSwapLittleToHostInt64(x)
#define crypt0_cpu_to_le64(x) OSSwapHostToLittleInt64(x)
#define crypt0_cpu_to_be32(x) OSSwapHostToBigInt32(x)
#define crypt0_cpu_to_be64(x) OSSwapHostToBigInt64(x)
#define crypt0_be32_to_cpu(x) OSSwapBigToHostInt32(x)
#define crypt0_be64_to_cpu(x) OSSwapBigToHostInt64(x)

#else // linux(!) non-zephyr and non-apple

#include <endian.h>
#include <arpa/inet.h>
#include <endian.h>

#define crypt0_le32_to_cpu(val) le32toh(val)
#define crypt0_cpu_to_le32(val) htole32(val)
#define crypt0_cpu_to_le64(val) htole64(val)
#define crypt0_le64_to_cpu(val) le64toh(val)
#define crypt0_cpu_to_be32(val) htonl(val)
#define crypt0_cpu_to_be64(val) htobe64((const uint64_t)val)
#define crypt0_be32_to_cpu(val) ntohl(val)
#define crypt0_be64_to_cpu(val) be64toh((const uint64_t)val)

#endif// __APPLE__
#endif// ZEPHYR
#endif//__crypt0_byteorder_h_included__
