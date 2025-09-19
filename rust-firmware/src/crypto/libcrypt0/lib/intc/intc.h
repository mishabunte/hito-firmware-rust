#if !defined(INTC_H)
#define INTC_H
// NOTE: Overview ==================================================================================
//
// calccrypto's uint128/256 C++ library converted into a single header file
// C/C++ library with some additional C-isms/cosmetic configurations. Some of
// the over-arching motivations of this library.
//
// - Single header, drop in - no build configuration required.
// - Minimal dependencies (only depends on stdbool.h when compiling in C).
// - Zero allocations (in other libs typically when converting to strings).
// - Ability to rename the API via the pre-processor for tighter integration
//   into C code bases (See Configuration > INTC_API_PREFIX).
// - C++ features are optional and can be opt out via the pre-processor, i.e.
//   operator overloading and constructors.
//   (See Configuration > INTC_NO_CPP_FEATURES).
//
// NOTE: License ===================================================================================
//
// MIT License
// Copyright (c) 2021 github.com/doy-lee
// Copyright (c) 2013 - 2017 Jason Lee @ calccrypto at gmail.com
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.
//
// NOTE: Configuration
// -----------------------------------------------------------------------------
// #define INTC_IMPLEMENTATION
//     Define this in one and only one C++ file to enable the implementation
//     code of the header file
//
// #define INTC_ASSERT
//     Define this macro to override the assert implementation in this file to
//     a custom macro. If not overridden, when NDEBUG is defined- the assert is
//     compiled out, otherwise a default assert macro is provided.
//
// #define INTC_NO_TYPEDEFS
//     Define this macro to disable 'typedef struct intc_u128 intc_u128;' to
//     force having to specify 'struct intc_u128' when declaring the integer
//     types. Only relevant when compiling with a C compiler, otherwise ignored.
//
// #define INTC_NO_U256
//     Define this macro to disable the uint256 code in the library.
//
// #define INTC_STATIC_API
//     Define this macro to prefix all functions with the static qualifier
//
// #define INTC_NO_CPP_FEATURES
//     Define this macro to disable the C++ constructors and operator
//     overloading in the library. When this library is compiled as a C library,
//     this macro is ignored. When compiling the library in C++ with this macro,
//     the functions in the library will be exported with 'extern "C"' defined
//     (i.e. prevent name-mangling).
//
// #define INTC_API_PREFIX
//     Define this macro to rename the 'intc_u' part of the functions in this
//     library to your desired name via the pre-processor, i.e.
//
//     #define INTC_API_PREFIX(expr) MyNamespace_u##expr
//     intc_u128 value = MyNamespace_u128_add(INTC_U128(128, 128), INTC_U128(128, 128));
//
//     or
//
//     #define INTC_API_PREFIX(expr) uint##expr
//     intc_u128 value = uint128_add(INTC_U128(128, 128), INTC_U128(128, 128));
//
// NOTE: Examples ==================================================================================
/*
    #define INTC_IMPLEMENTATION
    #include "intc.h"
    #include <stdio.h>

    int main(int, char**)
    {
    #if defined(__cplusplus)
        intc_u256 value = 32;
        value += 32;
        if (value == 64)
            value *= 1'000'000'000'000;
        intc_u256_string string = intc_u256_readable_int_str(value);
        printf("%.*s\n", string.size, string.data); // 64,000,000,000,000
    #else
        intc_u256 value = INTC_U64_TO_U256(32);
        value = intc_u256_add_u64(value, 32);
        if (intc_u256_eq_u64(value, 64))
            value = intc_u256_mul(value, INTC_U64_TO_U256(1'000'000'000'000));
        intc_u256_string string = intc_u256_readable_int_str(value);
        printf("%.*s\n", string.size, string.data); // 64,000,000,000,000
    #endif
        return 0;
    }

 */

// NOTE: INTC Macros ===============================================================================
#if !defined(INTC_ASSERT)
    #if defined(NDEBUG)
        #define INTC_ASSERT(expr)
    #else
        #define INTC_ASSERT(expr)             \
            do {                              \
                if (!(expr)) {                \
                    (*(volatile int *)0) = 0; \
                }                             \
            } while (0)
    #endif
#endif

#if !defined(INTC_API_PREFIX)
    #define INTC_API_PREFIX(expr) intc_u##expr
#endif

#if !defined(INTC_API)
    #if defined(INTC_STATIC_API)
        #define INTC_API static
    #else
        #define INTC_API
    #endif
#endif

#if defined(__cplusplus)
    #if defined(INTC_NO_CPP_FEATURES)
        #define INTC_BEGIN_EXTERN_C extern "C" {
        #define INTC_END_EXTERN_C }
    #else
        #define INTC_BEGIN_EXTERN_C
        #define INTC_END_EXTERN_C
    #endif
    #define INTC_ZERO_INIT {}
#else
    #if !defined(INTC_NO_CPP_FEATURES)
        #define INTC_NO_CPP_FEATURES
    #endif
    #define INTC_BEGIN_EXTERN_C
    #define INTC_END_EXTERN_C
    #define INTC_ZERO_INIT {0}
    #include <stdbool.h>
#endif

// NOTE: Typedefs ==================================================================================
typedef unsigned char  intc_u8;
typedef unsigned short intc_u16;
typedef unsigned int   intc_u32;
typedef unsigned int   intc_uint;
#ifdef _MSC_VER
    typedef unsigned __int64 intc_u64;
#else
    typedef unsigned long long intc_u64;
#endif

// NOTE: 128 Bit Unsigned Integer ==================================================================
struct intc_u128
{
    #if !defined(INTC_NO_CPP_FEATURES)
    intc_u128() = default;
    intc_u128(intc_u64 lo) : lo(lo), hi(0) {}
    intc_u128(intc_u64 lo, intc_u64 hi) : lo(lo), hi(hi) {}
    #endif // INTC_NO_CPP_FEATURES

    intc_u64 lo, hi;
};

struct intc_u128_divmod_result
{
    struct intc_u128 quot;
    struct intc_u128 rem;
};

struct intc_u128_string
{
    // NOTE: Max value 340,282,366,920,938,463,463,374,607,431,768,211,455
    char  data[63 + 1];
    int   size;
};

struct intc_u128_init_result
{
    bool             success;
    struct intc_u128 value;
};

// NOTE: Constructors ==============================================================================
#if defined (__cplusplus)
    #define INTC_U128(lo, hi) intc_u128{(lo), (hi)}
    #define INTC_U128_MIN intc_u128{0, 0}
    #define INTC_U128_MAX intc_u128{(intc_u64)-1, (intc_u64)-1}
    #define INTC_U128_ZERO intc_u128{0, 0}
    #define INTC_U64_TO_U128(u64) intc_u128{(u64), 0}
#else
    #define INTC_U128(lo, hi) (struct intc_u128){(lo), (hi)}
    #define INTC_U128_MIN (struct intc_u128){0, 0}
    #define INTC_U128_MAX (struct intc_u128){(intc_u64)-1, (intc_u64)-1}
    #define INTC_U128_ZERO (struct intc_u128){0, 0}
    #define INTC_U64_TO_U128(u64) (struct intc_u128){(u64), 0}

    #if !defined(INTC_NO_TYPEDEFS)
        typedef struct intc_u128 intc_u128;
        typedef struct intc_u128_divmod_result intc_u128_divmod_result;
        typedef struct intc_u128_string intc_u128_string;
    #endif
#endif

INTC_BEGIN_EXTERN_C
// NOTE: U128 Converters ===========================================================================
// Reminder: If INTC_API_PREFIX is not defined, example API looks like: intc_u128_init_hex_cstring(...)
//
// Construct a 128 unsigned integer from a string. This function supports
// hexadecimal strings with and without the 0x prefix i.e. "0xafc8a" or "afc8a"
// or "0xAFC8A" or "xafc8a" .
//
// The separator character given will be permitted and skipped in the string if
// encountered, e.g. ','. If no separator is permitted, pass 0 as the separator.
INTC_API struct intc_u128_init_result   INTC_API_PREFIX(128_init_hex_cstring)(const char *string, size_t size, char separator);

// Construct a 128 unsigned integer from a base 10 number string.
//
// The separator character given will be permitted and skipped in the string if
// encountered, e.g. ','. If no separator is permitted, pass 0 as the separator.
INTC_API struct intc_u128_init_result   INTC_API_PREFIX(128_init_cstring)(char const *string, size_t size, char separator);

// Interpret the 128 bit integer as a lower bit-type by using the lo bits of the
// integer and truncating where necessary.
INTC_API bool                           INTC_API_PREFIX(128_as_bool)(struct intc_u128 in);
INTC_API intc_u8                        INTC_API_PREFIX(128_as_u8)(struct intc_u128 in);
INTC_API intc_u16                       INTC_API_PREFIX(128_as_u16)(struct intc_u128 in);
INTC_API intc_u32                       INTC_API_PREFIX(128_as_u32)(struct intc_u128 in);
INTC_API intc_u64                       INTC_API_PREFIX(128_as_u64)(struct intc_u128 in);

// NOTE: U128 Bitwise ==============================================================================
// Reminder: If INTC_API_PREFIX is not defined, example API looks like: intc_u128_and(...)
INTC_API struct intc_u128               INTC_API_PREFIX(128_and)(struct intc_u128 lhs, struct intc_u128 rhs);
INTC_API struct intc_u128               INTC_API_PREFIX(128_or)(struct intc_u128 lhs, struct intc_u128 rhs);
INTC_API struct intc_u128               INTC_API_PREFIX(128_xor)(struct intc_u128 lhs, struct intc_u128 rhs);
INTC_API struct intc_u128               INTC_API_PREFIX(128_negate)(struct intc_u128 lhs);
INTC_API struct intc_u128               INTC_API_PREFIX(128_lshift)(struct intc_u128 lhs, unsigned rhs);
INTC_API struct intc_u128               INTC_API_PREFIX(128_rshift)(struct intc_u128 lhs, unsigned rhs);

// NOTE: U128 Equality =============================================================================
// If INTC_API_PREFIX not defined then, for example: intc_u128_eq(...)
INTC_API bool                           INTC_API_PREFIX(128_eq)(struct intc_u128 lhs, struct intc_u128 rhs);
INTC_API bool                           INTC_API_PREFIX(128_neq)(struct intc_u128 lhs, struct intc_u128 rhs);

// NOTE: U128 Equality U64 Helpers =================================================================
INTC_API bool                           INTC_API_PREFIX(128_eq_u64)(struct intc_u128 lhs, intc_u64 rhs);
INTC_API bool                           INTC_API_PREFIX(128_neq_u64)(struct intc_u128 lhs, intc_u64 rhs);

// NOTE: U128 Relational ===========================================================================
// Reminder: If INTC_API_PREFIX is not defined, example API looks like: intc_u128_gt(...)
INTC_API bool                           INTC_API_PREFIX(128_gt)(struct intc_u128 lhs, struct intc_u128 rhs);
INTC_API bool                           INTC_API_PREFIX(128_lt)(struct intc_u128 lhs, struct intc_u128 rhs);
INTC_API bool                           INTC_API_PREFIX(128_gt_eq)(struct intc_u128 lhs, struct intc_u128 rhs);
INTC_API bool                           INTC_API_PREFIX(128_lt_eq)(struct intc_u128 lhs, struct intc_u128 rhs);

// NOTE: U128 Relational U64 Helpers ===============================================================
// Reminder: If INTC_API_PREFIX is not defined, example API looks like: intc_u128_gt_u64(...)
INTC_API bool                           INTC_API_PREFIX(128_gt_u64)(struct intc_u128 lhs, intc_u64 rhs);
INTC_API bool                           INTC_API_PREFIX(128_lt_u64)(struct intc_u128 lhs, intc_u64 rhs);
INTC_API bool                           INTC_API_PREFIX(128_gt_eq_u64)(struct intc_u128 lhs, intc_u64 rhs);
INTC_API bool                           INTC_API_PREFIX(128_lt_eq_u64)(struct intc_u128 lhs, intc_u64 rhs);

// NOTE: U128 Arithmetic ===========================================================================
// Reminder: If INTC_API_PREFIX is not defined, example API looks like: intc_u128_add(...)
INTC_API struct intc_u128               INTC_API_PREFIX(128_add)(struct intc_u128 lhs, struct intc_u128 rhs);
INTC_API struct intc_u128               INTC_API_PREFIX(128_sub)(struct intc_u128 lhs, struct intc_u128 rhs);
INTC_API struct intc_u128               INTC_API_PREFIX(128_mul)(struct intc_u128 lhs, struct intc_u128 rhs);
INTC_API struct intc_u128_divmod_result INTC_API_PREFIX(128_divmod)(struct intc_u128 lhs, struct intc_u128 rhs);
INTC_API struct intc_u128               INTC_API_PREFIX(128_div)(struct intc_u128 lhs, struct intc_u128 rhs);
INTC_API struct intc_u128               INTC_API_PREFIX(128_mod)(struct intc_u128 lhs, struct intc_u128 rhs);

// NOTE: U128 Arithmetic U64 Helpers ===============================================================
// Reminder: If INTC_API_PREFIX is not defined, example API looks like: intc_u128_add_u64(...)
INTC_API struct intc_u128               INTC_API_PREFIX(128_add_u64)(struct intc_u128 lhs, intc_u64 rhs);
INTC_API struct intc_u128               INTC_API_PREFIX(128_sub_u64)(struct intc_u128 lhs, intc_u64 rhs);
INTC_API struct intc_u128               INTC_API_PREFIX(128_mul_u64)(struct intc_u128 lhs, intc_u64 rhs);
INTC_API struct intc_u128_divmod_result INTC_API_PREFIX(128_divmod_u64)(struct intc_u128 lhs, intc_u64 rhs);
INTC_API struct intc_u128               INTC_API_PREFIX(128_div_u64)(struct intc_u128 lhs, intc_u64 rhs);
INTC_API struct intc_u128               INTC_API_PREFIX(128_mod_u64)(struct intc_u128 lhs, intc_u64 rhs);

// NOTE: U128 Misc =================================================================================
// Reminder: If INTC_API_PREFIX is not defined, example API looks like: intc_u128_clz(...)
INTC_API int                            INTC_API_PREFIX(128_clz)(struct intc_u128 in); // CLZ (Count leading zeros)

// NOTE: U128 Printing =============================================================================
// Reminder: If INTC_API_PREFIX is not defined, example API looks like: intc_u128_str(...)

// TODO(dqn): Add API for letting user pass in a buffer and we write into it.

// Convert the 128 bit unsigned integer into a string.
// base: The number system base to print the string as where base must be (2 < base < 36).
// seperate_every_n_chars: When > 0, the string will be separated by the 'seperate_ch' for every N
//                         characters specified in this parameter.
// seperate_ch: The character to use to separate the tring. If
//              'seperate_every_n_chars <= 0' then this parameter is ignored.
// return: The integer converted to a string, on failure, an empty string is
// returned. The string is always null-terminated. The string's size is not
// inclusive of the null-terminator.
INTC_API struct intc_u128_string        INTC_API_PREFIX(128_str)(struct intc_u128 in, unsigned base, size_t separate_every_n_chars, char separate_ch);

// Helper function that calls intc_u128_str with base 10 (i.e. human readable).
INTC_API struct intc_u128_string        INTC_API_PREFIX(128_int_str)(struct intc_u128 in, size_t separate_every_n_chars, char separate_ch);

// Helper function that calls intc_u128_str with base 10 (i.e. human readable),
// seperated by the thousands with a comma, i.e. 1,000.
INTC_API struct intc_u128_string        INTC_API_PREFIX(128_readable_int_str)(struct intc_u128 in);

// Helper function that calls intc_u128_str with base 16 (i.e. hex)
INTC_API struct intc_u128_string        INTC_API_PREFIX(128_hex_str)(struct intc_u128 in, size_t separate_every_n_chars, char separate_ch);

// Helper function that calls intc_u128_str with base 16 (i.e. hex), seperated
// every 4 hex characters with '_', i.e. ffff_ffff
INTC_API struct intc_u128_string        INTC_API_PREFIX(128_readable_hex_str)(struct intc_u128 in);
INTC_END_EXTERN_C

#if !defined(INTC_NO_CPP_FEATURES)
// NOTE: U128 CPP Bitwise ==========================================================================
INTC_API intc_u128 operator&(intc_u128 lhs, intc_u128 rhs);
INTC_API intc_u128 operator|(intc_u128 lhs, intc_u128 rhs);
INTC_API intc_u128 operator^(intc_u128 lhs, intc_u128 rhs);
INTC_API intc_u128 operator~(intc_u128 lhs);
INTC_API intc_u128 operator<<(intc_u128 lhs, unsigned rhs);
INTC_API intc_u128 operator>>(intc_u128 lhs, unsigned rhs);

// NOTE: U128 CPP Equality =========================================================================
INTC_API bool operator==(intc_u128 lhs, intc_u128 rhs);
INTC_API bool operator!=(intc_u128 lhs, intc_u128 rhs);

// NOTE: U128 CPP Relational =======================================================================
INTC_API bool operator>(intc_u128 lhs, intc_u128 rhs);
INTC_API bool operator<(intc_u128 lhs, intc_u128 rhs);
INTC_API bool operator>=(intc_u128 lhs, intc_u128 rhs);
INTC_API bool operator<=(intc_u128 lhs, intc_u128 rhs);

// NOTE: U128 CPP Arithmetic =======================================================================
INTC_API intc_u128 operator+(intc_u128 lhs, intc_u128 rhs);
INTC_API intc_u128 operator-(intc_u128 lhs, intc_u128 rhs);
INTC_API intc_u128 operator*(intc_u128 lhs, intc_u128 rhs);
INTC_API intc_u128 operator/(intc_u128 lhs, intc_u128 rhs);
INTC_API intc_u128 operator%(intc_u128 lhs, intc_u128 rhs);

// NOTE: U128 CPP Other ============================================================================
INTC_API intc_u128 &operator&=(intc_u128 &lhs, intc_u128 rhs);
INTC_API intc_u128 &operator|=(intc_u128 &lhs, intc_u128 rhs);
INTC_API intc_u128 &operator^=(intc_u128 &lhs, intc_u128 rhs);
INTC_API intc_u128 &operator<<=(intc_u128 &lhs, unsigned rhs);
INTC_API intc_u128 &operator>>=(intc_u128 &lhs, unsigned rhs);

INTC_API intc_u128 &operator+=(intc_u128 &lhs, intc_u128 rhs);
INTC_API intc_u128 &operator-=(intc_u128 &lhs, intc_u128 rhs);
INTC_API intc_u128 &operator*=(intc_u128 &lhs, intc_u128 rhs);
INTC_API intc_u128 &operator/=(intc_u128 &lhs, intc_u128 rhs);
INTC_API intc_u128 &operator%=(intc_u128 &lhs, intc_u128 rhs);

INTC_API intc_u128 &operator++(intc_u128 &lhs);
INTC_API intc_u128 &operator--(intc_u128 &lhs);
INTC_API intc_u128  operator++(intc_u128 &lhs, int);
INTC_API intc_u128  operator--(intc_u128 &lhs, int);
#endif // !defined(INTC_NO_CPP_FEATURES)

// NOTE: 256 Bit Unsigned Integer ==================================================================
#if !defined(INTC_NO_U256)
struct intc_u256
{
    #if !defined(INTC_NO_CPP_FEATURES)
    intc_u256() = default;
    intc_u256(intc_u64 lo_u64) { *this = {}; this->lo.lo = lo_u64; }
    intc_u256(intc_u128 lo)    { *this = {}; this->lo    = lo; }
    intc_u256(intc_u128 lo, intc_u128 hi) : lo(lo), hi(hi) {}
    #endif // INTC_NO_CPP_FEATURES

    struct intc_u128 lo, hi;
};

struct intc_u256_divmod_result
{
    struct intc_u256 quot;
    struct intc_u256 rem;
};

struct intc_u256_string
{
    // NOTE: Max value 115,792,089,237,316,195,423,570,985,008,687,907,853,269,984,665,640,564,039,457,584,007,913,129,639,935
    char  data[512 + 1];
    int   size;
};

struct intc_u256_init_result
{
    bool             success;
    struct intc_u256 value;
};

// NOTE: U256 Constructors =========================================================================
#if defined(__cplusplus)
    #define INTC_U256(lo, hi) intc_u256{lo, hi}
    #define INTC_U256_MIN intc_u256{INTC_U128_MIN, INTC_U128_MIN}
    #define INTC_U256_MAX intc_u256{INTC_U128_MAX, INTC_U128_MAX}
    #define INTC_U256_ZERO intc_u256{INTC_U128_ZERO, INTC_U128_ZERO}

    #define INTC_U64_TO_U256(u64) intc_u256{INTC_U64_TO_U128(u64), INTC_U128_ZERO}
    #define INTC_U128_TO_U256(u128) intc_u256{(u128), 0}
#else
    #define INTC_U256(lo, hi) (struct intc_u256){(lo), (hi)}
    #define INTC_U256_MIN (struct intc_u256){INTC_U128_MIN, INTC_U128_MIN}
    #define INTC_U256_MAX (struct intc_u256){INTC_U128_MAX, INTC_U128_MAX}
    #define INTC_U256_ZERO (struct intc_u256){INTC_U128_ZERO, INTC_U128_ZERO}

    #define INTC_U64_TO_U256(u64) (struct intc_u256){INTC_U64_TO_U128(u64), INTC_U128_ZERO}
    #define INTC_U128_TO_U256(u128) (struct intc_u256){(u128), 0}

    #if !defined(INTC_NO_TYPEDEFS)
        typedef struct intc_u256 intc_u256;
        typedef struct intc_u256_divmod_result intc_u256_divmod_result;
        typedef struct intc_u256_string intc_u256_string;
    #endif
#endif

INTC_BEGIN_EXTERN_C
// NOTE: U256 Converters ===========================================================================
// Reminder: If INTC_API_PREFIX is not defined, example API looks like: intc_u256_init_hex_cstring(...)
//
// Construct a 128 unsigned integer from a string. This function supports
// hexadecimal strings with and without the 0x prefix i.e. "0xafc8a" or "afc8a"
// or "0xAFC8A" or "xafc8a".
//
// The separator character given will be permitted and skipped in the string if
// encountered, e.g. ','. If no separator is permitted, pass 0 as the separator.
INTC_API struct intc_u256_init_result   INTC_API_PREFIX(256_init_hex_cstring)(char const *string, size_t size, char separator);

// Construct a 256 unsigned integer from a base 10 number string.
//
// The separator character given will be permitted and skipped in the string if
// encountered, e.g. ','. If no separator is permitted, pass 0 as the separator.
INTC_API struct intc_u256_init_result   INTC_API_PREFIX(256_init_cstring)(char const *string, size_t size, char separator);

// Interpret the 256 bit integer as a lower bit-type by using the lo bits of the
// integer and truncating where necessary.
INTC_API bool                           INTC_API_PREFIX(256_as_bool)(struct intc_u256 in);
INTC_API intc_u8                        INTC_API_PREFIX(256_as_u8)(struct intc_u256 in);
INTC_API intc_u16                       INTC_API_PREFIX(256_as_u16)(struct intc_u256 in);
INTC_API intc_u32                       INTC_API_PREFIX(256_as_u32)(struct intc_u256 in);
INTC_API intc_u64                       INTC_API_PREFIX(256_as_u64)(struct intc_u256 in);
INTC_API struct intc_u128               INTC_API_PREFIX(256_as_u128)(struct intc_u256 in);

// NOTE: U256 Bitwise ==============================================================================
// Reminder: If INTC_API_PREFIX is not defined, example API looks like: intc_u256_and(...)
INTC_API struct intc_u256               INTC_API_PREFIX(256_and)(struct intc_u256 lhs, struct intc_u256 rhs);
INTC_API struct intc_u256               INTC_API_PREFIX(256_or)(struct intc_u256 lhs, struct intc_u256 rhs);
INTC_API struct intc_u256               INTC_API_PREFIX(256_xor)(struct intc_u256 lhs, struct intc_u256 rhs);

INTC_API struct intc_u256               INTC_API_PREFIX(256_and_u128)(struct intc_u256 lhs, struct intc_u128 rhs);
INTC_API struct intc_u256               INTC_API_PREFIX(256_or_u128)(struct intc_u256 lhs, struct intc_u128 rhs);
INTC_API struct intc_u256               INTC_API_PREFIX(256_xor_u128)(struct intc_u256 lhs, struct intc_u128 rhs);

INTC_API struct intc_u256               INTC_API_PREFIX(256_negate)(struct intc_u256 lhs);
INTC_API struct intc_u256               INTC_API_PREFIX(256_lshift)(struct intc_u256 lhs, unsigned shift);
INTC_API struct intc_u256               INTC_API_PREFIX(256_rshift)(struct intc_u256 lhs, unsigned shift);

// NOTE: U256 Equality =============================================================================
// Reminder: If INTC_API_PREFIX is not defined, example API looks like: intc_u256_eq(...)
INTC_API bool                           INTC_API_PREFIX(256_eq)(struct intc_u256 lhs, struct intc_u256 rhs);
INTC_API bool                           INTC_API_PREFIX(256_neq)(struct intc_u256 lhs, struct intc_u256 rhs);

// NOTE: U256 Equality U64 Helpers =================================================================
// Reminder: If INTC_API_PREFIX is not defined, example API looks like: intc_u256_eq_u64(...)
INTC_API bool                           INTC_API_PREFIX(256_eq_u64)(struct intc_u256 lhs, intc_u64 rhs);
INTC_API bool                           INTC_API_PREFIX(256_neq_u64)(struct intc_u256 lhs, intc_u64 rhs);

// NOTE: U256 Relational ===========================================================================
// Reminder: If INTC_API_PREFIX is not defined, example API looks like: intc_u256_gt(...)
INTC_API bool                           INTC_API_PREFIX(256_gt)(struct intc_u256 lhs, struct intc_u256 rhs);
INTC_API bool                           INTC_API_PREFIX(256_lt)(struct intc_u256 lhs, struct intc_u256 rhs);
INTC_API bool                           INTC_API_PREFIX(256_gt_eq)(struct intc_u256 lhs, struct intc_u256 rhs);
INTC_API bool                           INTC_API_PREFIX(256_lt_eq)(struct intc_u256 lhs, struct intc_u256 rhs);

// NOTE: U256 Relational U64 Helpers ===============================================================
// Reminder: If INTC_API_PREFIX is not defined, example API looks like: intc_u256_gt_u64(...)
INTC_API bool                           INTC_API_PREFIX(256_gt_u64)(struct intc_u256 lhs, intc_u64 rhs);
INTC_API bool                           INTC_API_PREFIX(256_lt_u64)(struct intc_u256 lhs, intc_u64 rhs);
INTC_API bool                           INTC_API_PREFIX(256_gt_eq_u64)(struct intc_u256 lhs, intc_u64 rhs);
INTC_API bool                           INTC_API_PREFIX(256_lt_eq_u64)(struct intc_u256 lhs, intc_u64 rhs);

// NOTE: U256 Arithmetic ===========================================================================
// Reminder: If INTC_API_PREFIX is not defined, example API looks like: intc_u256_add(...)
INTC_API struct intc_u256               INTC_API_PREFIX(256_add)(struct intc_u256 lhs, struct intc_u256 rhs);
INTC_API struct intc_u256               INTC_API_PREFIX(256_sub)(struct intc_u256 lhs, struct intc_u256 rhs);
INTC_API struct intc_u256               INTC_API_PREFIX(256_mul)(struct intc_u256 lhs, struct intc_u256 rhs);
INTC_API struct intc_u256_divmod_result INTC_API_PREFIX(256_divmod)(struct intc_u256 lhs, struct intc_u256 rhs);
INTC_API struct intc_u256               INTC_API_PREFIX(256_div)(struct intc_u256 lhs, struct intc_u256 rhs);
INTC_API struct intc_u256               INTC_API_PREFIX(256_mod)(struct intc_u256 lhs, struct intc_u256 rhs);

INTC_API struct intc_u256               INTC_API_PREFIX(256_add_u64)(struct intc_u256 lhs, intc_u64 rhs);
INTC_API struct intc_u256               INTC_API_PREFIX(256_sub_u64)(struct intc_u256 lhs, intc_u64 rhs);
INTC_API struct intc_u256               INTC_API_PREFIX(256_mul_u64)(struct intc_u256 lhs, intc_u64 rhs);
INTC_API struct intc_u256_divmod_result INTC_API_PREFIX(256_divmod_u64)(struct intc_u256 lhs, intc_u64 rhs);
INTC_API struct intc_u256               INTC_API_PREFIX(256_div_u64)(struct intc_u256 lhs, intc_u64 rhs);
INTC_API struct intc_u256               INTC_API_PREFIX(256_mod_u64)(struct intc_u256 lhs, intc_u64 rhs);

// NOTE: U256 Misc =================================================================================
// Reminder: If INTC_API_PREFIX is not defined, example API looks like: intc_u256_clz(...)
INTC_API int                            INTC_API_PREFIX(256_clz)(struct intc_u256 in);

// NOTE: U256 Printing =============================================================================
// Reminder: If INTC_API_PREFIX is not defined, example API looks like: intc_u256_str(...)
// Convert the 256 bit unsigned integer into a string.
// base: The number system base to print the string as where base must be (2 < base < 36).
// seperate_every_n_chars: When > 0, the string will be separated by the 'seperate_ch' for every N
//                         characters specified in this parameter.
// seperate_ch: The character to use to separate the tring. If
//              'seperate_every_n_chars <= 0' then this parameter is ignored.
// return: The integer converted to a string, on failure, an empty string is
// returned. The string is always null-terminated. The string's size is not
// inclusive of the null-terminator.
INTC_API struct intc_u256_string        INTC_API_PREFIX(256_str)(struct intc_u256 in, unsigned base, size_t separate_every_n_chars, char separate_ch);

// Helper function that calls intc_u256_str with base 10 (i.e. human readable).
INTC_API struct intc_u256_string        INTC_API_PREFIX(256_int_str)(struct intc_u256 in, size_t separate_every_n_chars, char separate_ch);

// Helper function that calls intc_u256_str with base 10 (i.e. human readable),
// seperated by the thousands with a comma, i.e. 1,000.
INTC_API struct intc_u256_string        INTC_API_PREFIX(256_readable_int_str)(struct intc_u256 in);

// Helper function that calls intc_u256_str with base 16 (i.e. hex)
INTC_API struct intc_u256_string        INTC_API_PREFIX(256_hex_str)(struct intc_u256 in, size_t separate_every_n_chars, char separate_ch);

// Helper function that calls intc_u256_str with base 16 (i.e. hex), seperated
// every 4 hex characters with '_', i.e. ffff_ffff
INTC_API struct intc_u256_string        INTC_API_PREFIX(256_readable_hex_str)(struct intc_u256 in);
INTC_END_EXTERN_C

#if !defined(INTC_NO_CPP_FEATURES)
// NOTE: U256 CPP Bitwise ==========================================================================
INTC_API intc_u256 operator&(intc_u256 lhs, intc_u256 rhs);
INTC_API intc_u256 operator|(intc_u256 lhs, intc_u256 rhs);
INTC_API intc_u256 operator^(intc_u256 lhs, intc_u256 rhs);
INTC_API intc_u256 operator~(intc_u256 lhs);
INTC_API intc_u256 operator<<(intc_u256 lhs, unsigned rhs);
INTC_API intc_u256 operator>>(intc_u256 lhs, unsigned rhs);

// NOTE: U256 CPP Equality =========================================================================
INTC_API bool operator==(intc_u256 lhs, intc_u256 rhs);
INTC_API bool operator!=(intc_u256 lhs, intc_u256 rhs);

// NOTE: U256 CPP Relational =======================================================================
INTC_API bool operator>(intc_u256 lhs, intc_u256 rhs);
INTC_API bool operator<(intc_u256 lhs, intc_u256 rhs);
INTC_API bool operator>=(intc_u256 lhs, intc_u256 rhs);
INTC_API bool operator<=(intc_u256 lhs, intc_u256 rhs);

// NOTE: U256 CPP Arithmetic =======================================================================
INTC_API intc_u256 operator+(intc_u256 lhs, intc_u256 rhs);
INTC_API intc_u256 operator-(intc_u256 lhs, intc_u256 rhs);
INTC_API intc_u256 operator*(intc_u256 lhs, intc_u256 rhs);
INTC_API intc_u256 operator/(intc_u256 lhs, intc_u256 rhs);
INTC_API intc_u256 operator%(intc_u256 lhs, intc_u256 rhs);

// NOTE: U256 CPP Other ============================================================================
INTC_API intc_u256 &operator&=(intc_u256 &lhs, intc_u256 rhs);
INTC_API intc_u256 &operator|=(intc_u256 &lhs, intc_u256 rhs);
INTC_API intc_u256 &operator^=(intc_u256 &lhs, intc_u256 rhs);
INTC_API intc_u256 &operator<<=(intc_u256 &lhs, unsigned rhs);
INTC_API intc_u256 &operator>>=(intc_u256 &lhs, unsigned rhs);

INTC_API intc_u256 &operator+=(intc_u256 &lhs, intc_u256 rhs);
INTC_API intc_u256 &operator-=(intc_u256 &lhs, intc_u256 rhs);
INTC_API intc_u256 &operator*=(intc_u256 &lhs, intc_u256 rhs);
INTC_API intc_u256 &operator/=(intc_u256 &lhs, intc_u256 rhs);
INTC_API intc_u256 &operator%=(intc_u256 &lhs, intc_u256 rhs);

INTC_API intc_u256 &operator++(intc_u256 &lhs);
INTC_API intc_u256 &operator--(intc_u256 &lhs);
INTC_API intc_u256  operator++(intc_u256 &lhs, int);
INTC_API intc_u256  operator--(intc_u256 &lhs, int);
#endif // !defined(INTC_NO_CPP_FEATURES)
#endif // !defined(INTC_NO_U256)
#endif // INTC_H

#if defined(INTC_IMPLEMENTATION)
static bool const INTC__U8_IS_8_BITS  [sizeof(intc_u8)  == 1 ? 1 : -1] = INTC_ZERO_INIT;
static bool const INTC__U16_IS_16_BITS[sizeof(intc_u16) == 2 ? 1 : -1] = INTC_ZERO_INIT;
static bool const INTC__U32_IS_32_BITS[sizeof(intc_u32) == 4 ? 1 : -1] = INTC_ZERO_INIT;
static bool const INTC__U64_IS_64_BITS[sizeof(intc_u64) == 8 ? 1 : -1] = INTC_ZERO_INIT;

INTC_BEGIN_EXTERN_C
// NOTE: 128 Bit Unsigned Integer ==================================================================
// NOTE: U128 Converters ===========================================================================
INTC_API struct intc_u128_init_result INTC_API_PREFIX(128_init_hex_cstring)(char const *string, size_t size, char separator)
{
    struct intc_u128_init_result result = INTC_ZERO_INIT;
    if (string == 0 || size <= 0)
        return result;

    if      (size >= 2 && string[0] == '0' && string[1] == 'x') { string += 2; size -= 2; }
    else if (size >= 1 && string[0] == 'x')                     { string += 1; size -= 1; }

    struct intc_u128 dest = INTC_ZERO_INIT;
    for (size_t index = size - 1, bits_written = 0; index < size; index--) {
        if (bits_written >= (sizeof(dest) * 8))
            return result;

        char hex_ch = string[index];
        if (separator != 0 && hex_ch == separator)
            continue;

        unsigned char bits4 =   (hex_ch >= '0' && hex_ch <= '9') ?  0 + (hex_ch - '0')
                              : (hex_ch >= 'a' && hex_ch <= 'f') ? 10 + (hex_ch - 'a')
                              : (hex_ch >= 'A' && hex_ch <= 'F') ? 10 + (hex_ch - 'A')
                              : 0xFF;

        if (bits4 == 0xFF)
            return result;

        struct intc_u128 bits4_as_u128 = INTC_U64_TO_U128(bits4);
        dest                           = INTC_API_PREFIX(128_or)(dest, (INTC_API_PREFIX(128_lshift)(bits4_as_u128, (unsigned)bits_written)));
        bits_written += 4;
    }

    result.value   = dest;
    result.success = true;
    return result;
}

size_t const INTC_API_PREFIX(U128_MAX_STRING_SIZE) = sizeof("340282366920938463463374607431768211455") - 1; // 2^128
INTC_API struct intc_u128_init_result INTC_API_PREFIX(128_init_cstring)(char const *string, size_t size, char separator)
{
    struct intc_u128_init_result result = INTC_ZERO_INIT;
    if (string == 0 || size <= 0)
        return result;

    struct intc_u128 dest = INTC_ZERO_INIT;
    for (size_t index = 0, digits_written = 0; index < size; index++) {
        if (digits_written >= INTC_API_PREFIX(U128_MAX_STRING_SIZE))
            return result;

        char digit = string[index];
        if (separator != 0 && digit == separator)
            continue;

        intc_u64 value = (digit >= '0' && digit <= '9') ? (digit - '0') : 0xFF;
        if (value == 0xFF)
            return result;

        dest = intc_u128_mul_u64(dest, 10);
        dest = intc_u128_add_u64(dest, value);
        digits_written++;
    }

    result.value   = dest;
    result.success = true;
    return result;
}

INTC_API bool INTC_API_PREFIX(128_as_bool)(struct intc_u128 in)
{
    bool result = in.lo | in.hi;
    return result;
}

INTC_API intc_u8 INTC_API_PREFIX(128_as_u8)(struct intc_u128 in)
{
    intc_u8 result = (intc_u8)in.lo;
    return result;
}

INTC_API intc_u16 INTC_API_PREFIX(128_as_u16)(struct intc_u128 in)
{
    intc_u16 result = (intc_u16)in.lo;
    return result;
}

INTC_API intc_u32 INTC_API_PREFIX(128_as_u32)(struct intc_u128 in)
{
    intc_u32 result = (intc_u32)in.lo;
    return result;
}

INTC_API intc_u64 INTC_API_PREFIX(128_as_u64)(struct intc_u128 in)
{
    intc_u64 result = (intc_u64)in.lo;
    return result;
}

// NOTE: U128 Bitwise ==============================================================================
INTC_API struct intc_u128 INTC_API_PREFIX(128_and)(struct intc_u128 lhs, struct intc_u128 rhs)
{
    struct intc_u128 result = INTC_U128(lhs.lo & rhs.lo, lhs.hi & rhs.hi);
    return result;
}

INTC_API struct intc_u128 INTC_API_PREFIX(128_or)(struct intc_u128 lhs, struct intc_u128 rhs)
{
    struct intc_u128 result = INTC_U128(lhs.lo | rhs.lo, lhs.hi | rhs.hi);
    return result;
}

INTC_API struct intc_u128 INTC_API_PREFIX(128_xor)(struct intc_u128 lhs, struct intc_u128 rhs)
{
    struct intc_u128 result = INTC_U128(lhs.lo ^ rhs.lo, lhs.hi ^ rhs.hi);
    return result;
}

INTC_API struct intc_u128 INTC_API_PREFIX(128_negate)(struct intc_u128 lhs)
{
    struct intc_u128 result = INTC_U128(~lhs.lo, ~lhs.hi);
    return result;
}

INTC_API struct intc_u128 INTC_API_PREFIX(128_lshift)(struct intc_u128 lhs, unsigned shift)
{
    if ((shift >= 128))
        return INTC_U128_ZERO;

    if (shift == 64)
        return INTC_U128(0, lhs.lo);

    if (shift == 0)
        return lhs;

    if (shift < 64)
        return INTC_U128(lhs.lo << shift, (lhs.hi << shift) + (lhs.lo >> (64 - shift)));

    if ((128 > shift) && (shift > 64))
        return INTC_U128(0, lhs.lo << (shift - 64));

    return INTC_U128_ZERO;
}

INTC_API struct intc_u128 INTC_API_PREFIX(128_rshift)(struct intc_u128 lhs, unsigned shift)
{
    if ((shift >= 128))
        return INTC_U128_ZERO;

    if (shift == 64)
        return INTC_U128(lhs.hi, 0);

    if (shift == 0)
        return lhs;

    if (shift < 64)
        return INTC_U128((lhs.hi << (64 - shift)) + (lhs.lo >> shift), lhs.hi >> shift);

    if ((128 > shift) && (shift > 64))
        return INTC_U128((lhs.hi >> (shift - 64)), 0);

    return INTC_U128_ZERO;
}

// NOTE: U128 Equality =============================================================================
INTC_API bool INTC_API_PREFIX(128_eq)(struct intc_u128 lhs, struct intc_u128 rhs)
{
    bool result = ((lhs.lo == rhs.lo) && (lhs.hi == rhs.hi));
    return result;
}

INTC_API bool INTC_API_PREFIX(128_neq)(struct intc_u128 lhs, struct intc_u128 rhs)
{
    bool result = ((lhs.lo != rhs.lo) | (lhs.hi != rhs.hi));
    return result;
}

// NOTE: U128 Equality U64 Helpers =================================================================
INTC_API bool INTC_API_PREFIX(128_eq_u64)(struct intc_u128 lhs, intc_u64 rhs)
{
    bool result = INTC_API_PREFIX(128_eq)(lhs, INTC_U64_TO_U128(rhs));
    return result;
}

INTC_API bool INTC_API_PREFIX(128_neq_u64)(struct intc_u128 lhs, intc_u64 rhs)
{
    bool result = INTC_API_PREFIX(128_neq)(lhs, INTC_U64_TO_U128(rhs));
    return result;
}

// NOTE: U128 Relational ===========================================================================
INTC_API bool INTC_API_PREFIX(128_gt)(struct intc_u128 lhs, struct intc_u128 rhs)
{
    bool result = (lhs.hi == rhs.hi) ? (lhs.lo > rhs.lo) : (lhs.hi > rhs.hi);
    return result;
}

INTC_API bool INTC_API_PREFIX(128_lt)(struct intc_u128 lhs, struct intc_u128 rhs)
{
    bool result = (lhs.hi == rhs.hi) ? (lhs.lo < rhs.lo) : (lhs.hi < rhs.hi);
    return result;
}

INTC_API bool INTC_API_PREFIX(128_gt_eq)(struct intc_u128 lhs, struct intc_u128 rhs)
{
    bool result = (lhs.hi == rhs.hi) ? (lhs.lo >= rhs.lo) : (lhs.hi >= rhs.hi);
    return result;
}

INTC_API bool INTC_API_PREFIX(128_lt_eq)(struct intc_u128 lhs, struct intc_u128 rhs)
{
    bool result = (lhs.hi == rhs.hi) ? (lhs.lo <= rhs.lo) : (lhs.hi <= rhs.hi);
    return result;
}

// NOTE: U128 Relational U64 Helpers ===============================================================
INTC_API bool INTC_API_PREFIX(128_gt_u64)(struct intc_u128 lhs, intc_u64 rhs)
{
    bool result = INTC_API_PREFIX(128_gt)(lhs, INTC_U64_TO_U128(rhs));
    return result;
}

INTC_API bool INTC_API_PREFIX(128_lt_u64)(struct intc_u128 lhs, intc_u64 rhs)
{
    bool result = INTC_API_PREFIX(128_lt)(lhs, INTC_U64_TO_U128(rhs));
    return result;
}

INTC_API bool INTC_API_PREFIX(128_gt_eq_u64)(struct intc_u128 lhs, intc_u64 rhs)
{
    bool resugt_eq = INTC_API_PREFIX(128_gt_eq)(lhs, INTC_U64_TO_U128(rhs));
    return resugt_eq;
}

INTC_API bool INTC_API_PREFIX(128_lt_eq_u64)(struct intc_u128 lhs, intc_u64 rhs)
{
    bool result_eq = INTC_API_PREFIX(128_lt_eq)(lhs, INTC_U64_TO_U128(rhs));
    return result_eq;
}

// NOTE: U128 Arithmetic ===========================================================================
INTC_API struct intc_u128 INTC_API_PREFIX(128_add)(struct intc_u128 lhs, struct intc_u128 rhs)
{
    struct intc_u128 result = INTC_U128(lhs.lo + rhs.lo, lhs.hi + rhs.hi + ((lhs.lo + rhs.lo) < lhs.lo));
    return result;
}

INTC_API struct intc_u128 INTC_API_PREFIX(128_sub)(struct intc_u128 lhs, struct intc_u128 rhs)
{
    struct intc_u128 result = INTC_U128(lhs.lo - rhs.lo, lhs.hi - rhs.hi - ((lhs.lo - rhs.lo) > lhs.lo));
    return result;
}

INTC_API struct intc_u128 INTC_API_PREFIX(128_mul)(struct intc_u128 lhs, struct intc_u128 rhs)
{
    // split values into 4 32-bit parts
    intc_u64 top[4]    = {lhs.hi >> 32, lhs.hi & 0xffffffff, lhs.lo >> 32, lhs.lo & 0xffffffff};
    intc_u64 bottom[4] = {rhs.hi >> 32, rhs.hi & 0xffffffff, rhs.lo >> 32, rhs.lo & 0xffffffff};
    intc_u64 products[4][4];

    // multiply each component of the values
    for (int y = 3; y > -1; y--) {
        for (int x = 3; x > -1; x--) {
            products[3 - x][y] = top[x] * bottom[y];
        }
    }

    // first row
    intc_u64 fourth32 = (products[0][3] & 0xffffffff);
    intc_u64 third32  = (products[0][2] & 0xffffffff) + (products[0][3] >> 32);
    intc_u64 second32 = (products[0][1] & 0xffffffff) + (products[0][2] >> 32);
    intc_u64 first32  = (products[0][0] & 0xffffffff) + (products[0][1] >> 32);

    // second row
    third32  += (products[1][3] & 0xffffffff);
    second32 += (products[1][2] & 0xffffffff) + (products[1][3] >> 32);
    first32  += (products[1][1] & 0xffffffff) + (products[1][2] >> 32);

    // third row
    second32 += (products[2][3] & 0xffffffff);
    first32  += (products[2][2] & 0xffffffff) + (products[2][3] >> 32);

    // fourth row
    first32  += (products[3][3] & 0xffffffff);

    // move carry to next digit
    third32  += fourth32 >> 32;
    second32 += third32  >> 32;
    first32  += second32 >> 32;

    // remove carry from current digit
    fourth32 &= 0xffffffff;
    third32  &= 0xffffffff;
    second32 &= 0xffffffff;
    first32  &= 0xffffffff;

    // combine components
    struct intc_u128 result = INTC_U128((third32 << 32) | fourth32, (first32 << 32) | second32);
    return result;
}

INTC_API struct intc_u128_divmod_result INTC_API_PREFIX(128_divmod)(struct intc_u128 lhs, struct intc_u128 rhs)
{
    // Save some calculations /////////////////////
    struct intc_u128_divmod_result result = INTC_ZERO_INIT;

    if (INTC_API_PREFIX(128_eq)(rhs, INTC_U128_ZERO)) {
        INTC_ASSERT(!"Division by zero");
        return result;
    }

    if (INTC_API_PREFIX(128_eq)(rhs, INTC_U64_TO_U128(1))) {
        result.quot = lhs;
        return result;
    }

    if (INTC_API_PREFIX(128_eq)(lhs, rhs)) {
        result.quot = INTC_U64_TO_U128(1);
        return result;
    }

    if (INTC_API_PREFIX(128_eq)(lhs, INTC_U128_ZERO) || INTC_API_PREFIX(128_lt)(lhs, rhs)) {
        result.rem = lhs;
        return result;
    }

    int count = INTC_API_PREFIX(128_clz)(lhs);
    for (int x = count; x > 0; x--) {
        result.quot = INTC_API_PREFIX(128_lshift)(result.quot, 1);
        result.rem  = INTC_API_PREFIX(128_lshift)(result.rem, 1);

        if (INTC_API_PREFIX(128_rshift)(lhs, x - 1U).lo & 1)
            result.rem = INTC_API_PREFIX(128_add)(result.rem, INTC_U64_TO_U128(1));

        if (INTC_API_PREFIX(128_gt_eq)(result.rem, rhs)) {
            result.rem  = INTC_API_PREFIX(128_sub)(result.rem, rhs);
            result.quot = INTC_API_PREFIX(128_add)(result.quot, INTC_U64_TO_U128(1));
        }
    }

    return result;
}

INTC_API struct intc_u128 INTC_API_PREFIX(128_div)(struct intc_u128 lhs, struct intc_u128 rhs)
{
    struct intc_u128 result = INTC_API_PREFIX(128_divmod)(lhs, rhs).quot;
    return result;
}

INTC_API struct intc_u128 INTC_API_PREFIX(128_mod)(struct intc_u128 lhs, struct intc_u128 rhs)
{
    struct intc_u128 result = INTC_API_PREFIX(128_divmod)(lhs, rhs).rem;
    return result;
}

// NOTE: U128 Arithmetic U64 Helpers ===============================================================
INTC_API struct intc_u128 INTC_API_PREFIX(128_add_u64)(struct intc_u128 lhs, intc_u64 rhs)
{
    struct intc_u128 result = INTC_API_PREFIX(128_add)(lhs, INTC_U64_TO_U128(rhs));
    return result;
}

INTC_API struct intc_u128 INTC_API_PREFIX(128_sub_u64)(struct intc_u128 lhs, intc_u64 rhs)
{
    struct intc_u128 result = INTC_API_PREFIX(128_sub)(lhs, INTC_U64_TO_U128(rhs));
    return result;
}

INTC_API struct intc_u128 INTC_API_PREFIX(128_mul_u64)(struct intc_u128 lhs, intc_u64 rhs)
{
    struct intc_u128 result = INTC_API_PREFIX(128_mul)(lhs, INTC_U64_TO_U128(rhs));
    return result;
}

INTC_API struct intc_u128_divmod_result INTC_API_PREFIX(128_divmod_u64)(struct intc_u128 lhs, intc_u64 rhs)
{
    struct intc_u128_divmod_result result = INTC_API_PREFIX(128_divmod)(lhs, INTC_U64_TO_U128(rhs));
    return result;
}

INTC_API struct intc_u128 INTC_API_PREFIX(128_div_u64)(struct intc_u128 lhs, intc_u64 rhs)
{
    struct intc_u128 result = INTC_API_PREFIX(128_div)(lhs, INTC_U64_TO_U128(rhs));
    return result;
}

INTC_API struct intc_u128 INTC_API_PREFIX(128_mod_u64)(struct intc_u128 lhs, intc_u64 rhs)
{
    struct intc_u128 result = INTC_API_PREFIX(128_mod)(lhs, INTC_U64_TO_U128(rhs));
    return result;
}

// NOTE: U128 Misc =================================================================================
INTC_API int INTC_API_PREFIX(128_clz)(struct intc_u128 in)
{
    int result = in.hi ? 64 /*include the 64 bits of the low part*/ : 0;
    for (intc_u64 val = result ? in.hi : in.lo;
         val;
         val >>= 1, result++)
        ;

    return result;
}

// NOTE: U128 Printing =============================================================================
INTC_API struct intc_u128_string INTC_API_PREFIX(128_str)(struct intc_u128 in, unsigned base, size_t separate_every_n_chars, char separate_ch)
{
    struct intc_u128_string val = INTC_ZERO_INIT;
    if ((base < 2) || (base > 36))
        return val;

    if (INTC_API_PREFIX(128_eq)(in, INTC_U128_ZERO)) {
        val.data[val.size++] = '0';
    } else {
        int insert_count = 0;
        struct intc_u128_divmod_result div_result;
        div_result.quot = in;
        div_result.rem  = INTC_U128_ZERO;

        do {
            div_result           = INTC_API_PREFIX(128_divmod)(div_result.quot, INTC_U64_TO_U128(base));
            val.data[val.size++] = "0123456789abcdefghijklmnopqrstuvwxyz"[div_result.rem.lo];

            if (separate_ch && separate_every_n_chars > 0 && INTC_API_PREFIX(128_as_bool)(div_result.quot)) {
                insert_count++;
                if (insert_count % separate_every_n_chars == 0)
                    val.data[val.size++] = separate_ch;
            }
        } while (INTC_API_PREFIX(128_as_bool)(div_result.quot));
    }

    INTC_ASSERT(val.size < (int)sizeof(val.data) - 1);

    struct intc_u128_string result;
    result.size = 0;

    for (int i = val.size - 1; i >= 0; i--)
        result.data[result.size++] = val.data[i];
    result.data[result.size] = 0;

    return result;
}

INTC_API struct intc_u128_string INTC_API_PREFIX(128_int_str)(struct intc_u128 in, size_t separate_every_n_chars, char separate_ch)
{
    struct intc_u128_string result = INTC_API_PREFIX(128_str)(in, 10 /*base*/, separate_every_n_chars, separate_ch);
    return result;
}

INTC_API struct intc_u128_string INTC_API_PREFIX(128_readable_int_str)(struct intc_u128 in)
{
    struct intc_u128_string result = INTC_API_PREFIX(128_str)(in, 10 /*base*/, 3 /*separate_every_n_chars*/, ',' /*separate_ch*/);
    return result;
}

INTC_API struct intc_u128_string INTC_API_PREFIX(128_hex_str)(struct intc_u128 in, size_t separate_every_n_chars, char separate_ch)
{
    struct intc_u128_string result = INTC_API_PREFIX(128_str)(in, 16 /*base*/, separate_every_n_chars, separate_ch);
    return result;
}

INTC_API struct intc_u128_string INTC_API_PREFIX(128_readable_hex_str)(struct intc_u128 in)
{
    struct intc_u128_string result = INTC_API_PREFIX(128_str)(in, 16 /*base*/, 4 /*separate_every_n_chars*/, '_' /*separate_ch*/);
    return result;
}
INTC_END_EXTERN_C

#if !defined(INTC_NO_CPP_FEATURES)
// NOTE: U128 CPP Bitwise ==========================================================================
INTC_API intc_u128 operator&(intc_u128 lhs, intc_u128 rhs)
{
    intc_u128 result = INTC_API_PREFIX(128_and)(lhs, rhs);
    return result;
}

INTC_API intc_u128 operator|(intc_u128 lhs, intc_u128 rhs)
{
    intc_u128 result = INTC_API_PREFIX(128_or)(lhs, rhs);
    return result;
}

INTC_API intc_u128 operator^(intc_u128 lhs, intc_u128 rhs)
{
    intc_u128 result = INTC_API_PREFIX(128_xor)(lhs, rhs);
    return result;
}

INTC_API intc_u128 operator~(intc_u128 lhs)
{
    intc_u128 result = INTC_API_PREFIX(128_negate)(lhs);
    return result;
}

INTC_API intc_u128 operator<<(intc_u128 lhs, unsigned shift)
{
    intc_u128 result = INTC_API_PREFIX(128_lshift)(lhs, shift);
    return result;
}

INTC_API intc_u128 operator>>(intc_u128 lhs, unsigned shift)
{
    intc_u128 result = INTC_API_PREFIX(128_rshift)(lhs, shift);
    return result;
}

// NOTE: U128 CPP Equality =========================================================================
INTC_API bool operator==(intc_u128 lhs, intc_u128 rhs)
{
    bool result = INTC_API_PREFIX(128_eq)(lhs, rhs);
    return result;
}

INTC_API bool operator!=(intc_u128 lhs, intc_u128 rhs)
{
    bool result = INTC_API_PREFIX(128_neq)(lhs, rhs);
    return result;
}

// NOTE: U128 CPP Relational =======================================================================
INTC_API bool operator>(intc_u128 lhs, intc_u128 rhs)
{
    bool result = INTC_API_PREFIX(128_gt)(lhs, rhs);
    return result;
}

INTC_API bool operator<(intc_u128 lhs, intc_u128 rhs)
{
    bool result = INTC_API_PREFIX(128_lt)(lhs, rhs);
    return result;
}

INTC_API bool operator>=(intc_u128 lhs, intc_u128 rhs)
{
    bool result = INTC_API_PREFIX(128_gt_eq)(lhs, rhs);
    return result;
}

INTC_API bool operator<=(intc_u128 lhs, intc_u128 rhs)
{
    bool result = INTC_API_PREFIX(128_lt_eq)(lhs, rhs);
    return result;
}

// NOTE: U128 CPP Arithmetic =======================================================================
INTC_API intc_u128 operator+(intc_u128 lhs, intc_u128 rhs)
{
    intc_u128 result = INTC_API_PREFIX(128_add)(lhs, rhs);
    return result;
}

INTC_API intc_u128 operator-(intc_u128 lhs, intc_u128 rhs)
{
    intc_u128 result = INTC_API_PREFIX(128_sub)(lhs, rhs);
    return result;
}

INTC_API intc_u128 operator*(intc_u128 lhs, intc_u128 rhs)
{
    intc_u128 result = INTC_API_PREFIX(128_mul)(lhs, rhs);
    return result;
}

INTC_API intc_u128 operator/(intc_u128 lhs, intc_u128 rhs)
{
    intc_u128 result = INTC_API_PREFIX(128_div)(lhs, rhs);
    return result;
}

INTC_API intc_u128 operator%(intc_u128 lhs, intc_u128 rhs)
{
    intc_u128 result = INTC_API_PREFIX(128_mod)(lhs, rhs);
    return result;
}

// NOTE: U128 CPP Other ============================================================================
INTC_API intc_u128 &operator&=(intc_u128 &lhs, intc_u128 rhs)
{
    lhs = lhs & rhs;
    return lhs;
}

INTC_API intc_u128 &operator|=(intc_u128 &lhs, intc_u128 rhs)
{
    lhs = lhs | rhs;
    return lhs;
}

INTC_API intc_u128 &operator^=(intc_u128 &lhs, intc_u128 rhs)
{
    lhs = lhs ^ rhs;
    return lhs;
}

INTC_API intc_u128 &operator<<=(intc_u128 &lhs, unsigned rhs)
{
    lhs = lhs << rhs;
    return lhs;
}

INTC_API intc_u128 &operator>>=(intc_u128 &lhs, unsigned rhs)
{
    lhs = lhs >> rhs;
    return lhs;
}

INTC_API intc_u128 &operator+=(intc_u128 &lhs, intc_u128 rhs)
{
    lhs = lhs + rhs;
    return lhs;
}

INTC_API intc_u128 &operator-=(intc_u128 &lhs, intc_u128 rhs)
{
    lhs = lhs - rhs;
    return lhs;
}

INTC_API intc_u128 &operator*=(intc_u128 &lhs, intc_u128 rhs)
{
    lhs = lhs * rhs;
    return lhs;
}

INTC_API intc_u128 &operator/=(intc_u128 &lhs, intc_u128 rhs)
{
    lhs = lhs / rhs;
    return lhs;
}

INTC_API intc_u128 &operator%=(intc_u128 &lhs, intc_u128 rhs)
{
    lhs = lhs % rhs;
    return lhs;
}

INTC_API intc_u128 &operator++(intc_u128 &lhs)
{
    lhs = lhs + 1;
    return lhs;
}

INTC_API intc_u128 &operator--(intc_u128 &lhs)
{
    lhs = lhs - 1;
    return lhs;
}

INTC_API intc_u128 operator++(intc_u128 &lhs, int)
{
    intc_u128 val = lhs;
    ++val;
    return val;
}

INTC_API intc_u128 operator--(intc_u128 &lhs, int)
{
    intc_u128 val = lhs;
    --val;
    return val;
}
#endif // !defined(INTC_NO_CPP_FEATURES)

#if !defined(INTC_NO_U256)
INTC_BEGIN_EXTERN_C
// NOTE: 256 Bit Unsigned Integer ==================================================================
// NOTE: U256 Converters ===========================================================================
INTC_API struct intc_u256_init_result INTC_API_PREFIX(256_init_hex_cstring)(char const *string, size_t size, char separator)
{
    struct intc_u256_init_result result = INTC_ZERO_INIT;
    if (string == 0 || size <= 0)
        return result;

    if      (size >= 2 && string[0] == '0' && string[1] == 'x') { string += 2; size -= 2; }
    else if (size >= 1 && string[0] == 'x')                     { string += 1; size -= 1; }

    struct intc_u256 dest = INTC_ZERO_INIT;
    for (size_t index = size - 1, bits_written = 0; index < size; index--) {
        if (bits_written >= (sizeof(dest) * 8))
            return result;

        char hex_ch = string[index];
        if (separator != 0 && hex_ch == separator)
            continue;

        unsigned char bits4 =   (hex_ch >= '0' && hex_ch <= '9') ?  0 + (hex_ch - '0')
                              : (hex_ch >= 'a' && hex_ch <= 'f') ? 10 + (hex_ch - 'a')
                              : (hex_ch >= 'A' && hex_ch <= 'F') ? 10 + (hex_ch - 'A')
                              : 0xFF;

        if (bits4 == 0xFF)
            return result;

        struct intc_u256 bits4_as_u256 = INTC_U64_TO_U256(bits4);
        dest                           = INTC_API_PREFIX(256_or)(dest, (INTC_API_PREFIX(256_lshift)(bits4_as_u256, (unsigned)bits_written)));
        bits_written += 4;
    }

    result.value   = dest;
    result.success = true;
    return result;
}

INTC_API struct intc_u256_init_result INTC_API_PREFIX(256_init_cstring)(char const *string, size_t size, char separator)
{
    struct intc_u256_init_result result = INTC_ZERO_INIT;
    if (string == 0 || size <= 0)
        return result;

    size_t const U256_MAX_STRING_SIZE = sizeof("115792089237316195423570985008687907853269984665640564039457584007913129639935") - 1; // 2^256
    struct intc_u256 dest             = INTC_ZERO_INIT;
    for (size_t index = 0, digits_written = 0; index < size; index++) {
        if (digits_written >= U256_MAX_STRING_SIZE)
            return result;

        char digit = string[index];
        if (separator != 0 && digit == separator)
            continue;

        intc_u64 value = (digit >= '0' && digit <= '9') ? (digit - '0') : 0xFF;
        if (value == 0xFF)
            return result;

        dest = INTC_API_PREFIX(256_mul_u64)(dest, 10);
        dest = INTC_API_PREFIX(256_add_u64)(dest, value);
        digits_written++;
    }

    result.value   = dest;
    result.success = true;
    return result;
}

INTC_API bool INTC_API_PREFIX(256_as_bool)(struct intc_u256 in)
{
    bool result = (bool)((unsigned)INTC_API_PREFIX(128_as_bool)(in.lo) |
                         (unsigned)INTC_API_PREFIX(128_as_bool)(in.hi));
    return result;
}

INTC_API intc_u8 INTC_API_PREFIX(256_as_u8)(struct intc_u256 in)
{
    intc_u8 result = (intc_u8)in.lo.lo;
    return result;
}

INTC_API intc_u16 INTC_API_PREFIX(256_as_u16)(struct intc_u256 in)
{
    intc_u16 result = (intc_u16)in.lo.lo;
    return result;
}

INTC_API intc_u32 INTC_API_PREFIX(256_as_u32)(struct intc_u256 in)
{
    intc_u32 result = (intc_u32)in.lo.lo;
    return result;
}

INTC_API intc_u64 INTC_API_PREFIX(256_as_u64)(struct intc_u256 in)
{
    intc_u64 result = (intc_u64)in.lo.lo;
    return result;
}

INTC_API struct intc_u128 INTC_API_PREFIX(256_as_u128)(struct intc_u256 in)
{
    struct intc_u128 result = in.lo;
    return result;
}

// NOTE: U256 Bitwise ==============================================================================
INTC_API struct intc_u256 INTC_API_PREFIX(256_and)(struct intc_u256 lhs, struct intc_u256 rhs)
{
    struct intc_u256 result = INTC_U256(INTC_API_PREFIX(128_and)(lhs.lo, rhs.lo), INTC_API_PREFIX(128_and)(lhs.hi, rhs.hi));
    return result;
}

INTC_API struct intc_u256 INTC_API_PREFIX(256_or)(struct intc_u256 lhs, struct intc_u256 rhs)
{
    struct intc_u256 result = INTC_U256(INTC_API_PREFIX(128_or)(lhs.lo, rhs.lo), INTC_API_PREFIX(128_or)(lhs.hi, rhs.hi));
    return result;
}

INTC_API struct intc_u256 INTC_API_PREFIX(256_xor)(struct intc_u256 lhs, struct intc_u256 rhs)
{
    struct intc_u256 result = INTC_U256(INTC_API_PREFIX(128_xor)(lhs.lo, rhs.lo), INTC_API_PREFIX(128_xor)(lhs.hi, rhs.hi));
    return result;
}

INTC_API struct intc_u256 INTC_API_PREFIX(256_and_u128)(struct intc_u256 lhs, struct intc_u128 rhs)
{
    struct intc_u256 result = INTC_U256(INTC_API_PREFIX(128_and)(lhs.lo, rhs), INTC_U128_ZERO);
    return result;
}

INTC_API struct intc_u256 INTC_API_PREFIX(256_or_u128)(struct intc_u256 lhs, struct intc_u128 rhs)
{
    struct intc_u256 result = INTC_U256(INTC_API_PREFIX(128_or)(lhs.lo, rhs), lhs.hi);
    return result;
}

INTC_API struct intc_u256 INTC_API_PREFIX(256_xor_u128)(struct intc_u256 lhs, struct intc_u128 rhs)
{
    struct intc_u256 result = INTC_U256(INTC_API_PREFIX(128_xor)(lhs.lo, rhs), lhs.hi);
    return result;
}

INTC_API struct intc_u256 INTC_API_PREFIX(256_negate)(struct intc_u256 lhs)
{
    struct intc_u256 result = INTC_U256(INTC_API_PREFIX(128_negate)(lhs.lo), INTC_API_PREFIX(128_negate)(lhs.hi));
    return result;
}

INTC_API struct intc_u256 INTC_API_PREFIX(256_lshift)(struct intc_u256 lhs, unsigned shift)
{

    if (shift >= 256)
        return INTC_U256_ZERO;

    if (shift == 128)
        return INTC_U256(INTC_U128_ZERO, lhs.lo);

    if (shift == 0)
        return lhs;

    if (shift < 128) {
        struct intc_u128 lo = INTC_API_PREFIX(128_lshift)(lhs.lo, shift);
        struct intc_u128 hi = INTC_API_PREFIX(128_add)(INTC_API_PREFIX(128_lshift)(lhs.hi, shift), INTC_API_PREFIX(128_rshift)(lhs.lo, 128 - shift));
        return INTC_U256(lo, hi);
    }

    if ((256 > shift) && (shift > 128))
        return INTC_U256(INTC_U128_ZERO, INTC_API_PREFIX(128_lshift)(lhs.lo, (shift - 128)));

    return INTC_U256_ZERO;
}

INTC_API struct intc_u256 INTC_API_PREFIX(256_rshift)(struct intc_u256 lhs, unsigned shift)
{
    if (shift >= 256)
        return INTC_U256_ZERO;

    if (shift == 128)
        return INTC_U256(lhs.hi, INTC_U128_ZERO);

    if (shift == 0)
        return lhs;

    if (shift < 128) {
        struct intc_u128 lo = INTC_API_PREFIX(128_add)(INTC_API_PREFIX(128_lshift)(lhs.hi, 128 - shift), INTC_API_PREFIX(128_rshift)(lhs.lo, shift));
        struct intc_u128 hi = INTC_API_PREFIX(128_rshift)(lhs.hi, shift);
        return INTC_U256(lo, hi);
    }

    if ((256 > shift) && (shift > 128))
        return INTC_U256(INTC_U128_ZERO, INTC_API_PREFIX(128_rshift)(lhs.hi, (shift - 128)));

    return INTC_U256_ZERO;
}

// NOTE: U256 Equality =============================================================================
INTC_API bool INTC_API_PREFIX(256_eq)(struct intc_u256 lhs, struct intc_u256 rhs)
{
    bool result = (INTC_API_PREFIX(128_eq)(lhs.lo, rhs.lo) && INTC_API_PREFIX(128_eq)(lhs.hi, rhs.hi));
    return result;
}

INTC_API bool INTC_API_PREFIX(256_neq)(struct intc_u256 lhs, struct intc_u256 rhs)
{
    bool result = (bool)((unsigned)INTC_API_PREFIX(128_neq)(lhs.lo, rhs.lo) |
                         (unsigned)INTC_API_PREFIX(128_neq)(lhs.hi, rhs.hi));
    return result;
}

// NOTE: U256 Equality U64 Helpers =================================================================
INTC_API bool INTC_API_PREFIX(256_eq_u64)(struct intc_u256 lhs, intc_u64 rhs)
{
    bool result = INTC_API_PREFIX(256_eq)(lhs, INTC_U64_TO_U256(rhs));
    return result;
}

INTC_API bool INTC_API_PREFIX(256_neq_u64)(struct intc_u256 lhs, intc_u64 rhs)
{
    bool result = INTC_API_PREFIX(256_neq)(lhs, INTC_U64_TO_U256(rhs));
    return result;
}

// NOTE: U256 Relational ===========================================================================
INTC_API bool INTC_API_PREFIX(256_gt)(struct intc_u256 lhs, struct intc_u256 rhs)
{
    bool result = INTC_API_PREFIX(128_eq)(lhs.hi, rhs.hi) ? INTC_API_PREFIX(128_gt)(lhs.lo, rhs.lo) : INTC_API_PREFIX(128_gt)(lhs.hi, rhs.hi);
    return result;
}

INTC_API bool INTC_API_PREFIX(256_lt)(struct intc_u256 lhs, struct intc_u256 rhs)
{
    bool result = INTC_API_PREFIX(128_eq)(lhs.hi, rhs.hi) ? INTC_API_PREFIX(128_lt)(lhs.lo, rhs.lo) : INTC_API_PREFIX(128_lt)(lhs.hi, rhs.hi);
    return result;
}

INTC_API bool INTC_API_PREFIX(256_gt_eq)(struct intc_u256 lhs, struct intc_u256 rhs)
{
    bool result = (bool)((unsigned)INTC_API_PREFIX(256_gt)(lhs, rhs) |
                         (unsigned)INTC_API_PREFIX(256_eq)(lhs, rhs));
    return result;
}

INTC_API bool INTC_API_PREFIX(256_lt_eq)(struct intc_u256 lhs, struct intc_u256 rhs)
{
    bool result = INTC_API_PREFIX(128_eq)(lhs.hi, rhs.hi) ? INTC_API_PREFIX(128_lt_eq)(lhs.lo, rhs.lo) : INTC_API_PREFIX(128_lt_eq)(lhs.hi, rhs.hi);
    return result;
}

// NOTE: U256 Relational U64 Helpers ===============================================================
INTC_API bool INTC_API_PREFIX(256_gt_u64)(struct intc_u256 lhs, intc_u64 rhs)
{
    bool result = INTC_API_PREFIX(256_gt)(lhs, INTC_U64_TO_U256(rhs));
    return result;
}

INTC_API bool INTC_API_PREFIX(256_lt_u64)(struct intc_u256 lhs, intc_u64 rhs)
{
    bool result = INTC_API_PREFIX(256_lt)(lhs, INTC_U64_TO_U256(rhs));
    return result;
}

INTC_API bool INTC_API_PREFIX(256_gt_eq_u64)(struct intc_u256 lhs, intc_u64 rhs)
{
    bool resugt_eq = INTC_API_PREFIX(256_gt_eq)(lhs, INTC_U64_TO_U256(rhs));
    return resugt_eq;
}

INTC_API bool INTC_API_PREFIX(256_lt_eq_u64)(struct intc_u256 lhs, intc_u64 rhs)
{
    bool result_eq = INTC_API_PREFIX(256_lt_eq)(lhs, INTC_U64_TO_U256(rhs));
    return result_eq;
}

// NOTE: U256 Arithmetic ===========================================================================
INTC_API struct intc_u256 INTC_API_PREFIX(256_add)(struct intc_u256 lhs, struct intc_u256 rhs)
{
    struct intc_u128 lo = INTC_API_PREFIX(128_add)(lhs.lo, rhs.lo);
    struct intc_u128 hi = INTC_API_PREFIX(128_add)(lhs.hi, rhs.hi);

    bool lo_overflow    = INTC_API_PREFIX(128_lt)(lo, lhs.lo);
    if (lo_overflow)
        hi = INTC_API_PREFIX(128_add)(hi, INTC_U64_TO_U128(1));

    struct intc_u256 result = INTC_U256(lo, hi);
    return result;
}

INTC_API struct intc_u256 INTC_API_PREFIX(256_sub)(struct intc_u256 lhs, struct intc_u256 rhs)
{
    struct intc_u128 lo = INTC_API_PREFIX(128_sub)(lhs.lo, rhs.lo);
    struct intc_u128 hi = INTC_API_PREFIX(128_sub)(lhs.hi, rhs.hi);

    bool lo_overflow    = INTC_API_PREFIX(128_gt)(lo, lhs.lo);
    if (lo_overflow)
        hi = INTC_API_PREFIX(128_sub)(hi, INTC_U64_TO_U128(1));

    struct intc_u256 result = INTC_U256(lo, hi);
    return result;
}

INTC_API struct intc_u256 INTC_API_PREFIX(256_mul)(struct intc_u256 lhs, struct intc_u256 rhs)
{
    // split values into 4 64-bit parts
    struct intc_u128 top[4]    = {INTC_U64_TO_U128(lhs.hi.hi), INTC_U64_TO_U128(lhs.hi.lo), INTC_U64_TO_U128(lhs.lo.hi), INTC_U64_TO_U128(lhs.lo.lo)};
    struct intc_u128 bottom[4] = {INTC_U64_TO_U128(rhs.hi.hi), INTC_U64_TO_U128(rhs.hi.lo), INTC_U64_TO_U128(rhs.lo.hi), INTC_U64_TO_U128(rhs.lo.lo)};
    struct intc_u128 products[4][4];

    // multiply each component of the values
    for (int y = 3; y > -1; y--) {
        for (int x = 3; x > -1; x--) {
            products[3 - y][x] = INTC_API_PREFIX(128_mul)(top[x], bottom[y]);
        }
    }

    // first row
    struct intc_u128 fourth64 = INTC_U64_TO_U128(products[0][3].lo);
    struct intc_u128 third64  = INTC_API_PREFIX(128_add)(INTC_U64_TO_U128(products[0][2].lo), INTC_U64_TO_U128(products[0][3].hi));
    struct intc_u128 second64 = INTC_API_PREFIX(128_add)(INTC_U64_TO_U128(products[0][1].lo), INTC_U64_TO_U128(products[0][2].hi));
    struct intc_u128 first64  = INTC_API_PREFIX(128_add)(INTC_U64_TO_U128(products[0][0].lo), INTC_U64_TO_U128(products[0][1].hi));

    // second row
    third64  = INTC_API_PREFIX(128_add)(third64, INTC_U64_TO_U128(products[1][3].lo));
    second64 = INTC_API_PREFIX(128_add)(second64, INTC_API_PREFIX(128_add)(INTC_U64_TO_U128(products[1][2].lo), INTC_U64_TO_U128(products[1][3].hi)));
    first64  = INTC_API_PREFIX(128_add)(first64, INTC_API_PREFIX(128_add)(INTC_U64_TO_U128(products[1][1].lo), INTC_U64_TO_U128(products[1][2].hi)));

    // third row
    second64 = INTC_API_PREFIX(128_add)(second64, INTC_U64_TO_U128(products[2][3].lo));
    first64  = INTC_API_PREFIX(128_add)(first64, INTC_API_PREFIX(128_add)(INTC_U64_TO_U128(products[2][2].lo), INTC_U64_TO_U128(products[2][3].hi)));

    // fourth roW
    first64  = INTC_API_PREFIX(128_add)(first64, INTC_U64_TO_U128(products[3][3].lo));

    // combines the values, taking care of carry over
    struct intc_u256 a = INTC_U256(INTC_U128_ZERO, INTC_API_PREFIX(128_lshift)(first64, 64));
    struct intc_u256 b = INTC_U256(INTC_API_PREFIX(128_lshift)(third64, 64), INTC_U64_TO_U128(third64.hi));
    struct intc_u256 c = INTC_U256(INTC_U128_ZERO, second64);
    struct intc_u256 d = INTC_U256(fourth64, INTC_U128_ZERO);

    struct intc_u256 result = INTC_API_PREFIX(256_add)(INTC_API_PREFIX(256_add)(INTC_API_PREFIX(256_add)(a, b), c), d);
    return result;
}

INTC_API struct intc_u256_divmod_result INTC_API_PREFIX(256_divmod)(struct intc_u256 lhs, struct intc_u256 rhs)
{
    // Save some calculations /////////////////////
    struct intc_u256_divmod_result result = INTC_ZERO_INIT;

    if (INTC_API_PREFIX(256_eq)(rhs, INTC_U256_ZERO)) {
        INTC_ASSERT(!"Division by zero");
        return result;
    }

    if (INTC_API_PREFIX(256_eq)(rhs, INTC_U64_TO_U256(1))) {
        result.quot = lhs;
        return result;
    }

    if (INTC_API_PREFIX(256_eq)(lhs, rhs)) {
        result.quot = INTC_U64_TO_U256(1);
        return result;
    }

    if (INTC_API_PREFIX(256_eq)(lhs, INTC_U256_ZERO) || INTC_API_PREFIX(256_lt)(lhs, rhs)) {
        result.rem = lhs;
        return result;
    }

    struct intc_u256 const ONE = INTC_U64_TO_U256(1);
    result.rem = lhs;

    int lhs_bit_count = INTC_API_PREFIX(256_clz)(lhs);
    int rhs_bit_count = INTC_API_PREFIX(256_clz)(rhs);
    int bit_count     = lhs_bit_count - rhs_bit_count;

    struct intc_u256 copyd = INTC_API_PREFIX(256_lshift)(rhs, bit_count);
    struct intc_u256 adder = INTC_API_PREFIX(256_lshift)(ONE, bit_count);

    if (INTC_API_PREFIX(256_gt)(copyd, result.rem)) {
        copyd = INTC_API_PREFIX(256_rshift)(copyd, 1);
        adder = INTC_API_PREFIX(256_rshift)(adder, 1);
    }

    while (INTC_API_PREFIX(256_gt_eq)(result.rem, rhs)) {
        if (INTC_API_PREFIX(256_gt_eq)(result.rem, copyd)) {
            result.rem  = INTC_API_PREFIX(256_sub)(result.rem, copyd);
            result.quot = INTC_API_PREFIX(256_or)(result.quot, adder);
        }
        copyd = INTC_API_PREFIX(256_rshift)(copyd, 1);
        adder = INTC_API_PREFIX(256_rshift)(adder, 1);
    }

    return result;
}

INTC_API struct intc_u256 INTC_API_PREFIX(256_div)(struct intc_u256 lhs, struct intc_u256 rhs)
{
    struct intc_u256 result = INTC_API_PREFIX(256_divmod)(lhs, rhs).quot;
    return result;
}

INTC_API struct intc_u256 INTC_API_PREFIX(256_mod)(struct intc_u256 lhs, struct intc_u256 rhs)
{
    struct intc_u256 result = INTC_API_PREFIX(256_divmod)(lhs, rhs).rem;
    return result;
}

// NOTE: U256 Arithmetic U64 Helpers ===============================================================
INTC_API struct intc_u256 INTC_API_PREFIX(256_add_u64)(struct intc_u256 lhs, intc_u64 rhs)
{
    struct intc_u256 result = INTC_API_PREFIX(256_add)(lhs, INTC_U64_TO_U256(rhs));
    return result;
}

INTC_API struct intc_u256 INTC_API_PREFIX(256_sub_u64)(struct intc_u256 lhs, intc_u64 rhs)
{
    struct intc_u256 result = INTC_API_PREFIX(256_sub)(lhs, INTC_U64_TO_U256(rhs));
    return result;
}

INTC_API struct intc_u256 INTC_API_PREFIX(256_mul_u64)(struct intc_u256 lhs, intc_u64 rhs)
{
    struct intc_u256 result = INTC_API_PREFIX(256_mul)(lhs, INTC_U64_TO_U256(rhs));
    return result;
}

INTC_API struct intc_u256_divmod_result INTC_API_PREFIX(256_divmod_u64)(struct intc_u256 lhs, intc_u64 rhs)
{
    struct intc_u256_divmod_result result = INTC_API_PREFIX(256_divmod)(lhs, INTC_U64_TO_U256(rhs));
    return result;
}

INTC_API struct intc_u256 INTC_API_PREFIX(256_div_u64)(struct intc_u256 lhs, intc_u64 rhs)
{
    struct intc_u256 result = INTC_API_PREFIX(256_div)(lhs, INTC_U64_TO_U256(rhs));
    return result;
}

INTC_API struct intc_u256 INTC_API_PREFIX(256_mod_u64)(struct intc_u256 lhs, intc_u64 rhs)
{
    struct intc_u256 result = INTC_API_PREFIX(256_mod)(lhs, INTC_U64_TO_U256(rhs));
    return result;
}

// NOTE: U256 Misc =================================================================================
INTC_API int INTC_API_PREFIX(256_clz)(struct intc_u256 in)
{
    int result = INTC_API_PREFIX(128_as_bool)(in.hi) ? 128 /*include the 128 bits of the low part*/ : 0;
    for (struct intc_u128 val = result ? in.hi : in.lo;
         INTC_API_PREFIX(128_as_bool)(val);
         val = INTC_API_PREFIX(128_rshift)(val, 1), result++)
        ;

    return result;
}

// NOTE: U256 Printing =============================================================================
INTC_API struct intc_u256_string INTC_API_PREFIX(256_str)(struct intc_u256 in, unsigned base, size_t separate_every_n_chars, char separate_ch)
{
    struct intc_u256_string val = INTC_ZERO_INIT;
    if ((base < 2) || (base > 36))
        return val;

    if (INTC_API_PREFIX(256_eq)(in, INTC_U256_ZERO)) {
        val.data[val.size++] = '0';
    } else {
        int insert_count = 0;

        struct intc_u256_divmod_result div_result = INTC_ZERO_INIT;
        div_result.quot                           = in;
        do {
            div_result          = INTC_API_PREFIX(256_divmod)(div_result.quot, INTC_U64_TO_U256(base));
            val.data[val.size++] = "0123456789abcdefghijklmnopqrstuvwxyz"[INTC_API_PREFIX(128_as_u32)(div_result.rem.lo)];

            if (separate_ch && separate_every_n_chars > 0 && INTC_API_PREFIX(256_as_bool)(div_result.quot)) {
                insert_count++;
                if (insert_count % separate_every_n_chars == 0)
                    val.data[val.size++] = separate_ch;
            }
        } while (INTC_API_PREFIX(256_as_bool)(div_result.quot));
    }

    INTC_ASSERT(val.size <= (int)(sizeof(val.data) - 1));

    struct intc_u256_string result;
    result.size = 0;

    for (int i = val.size - 1; i >= 0; i--)
        result.data[result.size++] = val.data[i];
    result.data[result.size] = 0;

    return result;
}

INTC_API struct intc_u256_string INTC_API_PREFIX(256_int_str)(struct intc_u256 in, size_t separate_every_n_chars, char separate_ch)
{
    struct intc_u256_string result = INTC_API_PREFIX(256_str)(in, 10 /*base*/, separate_every_n_chars, separate_ch);
    return result;
}

INTC_API struct intc_u256_string INTC_API_PREFIX(256_readable_int_str)(struct intc_u256 in)
{
    struct intc_u256_string result = INTC_API_PREFIX(256_str)(in, 10 /*base*/, 3 /*separate_every_n_chars*/, ',');
    return result;
}

INTC_API struct intc_u256_string INTC_API_PREFIX(256_hex_str)(struct intc_u256 in, size_t separate_every_n_chars, char separate_ch)
{
    struct intc_u256_string result = INTC_API_PREFIX(256_str)(in, 16 /*base*/, separate_every_n_chars, separate_ch);
    return result;
}

INTC_API struct intc_u256_string INTC_API_PREFIX(256_readable_hex_str)(struct intc_u256 in)
{
    struct intc_u256_string result = INTC_API_PREFIX(256_str)(in, 16 /*base*/, 4 /*separate_ever_n_chars*/, '_' /*separate_ch*/);
    return result;
}
INTC_END_EXTERN_C

#if !defined(INTC_NO_CPP_FEATURES)
// NOTE: U256 CPP Bitwise ==========================================================================
INTC_API intc_u256 operator&(intc_u256 lhs, intc_u256 rhs)
{
    intc_u256 result = INTC_API_PREFIX(256_and)(lhs, rhs);
    return result;
}

INTC_API intc_u256 operator|(intc_u256 lhs, intc_u256 rhs)
{
    intc_u256 result = INTC_API_PREFIX(256_or)(lhs, rhs);
    return result;
}

INTC_API intc_u256 operator^(intc_u256 lhs, intc_u256 rhs)
{
    intc_u256 result = INTC_API_PREFIX(256_xor)(lhs, rhs);
    return result;
}

INTC_API intc_u256 operator~(intc_u256 lhs)
{
    intc_u256 result = INTC_API_PREFIX(256_negate)(lhs);
    return result;
}

INTC_API intc_u256 operator<<(intc_u256 lhs, unsigned shift)
{
    intc_u256 result = INTC_API_PREFIX(256_lshift)(lhs, shift);
    return result;
}

INTC_API intc_u256 operator>>(intc_u256 lhs, unsigned shift)
{
    intc_u256 result = INTC_API_PREFIX(256_rshift)(lhs, shift);
    return result;
}

// NOTE: U256 CPP Equality =========================================================================
INTC_API bool operator==(intc_u256 lhs, intc_u256 rhs)
{
    bool result = INTC_API_PREFIX(256_eq)(lhs, rhs);
    return result;
}

INTC_API bool operator!=(intc_u256 lhs, intc_u256 rhs)
{
    bool result = INTC_API_PREFIX(256_neq)(lhs, rhs);
    return result;
}

// NOTE: U256 CPP Relational =======================================================================
INTC_API bool operator>(intc_u256 lhs, intc_u256 rhs)
{
    bool result = INTC_API_PREFIX(256_gt)(lhs, rhs);
    return result;
}

INTC_API bool operator<(intc_u256 lhs, intc_u256 rhs)
{
    bool result = INTC_API_PREFIX(256_lt)(lhs, rhs);
    return result;
}

INTC_API bool operator>=(intc_u256 lhs, intc_u256 rhs)
{
    bool result = INTC_API_PREFIX(256_gt_eq)(lhs, rhs);
    return result;
}

INTC_API bool operator<=(intc_u256 lhs, intc_u256 rhs)
{
    bool result = INTC_API_PREFIX(256_lt_eq)(lhs, rhs);
    return result;
}

// NOTE: U256 CPP Arithmetic =======================================================================
INTC_API intc_u256 operator+(intc_u256 lhs, intc_u256 rhs)
{
    intc_u256 result = INTC_API_PREFIX(256_add)(lhs, rhs);
    return result;
}

INTC_API intc_u256 operator-(intc_u256 lhs, intc_u256 rhs)
{
    intc_u256 result = INTC_API_PREFIX(256_sub)(lhs, rhs);
    return result;
}

INTC_API intc_u256 operator*(intc_u256 lhs, intc_u256 rhs)
{
    intc_u256 result = INTC_API_PREFIX(256_mul)(lhs, rhs);
    return result;
}

INTC_API intc_u256 operator/(intc_u256 lhs, intc_u256 rhs)
{
    intc_u256 result = INTC_API_PREFIX(256_div)(lhs, rhs);
    return result;
}

INTC_API intc_u256 operator%(intc_u256 lhs, intc_u256 rhs)
{
    intc_u256 result = INTC_API_PREFIX(256_mod)(lhs, rhs);
    return result;
}

// NOTE: U256 CPP Other ============================================================================
INTC_API intc_u256 &operator&=(intc_u256 &lhs, intc_u256 rhs)
{
    lhs = lhs & rhs;
    return lhs;
}

INTC_API intc_u256 &operator|=(intc_u256 &lhs, intc_u256 rhs)
{
    lhs = lhs | rhs;
    return lhs;
}

INTC_API intc_u256 &operator^=(intc_u256 &lhs, intc_u256 rhs)
{
    lhs = lhs ^ rhs;
    return lhs;
}

INTC_API intc_u256 &operator<<=(intc_u256 &lhs, unsigned rhs)
{
    lhs = lhs << rhs;
    return lhs;
}

INTC_API intc_u256 &operator>>=(intc_u256 &lhs, unsigned rhs)
{
    lhs = lhs >> rhs;
    return lhs;
}

INTC_API intc_u256 &operator+=(intc_u256 &lhs, intc_u256 rhs)
{
    lhs = lhs + rhs;
    return lhs;
}

INTC_API intc_u256 &operator-=(intc_u256 &lhs, intc_u256 rhs)
{
    lhs = lhs - rhs;
    return lhs;
}

INTC_API intc_u256 &operator*=(intc_u256 &lhs, intc_u256 rhs)
{
    lhs = lhs * rhs;
    return lhs;
}

INTC_API intc_u256 &operator/=(intc_u256 &lhs, intc_u256 rhs)
{
    lhs = lhs / rhs;
    return lhs;
}

INTC_API intc_u256 &operator%=(intc_u256 &lhs, intc_u256 rhs)
{
    lhs = lhs % rhs;
    return lhs;
}

INTC_API intc_u256 &operator++(intc_u256 &lhs)
{
    lhs = lhs + 1;
    return lhs;
}

INTC_API intc_u256 &operator--(intc_u256 &lhs)
{
    lhs = lhs - 1;
    return lhs;
}

INTC_API intc_u256 operator++(intc_u256 &lhs, int)
{
    intc_u256 val = lhs;
    ++val;
    return val;
}

INTC_API intc_u256 operator--(intc_u256 &lhs, int)
{
    intc_u256 val = lhs;
    --val;
    return val;
}
#endif // !defined(INTC_NO_CPP_FEATURES)
#endif // !defined(INTC_NO_U256)
#endif // INTC_IMPLEMENTATION
