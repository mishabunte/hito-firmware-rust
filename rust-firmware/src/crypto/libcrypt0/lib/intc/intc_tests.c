#if !defined(INTC_TESTS_H)
#define INTC_TESTS_H

// NOTE: Overview ==================================================================================
// calccrypto's uint128/256 C++ tests converted into tests compatible with
// intc.h variations. This file compiles down and exposes a couple of functions
// that allow you to run the unit tests for "intc.h"
//
// This file can be compiled standalone by defining the macro
// INTC_TESTS_WITH_MAIN.
//
// NOTE: Configuration =============================================================================
// #define INTC_TESTS_IMPLEMENTATION
//     Define this in one and only one C++ file to enable the implementation
//     code of the header file.
//
// #define INTC_TESTS_WITH_MAIN
//     Define this macro to enable a main entry point function such that this
//     file can be compiled standalone into a runnable executable.
//
// #define INTC_TESTS_NO_COLORS
//     Define this macro to disable colors from the output of the unit tests.

#if defined(INTC_TESTS_WITH_MAIN)
    #if !defined(INTC_TESTS_IMPLEMENTATION)
        #define INTC_TESTS_IMPLEMENTATION
    #endif

    #define INTC_IMPLEMENTATION
    #include <intc.h>
#endif

#include <stdio.h>
#include <string.h>

// NOTE: Data Structures ===========================================================================
struct intc_test_state
{
    int test_count;
    int fail_count;
};

struct intc_test_case
{
    char const *name;
    int         name_size;
    bool        failed;
};

// NOTE: Functions =================================================================================
// Run the unit tests for intc_u128/u256 integer types.
struct intc_test_state intc_u128_unit_tests();

#if !defined(INTC_NO_U256)
struct intc_test_state intc_u256_unit_tests();
#endif

// Helper function that runs both tests and pretty prints the summary of the tests
void                   intc_unit_tests();

// Helper function to print out test state
void                   intc_test_state_print(char const *label, struct intc_test_state const *state);
#endif // INTC_TESTS_H

// -----------------------------------------------------------------------------
// NOTE: Test Data Declarations
// -----------------------------------------------------------------------------
#if defined(INTC_TESTS_IMPLEMENTATION)
struct intc_base_to_string {
    int base;
    char const *expect;
} const INTC_TESTS_STRING_BASE_TESTS[] = {
    {2,  "10000100000101011000010101101100"},
    {3,  "12201102210121112101"},
    {4,  "2010011120111230"},
    {5,  "14014244043144"},
    {6,  "1003520344444"},
    {7,  "105625466632"},
    {8,  "20405302554"},
    {9,  "5642717471"},
    {10, "2216002924"},
    {11, "a3796a883"},
    {12, "51a175124"},
    {13, "294145645"},
    {14, "170445352"},
    {15, "ce82d6d4"},
    {16, "8415856c"},
    {17, "56dc4e33"},
    {18, "3b2db13a"},
    {19, "291i3b4g"},
    {20, "1eca0764"},
    {21, "14hc96jg"},
    {22, "jblga9e"},
    {23, "em6i5a5"},
    {24, "be75374"},
    {25, "91mo4go"},
    {26, "74d74li"},
    {27, "5jblgea"},
    {28, "4gl7i9g"},
    {29, "3l13lor"},
    {30, "315o5e4"},
    {31, "2fcfub9"},
    {32, "221b1bc"},
    {33, "1nkji2p"},
    {34, "1eq93ik"},
    {35, "176p6y9"},
    {36, "10ncmss"}
};

intc_u64 const INTC_TESTS_MAX_UNSIGNED_VALUES[] = {
    (intc_u8)-1,
    (intc_u16)-1,
    (intc_u32)-1,
    (intc_u64)-1,
};

// -----------------------------------------------------------------------------
// NOTE: Testing Macros
// -----------------------------------------------------------------------------
#define INTC_TESTS_ASSERT_MSG(expr, fmt, ...)                                                                          \
    do                                                                                                                 \
    {                                                                                                                  \
        if (!(expr))                                                                                                   \
        {                                                                                                              \
            test_case_.failed = true;                                                                                  \
            char const *file  = intc_strip_path_to_file(__FILE__);                                                     \
            printf("+--Test failed at %s:%d, expression was: %s\n", file, __LINE__, #expr);                      \
            printf("V\n");                                                                                       \
            printf("|");                                                                                       \
            printf(fmt, __VA_ARGS__);                                                                                  \
            fputc('\n', stdout);                                                                                       \
        }                                                                                                              \
    } while (0)

#define INTC_TESTS_ASSERT(expr)                                                                                        \
    do                                                                                                                 \
    {                                                                                                                  \
        if (!(expr))                                                                                                   \
        {                                                                                                              \
            test_case_.failed = true;                                                                                  \
            char const *file  = intc_strip_path_to_file(__FILE__);                                                     \
            printf("+--Test failed at %s:%d, expression was: %s\n", file, __LINE__, #expr);                      \
            printf("V\n");                                                                                       \
        }                                                                                                              \
    } while (0)

#if defined(INTC_TESTS_NO_COLORS)
    #define INTC_TESTS_COLOR_RED
    #define INTC_TESTS_COLOR_GREEN
    #define INTC_TESTS_COLOR_MAGENTA
    #define INTC_TESTS_COLOR_RESET
#else
    #define INTC_TESTS_COLOR_RED "\x1b[31m"
    #define INTC_TESTS_COLOR_GREEN "\x1b[32m"
    #define INTC_TESTS_COLOR_MAGENTA "\x1b[35m"
    #define INTC_TESTS_COLOR_RESET "\x1b[0m"
#endif

#define INTC_TESTS_BEGIN(test_name)                                                                                    \
    struct intc_test_case test_case_ = INTC_ZERO_INIT;                                                                 \
    test_case_.name                  = test_name;                                                                      \
    test_case_.name_size             = sizeof(test_name) - 1;                                                          \
    test_case_.failed                = false;                                                                          \
    result.test_count++

#define INTC_TESTS_END                                                                                                 \
    fprintf(stdout, "    %s", test_case_.name);                                                                        \
    for (int index = 0; index < (64 - test_case_.name_size); index++)                                                  \
    {                                                                                                                  \
        if (index)                                                                                                     \
            fputc('.', stdout);                                                                                        \
        else                                                                                                           \
            fputc(' ', stdout);                                                                                        \
    }                                                                                                                  \
                                                                                                                       \
    fputc(' ', stdout);                                                                                                \
    if (test_case_.failed)                                                                                             \
    {                                                                                                                  \
        fprintf(stdout, INTC_TESTS_COLOR_RED "FAILED" INTC_TESTS_COLOR_RESET);                                         \
        result.fail_count++;                                                                                           \
    }                                                                                                                  \
    else                                                                                                               \
        fprintf(stdout, INTC_TESTS_COLOR_GREEN "OK" INTC_TESTS_COLOR_RESET);                                           \
    fputc('\n', stdout);

// -----------------------------------------------------------------------------
// NOTE: Implementation
// -----------------------------------------------------------------------------
static char *intc_strip_path_to_file(char const *file)
{
    int   size   = (int)strlen(file);
    char *result = (char *)file;
    for (int i = size - 1; i >= 0; i--)
    {
        if (file[i] == '\\' || file[i] == '/')
        {
            result = result + i + 1;
            break;
        }
    }

    return result;
}

struct intc_test_state intc_u128_unit_tests(void)
{
    struct intc_test_state result = INTC_ZERO_INIT;
    printf(INTC_TESTS_COLOR_MAGENTA "intc_u128 unit tests" INTC_TESTS_COLOR_RESET);
    printf("\n  accessors.cpp\n");
    {
        {
            INTC_TESTS_BEGIN("Accessor.bits");
            struct intc_u128 value = INTC_U64_TO_U128(1);
            for(int i = 0; i < 127; i++){
                INTC_TESTS_ASSERT(intc_u128_clz(value) == i + 1); // before shift
                value = intc_u128_lshift(value, 1);
            }

            INTC_TESTS_ASSERT(intc_u128_clz(INTC_U128_ZERO) == 0);
            INTC_TESTS_END;
        }

        {
            INTC_TESTS_BEGIN("Accessor.data");
            struct intc_u128 const value = INTC_U128(0x0123456789abcdefULL, 0xfedcba9876543210ULL);
            INTC_TESTS_ASSERT(value.hi == 0xfedcba9876543210ULL);
            INTC_TESTS_ASSERT(value.lo == 0x0123456789abcdefULL);
            INTC_TESTS_END;
        }
    }

    printf("\n  add.cpp\n");
    {
        {
            INTC_TESTS_BEGIN("Arithmetic.add");
            struct intc_u128 const low = INTC_U128(1, 0);
            struct intc_u128 const high = INTC_U128(0, 1);

            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_add(low, low), INTC_U64_TO_U128(2)));
            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_add(low, high), INTC_U128(1, 1)));
            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_add(high, high), INTC_U128(0, 2)));
            INTC_TESTS_END;
        }

        {
            INTC_TESTS_BEGIN("External.add");
            bool     t   = true;
            bool     f   = false;
            intc_u8  u8  = 0xaaULL;
            intc_u16 u16 = 0xaaaaULL;
            intc_u32 u32 = 0xaaaaaaaaULL;
            intc_u64 u64 = 0xaaaaaaaaaaaaaaaaULL;

            struct intc_u128 const val = INTC_U128(0xf0f0f0f0f0f0f0f0ULL, 0xf0f0f0f0f0f0f0f0ULL);

            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_add(INTC_U64_TO_U128(t)  , val), INTC_U128(0xf0f0f0f0f0f0f0f1ULL, 0xf0f0f0f0f0f0f0f0ULL)));
            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_add(INTC_U64_TO_U128(f)  , val), INTC_U128(0xf0f0f0f0f0f0f0f0ULL, 0xf0f0f0f0f0f0f0f0ULL)));
            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_add(INTC_U64_TO_U128(u8) , val), INTC_U128(0xf0f0f0f0f0f0f19aULL, 0xf0f0f0f0f0f0f0f0ULL)));
            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_add(INTC_U64_TO_U128(u16), val), INTC_U128(0xf0f0f0f0f0f19b9aULL, 0xf0f0f0f0f0f0f0f0ULL)));
            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_add(INTC_U64_TO_U128(u32), val), INTC_U128(0xf0f0f0f19b9b9b9aULL, 0xf0f0f0f0f0f0f0f0ULL)));
            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_add(INTC_U64_TO_U128(u64), val), INTC_U128(0x9b9b9b9b9b9b9b9aULL, 0xf0f0f0f0f0f0f0f1ULL)));

            INTC_TESTS_ASSERT(u8  += intc_u128_as_u8(val)   == (intc_u8)0x9aULL);
            INTC_TESTS_ASSERT(u16 += intc_u128_as_u16(val)  == (intc_u16)0x9b9aULL);
            INTC_TESTS_ASSERT(u32 += intc_u128_as_u32(val)  == (intc_u32)0x9b9b9b9aULL);
            INTC_TESTS_ASSERT(u64 += intc_u128_as_u64(val)  == (intc_u64)0x9b9b9b9b9b9b9b9aULL);
            INTC_TESTS_END;
        }
    }

    printf("\n  and.cpp\n");
    {
        {
            INTC_TESTS_BEGIN("BitWise.and");
            struct intc_u128 t   = INTC_U64_TO_U128((bool)true);
            struct intc_u128 f   = INTC_U64_TO_U128((bool)false);
            struct intc_u128 u8  = INTC_U64_TO_U128((intc_u8)0xaaULL);
            struct intc_u128 u16 = INTC_U64_TO_U128((intc_u16)0xaaaaULL);
            struct intc_u128 u32 = INTC_U64_TO_U128((intc_u32)0xaaaaaaaaULL);
            struct intc_u128 u64 = INTC_U64_TO_U128((intc_u64)0xaaaaaaaaaaaaaaaaULL);

            struct intc_u128 const val = INTC_U128(0xf0f0f0f0f0f0f0f0ULL, 0xf0f0f0f0f0f0f0f0ULL);

            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_and(t  , val), INTC_U64_TO_U128(0)));
            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_and(f  , val), INTC_U64_TO_U128(0)));
            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_and(u8 , val), INTC_U64_TO_U128(0xa0ULL)));
            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_and(u16, val), INTC_U64_TO_U128(0xa0a0ULL)));
            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_and(u32, val), INTC_U64_TO_U128(0xa0a0a0a0ULL)));
            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_and(u64, val), INTC_U64_TO_U128(0xa0a0a0a0a0a0a0a0ULL)));
            INTC_TESTS_END;
        }

        {
            INTC_TESTS_BEGIN("External.and");
            bool                   t   = true;
            bool                   f   = false;
            intc_u8                u8  = 0xaaULL;
            intc_u16               u16 = 0xaaaaULL;
            intc_u32               u32 = 0xaaaaaaaaULL;
            intc_u64               u64 = 0xaaaaaaaaaaaaaaaaULL;
            struct intc_u128 const val = INTC_U128(0xf0f0f0f0f0f0f0f0ULL, 0xf0f0f0f0f0f0f0f0ULL);

            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_and(INTC_U64_TO_U128(t)  , val), INTC_U64_TO_U128(0x0ULL)));
            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_and(INTC_U64_TO_U128(f)  , val), INTC_U64_TO_U128(0x0ULL)));
            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_and(INTC_U64_TO_U128(u8) , val), INTC_U64_TO_U128(0xa0ULL)));
            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_and(INTC_U64_TO_U128(u16), val), INTC_U64_TO_U128(0xa0a0ULL)));
            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_and(INTC_U64_TO_U128(u32), val), INTC_U64_TO_U128(0xa0a0a0a0ULL)));
            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_and(INTC_U64_TO_U128(u64), val), INTC_U64_TO_U128(0xa0a0a0a0a0a0a0a0ULL)));

            INTC_TESTS_ASSERT((bool)(t &= intc_u128_as_bool(val)) == true);
            INTC_TESTS_ASSERT((bool)(f &= intc_u128_as_bool(val)) == false);
            INTC_TESTS_ASSERT((u8  &= intc_u128_as_u8(val)) == (intc_u8)0xa0ULL);
            INTC_TESTS_ASSERT((u16 &= intc_u128_as_u16(val)) == (intc_u16)0xa0a0ULL);
            INTC_TESTS_ASSERT((u32 &= intc_u128_as_u32(val)) == (intc_u32)0xa0a0a0a0ULL);
            INTC_TESTS_ASSERT((u64 &= intc_u128_as_u64(val)) == (intc_u64)0xa0a0a0a0a0a0a0a0ULL);

            // zero
            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_and(INTC_U128_ZERO, val), INTC_U128_ZERO));
            INTC_TESTS_END;
        }
    }

    printf("\n  assignment.cpp\n");
    {
        {
            INTC_TESTS_BEGIN("Assignment.all");
            const struct intc_u128 t_1   = INTC_U64_TO_U128(true);
            const struct intc_u128 f_1   = INTC_U64_TO_U128(false);
            const struct intc_u128 u8_1  = INTC_U64_TO_U128(0x01);
            const struct intc_u128 u16_1 = INTC_U64_TO_U128(0x0123);
            const struct intc_u128 u32_1 = INTC_U64_TO_U128(0x01234567);
            const struct intc_u128 u64_1 = INTC_U64_TO_U128(0x0123456789abcdef);

            struct intc_u128 t_2   = INTC_U128_ZERO;
            struct intc_u128 f_2   = INTC_U128_ZERO;
            struct intc_u128 u8_2  = INTC_U128_ZERO;
            struct intc_u128 u16_2 = INTC_U128_ZERO;
            struct intc_u128 u32_2 = INTC_U128_ZERO;
            struct intc_u128 u64_2 = INTC_U128_ZERO;

            t_2   = t_1;
            f_2   = f_1;
            u8_2  = u8_1;
            u16_2 = u16_1;
            u32_2 = u32_1;
            u64_2 = u64_1;

            INTC_TESTS_ASSERT(intc_u128_eq(t_1, t_2));
            INTC_TESTS_ASSERT(intc_u128_eq(f_1, f_2));
            INTC_TESTS_ASSERT(intc_u128_eq(u8_1, u8_2));
            INTC_TESTS_ASSERT(intc_u128_eq(u16_1, u16_2));
            INTC_TESTS_ASSERT(intc_u128_eq(u32_1, u32_2));
            INTC_TESTS_ASSERT(intc_u128_eq(u64_1, u64_2));
            INTC_TESTS_END;
        }
    }

    printf("\n  constructor.cpp\n");
    {
        {
            INTC_TESTS_BEGIN("Constructor.standard");
            struct intc_u128       value = INTC_U64_TO_U128(0x0123456789abcdefULL);
            const struct intc_u128 original = value;

            INTC_TESTS_ASSERT(intc_u128_eq(INTC_U128_ZERO, INTC_U128_ZERO));
            INTC_TESTS_ASSERT(intc_u128_eq(value, original));
            INTC_TESTS_ASSERT(intc_u128_eq(value, INTC_U64_TO_U128(0x0123456789abcdefULL)));
            INTC_TESTS_END;
        }

        {
            INTC_TESTS_BEGIN("Constructor.one");
            intc_u64 a = INTC_U64_TO_U128(true).hi;
            intc_u64 b = INTC_U64_TO_U128(true).lo;
            intc_u64 c = INTC_U64_TO_U128(false).hi;
            intc_u64 d = INTC_U64_TO_U128(false).lo;

            intc_u64 e = INTC_U64_TO_U128((intc_u8)0x01ULL).hi;
            intc_u64 f = INTC_U64_TO_U128((intc_u16)0x0123ULL).hi;
            intc_u64 g = INTC_U64_TO_U128((intc_u32)0x01234567ULL).hi;
            intc_u64 h = INTC_U64_TO_U128((intc_u64)0x0123456789abcdefULL).hi;

            intc_u64 i = INTC_U64_TO_U128((intc_u8)0x01ULL).lo;
            intc_u64 j = INTC_U64_TO_U128((intc_u16)0x0123ULL).lo;
            intc_u64 k = INTC_U64_TO_U128((intc_u32)0x01234567ULL).lo;
            intc_u64 l = INTC_U64_TO_U128((intc_u64)0x0123456789abcdefULL).lo;

            INTC_TESTS_ASSERT(a == (intc_u64)false);
            INTC_TESTS_ASSERT(b == (intc_u64)true);
            INTC_TESTS_ASSERT(c == (intc_u64)false);
            INTC_TESTS_ASSERT(d == (intc_u64)false);

            INTC_TESTS_ASSERT(e == 0ULL);
            INTC_TESTS_ASSERT(f == 0ULL);
            INTC_TESTS_ASSERT(g == 0ULL);
            INTC_TESTS_ASSERT(h == 0ULL);

            INTC_TESTS_ASSERT(i == (intc_u8)0x01ULL);
            INTC_TESTS_ASSERT(j == (intc_u16)0x0123ULL);
            INTC_TESTS_ASSERT(k == (intc_u32)0x01234567ULL);
            INTC_TESTS_ASSERT(l == (intc_u64)0x0123456789abcdefULL);
            INTC_TESTS_END;
        }

        {
            INTC_TESTS_BEGIN("Constructor.two");
            for (int hi = 0; hi < 2; hi++)
            {
                for (int lo = 0; lo < 2; lo++)
                {
                    struct intc_u128 const val = INTC_U128((intc_u64)lo, (intc_u64)hi);
                    INTC_TESTS_ASSERT(val.hi == (intc_u64)hi);
                    INTC_TESTS_ASSERT(val.lo == (intc_u64)lo);
                }
            }

            intc_u64 a = INTC_U128((intc_u8)0x01ULL, (intc_u8)0x01ULL).hi;
            intc_u64 b = INTC_U128((intc_u16)0x0123ULL, (intc_u16)0x0123ULL).hi;
            intc_u64 c = INTC_U128((intc_u32)0x01234567ULL, (intc_u32)0x01234567ULL).hi;
            intc_u64 d = INTC_U128((intc_u64)0x0123456789abcdefULL, (intc_u64)0x0123456789abcdefULL).hi;

            intc_u64 e = INTC_U128((intc_u8)0x01ULL, (intc_u8)0x01ULL).lo;
            intc_u64 f = INTC_U128((intc_u16)0x0123ULL, (intc_u16)0x0123ULL).lo;
            intc_u64 g = INTC_U128((intc_u32)0x01234567ULL, (intc_u32)0x01234567ULL).lo;
            intc_u64 h = INTC_U128((intc_u64)0x0123456789abcdefULL, (intc_u64)0x0123456789abcdefULL).lo;

            INTC_TESTS_ASSERT(a == (intc_u8)0x01ULL);
            INTC_TESTS_ASSERT(b == (intc_u16)0x0123ULL);
            INTC_TESTS_ASSERT(c == (intc_u32)0x01234567ULL);
            INTC_TESTS_ASSERT(d == (intc_u64)0x0123456789abcdefULL);

            INTC_TESTS_ASSERT(e == (intc_u8)0x01ULL);
            INTC_TESTS_ASSERT(f == (intc_u16)0x0123ULL);
            INTC_TESTS_ASSERT(g == (intc_u32)0x01234567ULL);
            INTC_TESTS_ASSERT(h == (intc_u64)0x0123456789abcdefULL);
            INTC_TESTS_END;
        }

        {
            INTC_TESTS_BEGIN("Constructor.data");
            {
                char const string[]                      = "0xffffffffffffffffffffffffffffffff";
                struct intc_u128_init_result init_result = intc_u128_init_hex_cstring(string, sizeof(string) - 1, 0 /*separator*/);
                INTC_TESTS_ASSERT(init_result.success);
                INTC_TESTS_ASSERT_MSG(intc_u128_eq(init_result.value, INTC_U128_MAX),
                                     "val:    %s\n"
                                     "expect: %s\n",
                                     intc_u128_readable_hex_str(init_result.value).data,
                                     intc_u128_readable_hex_str(INTC_U128_MAX).data);
            }

            {
                char const string[]                      = "ffffffffffffffffffffffffffffffff";
                struct intc_u128_init_result init_result = intc_u128_init_hex_cstring(string, sizeof(string) - 1, 0 /*separator*/);
                INTC_TESTS_ASSERT(init_result.success);
                INTC_TESTS_ASSERT_MSG(intc_u128_eq(init_result.value, INTC_U128_MAX),
                                     "val:    %s\n"
                                     "expect: %s\n",
                                     intc_u128_readable_hex_str(init_result.value).data,
                                     intc_u128_readable_hex_str(INTC_U128_MAX).data);
            }

            {
                char const string[]                      = "0xffff'ffff'ffff'ffff'ffff'ffff'ffff'ffff";
                struct intc_u128_init_result init_result = intc_u128_init_hex_cstring(string, sizeof(string) - 1, '\'');
                INTC_TESTS_ASSERT(init_result.success);
                INTC_TESTS_ASSERT_MSG(intc_u128_eq(init_result.value, INTC_U128_MAX),
                                     "val:    %s\n"
                                     "expect: %s\n",
                                     intc_u128_readable_hex_str(init_result.value).data,
                                     intc_u128_readable_hex_str(INTC_U128_MAX).data);
            }

            {
                char const string[]                      = "ffff'ffff'ffff'ffff'ffff'ffff'ffff'ffff";
                struct intc_u128_init_result init_result = intc_u128_init_hex_cstring(string, sizeof(string) - 1, '\'');
                INTC_TESTS_ASSERT(init_result.success);
                INTC_TESTS_ASSERT_MSG(intc_u128_eq(init_result.value, INTC_U128_MAX),
                                     "val:    %s\n"
                                     "expect: %s\n",
                                     intc_u128_readable_hex_str(init_result.value).data,
                                     intc_u128_readable_hex_str(INTC_U128_MAX).data);
            }

            {
                char const string[]                      = "0xffff'ffff'ffff'ffff'ffff'ffff'ffff'ffff'f";
                struct intc_u128_init_result init_result = intc_u128_init_hex_cstring(string, sizeof(string) - 1, '\'');
                INTC_TESTS_ASSERT(!init_result.success);
            }

            {
                char const string[]                      = "0xffff'ffff'ffff'ffff'ffff'ffff'ffff'fffg";
                struct intc_u128_init_result init_result = intc_u128_init_hex_cstring(string, sizeof(string) - 1, '\'');
                INTC_TESTS_ASSERT(!init_result.success);
            }

            {
                char const string[]                      = "0x0";
                struct intc_u128_init_result init_result = intc_u128_init_hex_cstring(string, sizeof(string) - 1, 0 /*separator*/);
                INTC_TESTS_ASSERT_MSG(intc_u128_eq(init_result.value, INTC_U128_ZERO),
                                      "val: %s\n",
                                      intc_u128_readable_hex_str(init_result.value).data);
            }

            {
                char const       string[]                = "0x0123456789abcdef";
                struct intc_u128 expect                  = INTC_U64_TO_U128(0x0123456789abcdefULL);
                struct intc_u128_init_result init_result = intc_u128_init_hex_cstring(string, sizeof(string) - 1, 0 /*separator*/);
                INTC_TESTS_ASSERT_MSG(intc_u128_eq(init_result.value, expect),
                                     "val:    %s\n"
                                     "expect: %s\n",
                                     intc_u128_readable_hex_str(init_result.value).data,
                                     intc_u128_readable_hex_str(expect).data);
            }
            INTC_TESTS_END;
        }
    }

    printf("\n  div.cpp\n");
    {
        {
            INTC_TESTS_BEGIN("Arithmetic.divide");
            struct intc_u128 const big_val = INTC_U64_TO_U128(0xfedbca9876543210ULL);
            struct intc_u128 const small_val = INTC_U64_TO_U128(0xffffULL);

            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_div(small_val, small_val), INTC_U64_TO_U128(1)));
            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_div(small_val, big_val), INTC_U64_TO_U128(0)));

            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_div(big_val, big_val), INTC_U64_TO_U128(1)));

            // TODO(dqn): How do we want to do this ...
            // EXPECT_THROW(uint128_t(1) / uint128_t(0), std::domain_error);
            INTC_TESTS_END;
        }

        {
            INTC_TESTS_BEGIN("External.divide");
            bool     t   = true;
            bool     f   = false;
            intc_u8  u8  = 0xaaULL;
            intc_u16 u16 = 0xaaaaULL;
            intc_u32 u32 = 0xaaaaaaaaULL;
            intc_u64 u64 = 0xaaaaaaaaaaaaaaaaULL;

            struct intc_u128 const val = INTC_U64_TO_U128(0x7bULL);

            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_div(INTC_U64_TO_U128(t) , val),  INTC_U64_TO_U128(false)));
            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_div(INTC_U64_TO_U128(f) , val),  INTC_U64_TO_U128(false)));
            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_div(INTC_U64_TO_U128(u8), val),  INTC_U64_TO_U128(0x1ULL)));
            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_div(INTC_U64_TO_U128(u16), val), INTC_U64_TO_U128(0x163ULL)));
            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_div(INTC_U64_TO_U128(u32), val), INTC_U64_TO_U128(0x163356bULL)));
            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_div(INTC_U64_TO_U128(u64), val), INTC_U64_TO_U128(0x163356b88ac0de0ULL)));

            INTC_TESTS_ASSERT((intc_u8)(u8   /= intc_u128_as_u8(val))  ==   (intc_u8)0x1ULL);
            INTC_TESTS_ASSERT((intc_u16)(u16 /= intc_u128_as_u16(val)) == (intc_u16)0x163ULL);
            INTC_TESTS_ASSERT((intc_u32)(u32 /= intc_u128_as_u32(val)) == (intc_u32)0x163356bULL);
            INTC_TESTS_ASSERT((intc_u64)(u64 /= intc_u128_as_u64(val)) == (intc_u64)0x163356b88ac0de0ULL);
            INTC_TESTS_END;
        }
    }

    printf("\n  equals.cpp\n");
    {
        {
            INTC_TESTS_BEGIN("Comparison.equals");
            INTC_TESTS_ASSERT(intc_u128_eq(INTC_U64_TO_U128(0xdeadbeefULL), INTC_U64_TO_U128(0xdeadbeefULL)));
            INTC_TESTS_ASSERT(!intc_u128_eq(INTC_U64_TO_U128(0xdeadbeefULL), INTC_U64_TO_U128(0xfee1baadULL)));
            INTC_TESTS_END;
        }
    }

    printf("\n  fix.cpp\n");
    {
        {
            INTC_TESTS_BEGIN("Arithmetic.increment");
            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_add_u64(INTC_U128_ZERO, 1), INTC_U64_TO_U128(1)));
            INTC_TESTS_END;
        }

        {
            INTC_TESTS_BEGIN("Arithmetic.decrement");
            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_sub_u64(INTC_U128_ZERO, 1), INTC_U128(0xffffffffffffffffULL, 0xffffffffffffffffULL)));
            INTC_TESTS_END;
        }
    }

    printf("\n  functions.cpp\n");
    {
        {
            INTC_TESTS_BEGIN("Function.data");
            // make sure all of the test strings create the ASCII version of the string
            struct intc_u128 const original = INTC_U64_TO_U128(2216002924);
            for (int test_index = 0; test_index < (int)(sizeof(INTC_TESTS_STRING_BASE_TESTS)/sizeof(INTC_TESTS_STRING_BASE_TESTS[0])); test_index++)
            {
                struct intc_base_to_string const *test_entry = INTC_TESTS_STRING_BASE_TESTS + test_index;
                struct intc_u128_string output = intc_u128_str(original, test_entry->base, 0 /*separate_every_n_chars*/, ' ' /*separate_ch*/);
                INTC_TESTS_ASSERT_MSG(strcmp(output.data, test_entry->expect) == 0,
                                     "output: %s\n"
                                     "expect: %s\n",
                                     output.data,
                                     test_entry->expect);
            }
            INTC_TESTS_END;
        }
    }

    printf("\n  gt.cpp\n");
    {
        {
            INTC_TESTS_BEGIN("Comparison.greater_than");
            struct intc_u128 const big   = INTC_U128(0xffffffffffffffffULL, 0xffffffffffffffffULL);
            struct intc_u128 const small = INTC_U128(0x0000000000000000ULL, 0x0000000000000000ULL);

            INTC_TESTS_ASSERT(intc_u128_gt(small, small) == false);
            INTC_TESTS_ASSERT(intc_u128_gt(small, big) ==  false);

            INTC_TESTS_ASSERT(intc_u128_gt(big, small) == true);
            INTC_TESTS_ASSERT(intc_u128_gt(big, big) == false);
            INTC_TESTS_END;
        }

        {
            INTC_TESTS_BEGIN("External.greater_than");
            for (int index = 0;
                 index < (int)(sizeof(INTC_TESTS_MAX_UNSIGNED_VALUES)/sizeof(INTC_TESTS_MAX_UNSIGNED_VALUES[0]));
                 index++)
            {
                struct intc_u128 const small = INTC_U128_ZERO;
                struct intc_u128 const big   = INTC_U64_TO_U128(INTC_TESTS_MAX_UNSIGNED_VALUES[index]);

                INTC_TESTS_ASSERT(intc_u128_gt(small, small) == false);
                INTC_TESTS_ASSERT(intc_u128_gt(small, big) ==  false);

                INTC_TESTS_ASSERT(intc_u128_gt(big, small) == true);
                INTC_TESTS_ASSERT(intc_u128_gt(big, big) == false);
            }
            INTC_TESTS_END;
        }
    }

    printf("\n  gte.cpp\n");
    {
        {
            INTC_TESTS_BEGIN("Comparison.greater_than_or_equals");
            struct intc_u128 const big   = INTC_U128(0xffffffffffffffffULL, 0xffffffffffffffffULL);
            struct intc_u128 const small = INTC_U128(0x0000000000000000ULL, 0x0000000000000000ULL);

            INTC_TESTS_ASSERT(intc_u128_gt_eq(small, small) == true);
            INTC_TESTS_ASSERT(intc_u128_gt_eq(small, big)   ==  false);

            INTC_TESTS_ASSERT(intc_u128_gt_eq(big, small) == true);
            INTC_TESTS_ASSERT(intc_u128_gt_eq(big, big) == true);
            INTC_TESTS_END;
        }

        {
            INTC_TESTS_BEGIN("External.greater_than_or_equals");
            for (int index = 0;
                 index < (int)(sizeof(INTC_TESTS_MAX_UNSIGNED_VALUES)/sizeof(INTC_TESTS_MAX_UNSIGNED_VALUES[0]));
                 index++)
            {
                struct intc_u128 const small = INTC_U128_ZERO;
                struct intc_u128 const big   = INTC_U64_TO_U128(INTC_TESTS_MAX_UNSIGNED_VALUES[index]);

                INTC_TESTS_ASSERT(intc_u128_gt_eq(small, small) == true);
                INTC_TESTS_ASSERT(intc_u128_gt_eq(small, big) ==  false);

                INTC_TESTS_ASSERT(intc_u128_gt_eq(big, small) == true);
                INTC_TESTS_ASSERT(intc_u128_gt_eq(big, big) == true);
            }
            INTC_TESTS_END;
        }
    }

    printf("\n  invert.cpp\n");
    {
        {
            INTC_TESTS_BEGIN("BitWise.invert");
            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_negate(INTC_U128(0x0000000000000000ULL, 0x0000000000000000ULL)), INTC_U128(0xffffffffffffffffULL, 0xffffffffffffffffULL)));
            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_negate(INTC_U128(0x0000000000000000ULL, 0xffffffffffffffffULL)), INTC_U128(0xffffffffffffffffULL, 0x0000000000000000ULL)));
            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_negate(INTC_U128(0xffffffffffffffffULL, 0xffffffffffffffffULL)), INTC_U128(0x0000000000000000ULL, 0x0000000000000000ULL)));
            INTC_TESTS_END;
        }
    }

    printf("\n  leftshift.cpp\n");
    {
        {
            INTC_TESTS_BEGIN("BitShift.left");
            // operator<<
            struct intc_u128 val     = INTC_U64_TO_U128(0x1);
            intc_u64         exp_val = 1;
            for(int i = 0; i < 64; i++){
                INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_lshift(val, i), INTC_U64_TO_U128(exp_val << i)));
            }

            struct intc_u128 zero = INTC_U128_ZERO;
            for (int i = 0; i < 64; i++)
                INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_lshift(zero, i), INTC_U128_ZERO));
            INTC_TESTS_END;
        }

        {
            INTC_TESTS_BEGIN("External.shift_left");
            intc_u8                u8   = 0xffULL;
            intc_u16               u16  = 0xffffULL;
            intc_u32               u32  = 0xffffffff;
            intc_u64               u64  = 0xffffffffffffffff;

            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_lshift(INTC_U64_TO_U128(u8), 7),   INTC_U64_TO_U128(0x7f80ULL)));
            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_lshift(INTC_U64_TO_U128(u16), 15), INTC_U64_TO_U128(0x7fff8000ULL)));
            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_lshift(INTC_U64_TO_U128(u32), 31), INTC_U64_TO_U128(0x7fffffff80000000ULL)));
            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_lshift(INTC_U64_TO_U128(u64), 63), INTC_U128(0x8000000000000000ULL, 0x7fffffffffffffffULL)));
            INTC_TESTS_END;
        }
    }

    printf("\n  lt.cpp\n");
    {
        {
            INTC_TESTS_BEGIN("Comparison.less_than");
            struct intc_u128 const big   = INTC_U128(0xffffffffffffffffULL, 0xffffffffffffffffULL);
            struct intc_u128 const small = INTC_U128(0x0000000000000000ULL, 0x0000000000000000ULL);

            INTC_TESTS_ASSERT(intc_u128_lt(small, small) == false);
            INTC_TESTS_ASSERT(intc_u128_lt(small, big)   == true);

            INTC_TESTS_ASSERT(intc_u128_lt(big, small) == false);
            INTC_TESTS_ASSERT(intc_u128_lt(big, big)   == false);
            INTC_TESTS_END;
        }

        {
            INTC_TESTS_BEGIN("External.less_than");
            for (int index = 0;
                 index < (int)(sizeof(INTC_TESTS_MAX_UNSIGNED_VALUES)/sizeof(INTC_TESTS_MAX_UNSIGNED_VALUES[0]));
                 index++)
            {
                struct intc_u128 const small = INTC_U128_ZERO;
                struct intc_u128 const big   = INTC_U64_TO_U128(INTC_TESTS_MAX_UNSIGNED_VALUES[index]);

                INTC_TESTS_ASSERT(intc_u128_lt(small, small) == false);
                INTC_TESTS_ASSERT(intc_u128_lt(small, big)   == true);

                INTC_TESTS_ASSERT(intc_u128_lt(big, small) == false);
                INTC_TESTS_ASSERT(intc_u128_lt(big, big)   == false);
            }
            INTC_TESTS_END;
        }
    }

    printf("\n  lte.cpp\n");
    {
        {
            INTC_TESTS_BEGIN("Comparison.less_than_or_equals");
            struct intc_u128 const big   = INTC_U128(0xffffffffffffffffULL, 0xffffffffffffffffULL);
            struct intc_u128 const small = INTC_U128(0x0000000000000000ULL, 0x0000000000000000ULL);

            INTC_TESTS_ASSERT(intc_u128_lt_eq(small, small) == true);
            INTC_TESTS_ASSERT(intc_u128_lt_eq(small, big)   == true);

            INTC_TESTS_ASSERT(intc_u128_lt_eq(big, small) == false);
            INTC_TESTS_ASSERT(intc_u128_lt_eq(big, big)   == true);
            INTC_TESTS_END;
        }

        {
            INTC_TESTS_BEGIN("External.less_than_or_equals");
            for (int index = 0;
                 index < (int)(sizeof(INTC_TESTS_MAX_UNSIGNED_VALUES)/sizeof(INTC_TESTS_MAX_UNSIGNED_VALUES[0]));
                 index++)
            {
                struct intc_u128 const small = INTC_U128_ZERO;
                struct intc_u128 const big   = INTC_U64_TO_U128(INTC_TESTS_MAX_UNSIGNED_VALUES[index]);

                INTC_TESTS_ASSERT(intc_u128_lt_eq(small, small) == true);
                INTC_TESTS_ASSERT(intc_u128_lt_eq(small, big)   == true);

                INTC_TESTS_ASSERT(intc_u128_lt_eq(big, small) == false);
                INTC_TESTS_ASSERT(intc_u128_lt_eq(big, big)   == true);
            }
            INTC_TESTS_END;
        }
    }

    printf("\n  mod.cpp\n");
    {
        {
            INTC_TESTS_BEGIN("Arithmetic.modulo");
            // has remainder
            struct intc_u128 const val     = INTC_U128(0xffffffffffffffffULL, 0xffffffffffffffffULL);
            struct intc_u128 const val_mod = INTC_U64_TO_U128(0xfedcba9876543210ULL);

            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_mod(val, val_mod), INTC_U64_TO_U128(0x7f598f328cc265bfULL)));

            // no remainder
            struct intc_u128 const val_0 = INTC_U128(0, 0xfedcba9876543210);
            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_mod(val_0, val_mod), INTC_U128_ZERO));

            // TODO(dqn): Add a way to catch divide by 0 assert nicely? Maybe?
            // mod 0
            // EXPECT_THROW(uint128_t(1) % uint128_t(0), std::domain_error);
            INTC_TESTS_END;
        }

        {
            INTC_TESTS_BEGIN("External.modulo");
            bool                   t    = true;
            bool                   f    = false;
            intc_u8                u8   = 0xaaULL;
            intc_u16               u16  = 0xaaaaULL;
            intc_u32               u32  = 0xaaaaaaaaULL;
            intc_u64               u64  = 0xaaaaaaaaaaaaaaaaULL;
            struct intc_u128 const val  = INTC_U64_TO_U128(0xd03ULL); // prime

            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_mod(INTC_U64_TO_U128(t), val),   INTC_U64_TO_U128(true)));
            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_mod(INTC_U64_TO_U128(f), val),   INTC_U64_TO_U128(false)));
            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_mod(INTC_U64_TO_U128(u8), val),  INTC_U64_TO_U128((intc_u8) 0xaaULL)));
            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_mod(INTC_U64_TO_U128(u16), val), INTC_U64_TO_U128((intc_u16)0x183ULL)));
            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_mod(INTC_U64_TO_U128(u32), val), INTC_U64_TO_U128((intc_u32)0x249ULL)));
            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_mod(INTC_U64_TO_U128(u64), val), INTC_U64_TO_U128((intc_u64)0xc7fULL)));
            INTC_TESTS_END;
        }
    }

    printf("\n  mult.cpp\n");
    {
        {
            INTC_TESTS_BEGIN("Arithmetic.multiply");
            struct intc_u128 const val = INTC_U64_TO_U128(0xfedbca9876543210ULL);
            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_mul(val, val), INTC_U128(0x010e6cd7a44a4100ULL, 0xfdb8e2bacbfe7cefULL)));

            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_mul(val, INTC_U128_ZERO), INTC_U128_ZERO));
            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_mul(INTC_U128_ZERO, val), INTC_U128_ZERO));

            struct intc_u128 const one = INTC_U64_TO_U128(1);
            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_mul(val, one), val));
            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_mul(one, val), val));
            INTC_TESTS_END;
        }

        {
            INTC_TESTS_BEGIN("External.multiply");
            bool                   t    = true;
            bool                   f    = false;
            intc_u8                u8   = 0xaaULL;
            intc_u16               u16  = 0xaaaaULL;
            intc_u32               u32  = 0xaaaaaaaaULL;
            intc_u64               u64  = 0xaaaaaaaaaaaaaaaaULL;
            struct intc_u128 const val  = INTC_U128(0xf0f0f0f0f0f0f0f0ULL, 0xf0f0f0f0f0f0f0f0ULL);

            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_mul(INTC_U64_TO_U128(t), val),     INTC_U128(0xf0f0f0f0f0f0f0f0ULL, 0xf0f0f0f0f0f0f0f0ULL)));
            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_mul(INTC_U64_TO_U128(f), val),     INTC_U128(0x0000000000000000ULL, 0x0000000000000000ULL)));
            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_mul(INTC_U64_TO_U128(u8), val),    INTC_U128(0xffffffffffffff60ULL, 0xffffffffffffffffULL)));
            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_mul(INTC_U64_TO_U128(u16), val),   INTC_U128(0xffffffffffff5f60ULL, 0xffffffffffffffffULL)));
            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_mul(INTC_U64_TO_U128(u32), val),   INTC_U128(0xffffffff5f5f5f60ULL, 0xffffffffffffffffULL)));
            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_mul(INTC_U64_TO_U128(u64), val),   INTC_U128(0x5f5f5f5f5f5f5f60ULL, 0xffffffffffffffffULL)));

            INTC_TESTS_ASSERT((intc_u8)(u8   * intc_u128_as_u8(val))  == (intc_u8)0x60ULL);
            INTC_TESTS_ASSERT((intc_u16)(u16 * intc_u128_as_u16(val)) == (intc_u16)0x5f60ULL);
            INTC_TESTS_ASSERT((intc_u32)(u32 * intc_u128_as_u32(val)) == (intc_u32)0x5f5f5f60ULL);
            INTC_TESTS_ASSERT((intc_u64)(u64 * intc_u128_as_u64(val)) == (intc_u64)0x5f5f5f5f5f5f5f60ULL);
            INTC_TESTS_END;
        }
    }

    printf("\n  notequals.cpp\n");
    {
        {
            INTC_TESTS_BEGIN("Comparison.not_equals");
            INTC_TESTS_ASSERT(intc_u128_neq(INTC_U64_TO_U128(0xdeadbeefULL), INTC_U64_TO_U128(0xdeadbeefULL)) == false);
            INTC_TESTS_ASSERT(intc_u128_neq(INTC_U64_TO_U128(0xdeadbeefULL), INTC_U64_TO_U128(0xfee1baadULL)) == true);
            INTC_TESTS_END;
        }

        {
            INTC_TESTS_BEGIN("External.not_equals");
            bool     const t   = true;
            bool     const f   = false;
            intc_u8  const u8  = 0xaaULL;
            intc_u16 const u16 = 0xaaaaULL;
            intc_u32 const u32 = 0xaaaaaaaaULL;
            intc_u64 const u64 = 0xaaaaaaaaaaaaaaaaULL;

            INTC_TESTS_ASSERT(intc_u128_neq(INTC_U64_TO_U128(t)  , INTC_U64_TO_U128(f))   == true);
            INTC_TESTS_ASSERT(intc_u128_neq(INTC_U64_TO_U128(f)  , INTC_U64_TO_U128(t))   == true);
            INTC_TESTS_ASSERT(intc_u128_neq(INTC_U64_TO_U128(u8) , INTC_U64_TO_U128(u64)) == true);
            INTC_TESTS_ASSERT(intc_u128_neq(INTC_U64_TO_U128(u16), INTC_U64_TO_U128(u32)) == true);
            INTC_TESTS_ASSERT(intc_u128_neq(INTC_U64_TO_U128(u32), INTC_U64_TO_U128(u16)) == true);
            INTC_TESTS_ASSERT(intc_u128_neq(INTC_U64_TO_U128(u64), INTC_U64_TO_U128(u8))  == true);

            INTC_TESTS_ASSERT(intc_u128_neq(INTC_U64_TO_U128(t)  , INTC_U64_TO_U128(t))   == false);
            INTC_TESTS_ASSERT(intc_u128_neq(INTC_U64_TO_U128(f)  , INTC_U64_TO_U128(f))   == false);
            INTC_TESTS_ASSERT(intc_u128_neq(INTC_U64_TO_U128(u8) , INTC_U64_TO_U128(u8))  == false);
            INTC_TESTS_ASSERT(intc_u128_neq(INTC_U64_TO_U128(u16), INTC_U64_TO_U128(u16)) == false);
            INTC_TESTS_ASSERT(intc_u128_neq(INTC_U64_TO_U128(u32), INTC_U64_TO_U128(u32)) == false);
            INTC_TESTS_ASSERT(intc_u128_neq(INTC_U64_TO_U128(u64), INTC_U64_TO_U128(u64)) == false);
            INTC_TESTS_END;
        }
    }

    printf("\n  or.cpp\n");
    {
        {
            INTC_TESTS_BEGIN("BitWise.or");
            struct intc_u128 const t   = INTC_U64_TO_U128((bool)     true);
            struct intc_u128 const f   = INTC_U64_TO_U128((bool)     false);
            struct intc_u128 const u8  = INTC_U64_TO_U128((intc_u8)  0xaaULL);
            struct intc_u128 const u16 = INTC_U64_TO_U128((intc_u16) 0xaaaaULL);
            struct intc_u128 const u32 = INTC_U64_TO_U128((intc_u32) 0xaaaaaaaaULL);
            struct intc_u128 const u64 = INTC_U64_TO_U128((intc_u64) 0xaaaaaaaaaaaaaaaaULL);
            struct intc_u128 const val = INTC_U128(0xf0f0f0f0f0f0f0f0ULL, 0xf0f0f0f0f0f0f0f0ULL);

            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_or(t  , val), INTC_U128(0xf0f0f0f0f0f0f0f1ULL, 0xf0f0f0f0f0f0f0f0ULL)));
            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_or(f  , val), INTC_U128(0xf0f0f0f0f0f0f0f0ULL, 0xf0f0f0f0f0f0f0f0ULL)));
            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_or(u8 , val), INTC_U128(0xf0f0f0f0f0f0f0faULL, 0xf0f0f0f0f0f0f0f0ULL)));
            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_or(u16, val), INTC_U128(0xf0f0f0f0f0f0fafaULL, 0xf0f0f0f0f0f0f0f0ULL)));
            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_or(u32, val), INTC_U128(0xf0f0f0f0fafafafaULL, 0xf0f0f0f0f0f0f0f0ULL)));
            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_or(u64, val), INTC_U128(0xfafafafafafafafaULL, 0xf0f0f0f0f0f0f0f0ULL)));

            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_or(INTC_U128_ZERO, val), val));
            INTC_TESTS_END;
        }

        {
            INTC_TESTS_BEGIN("External.or");
            bool     t   = true;
            bool     f   = false;
            intc_u8  u8  = 0xaa;
            intc_u16 u16 = 0xaaaa;
            intc_u32 u32 = 0xaaaaaaaaULL;
            intc_u64 u64 = 0xaaaaaaaaaaaaaaaaULL;
            struct intc_u128 const val = INTC_U128(0xf0f0f0f0f0f0f0f0ULL, 0xf0f0f0f0f0f0f0f0ULL);

            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_or(INTC_U64_TO_U128(t)  , val), INTC_U128(0xf0f0f0f0f0f0f0f1ULL, 0xf0f0f0f0f0f0f0f0ULL)));
            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_or(INTC_U64_TO_U128(f)  , val), INTC_U128(0xf0f0f0f0f0f0f0f0ULL, 0xf0f0f0f0f0f0f0f0ULL)));
            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_or(INTC_U64_TO_U128(u8) , val), INTC_U128(0xf0f0f0f0f0f0f0faULL, 0xf0f0f0f0f0f0f0f0ULL)));
            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_or(INTC_U64_TO_U128(u16), val), INTC_U128(0xf0f0f0f0f0f0fafaULL, 0xf0f0f0f0f0f0f0f0ULL)));
            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_or(INTC_U64_TO_U128(u32), val), INTC_U128(0xf0f0f0f0fafafafaULL, 0xf0f0f0f0f0f0f0f0ULL)));
            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_or(INTC_U64_TO_U128(u64), val), INTC_U128(0xfafafafafafafafaULL, 0xf0f0f0f0f0f0f0f0ULL)));
            INTC_TESTS_END;
        }
    }

    printf("\n  rightshift.cpp\n");
    {
        {
            INTC_TESTS_BEGIN("BitShift.right");
            // operator>>
            struct intc_u128 val = INTC_U64_TO_U128(0xffffffffffffffffULL);
            intc_u64         exp = 0xffffffffffffffffULL;

            for(int i = 0; i < 64; i++){
                INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_rshift(val, i), INTC_U64_TO_U128(exp >> i)));
            }

            struct intc_u128 zero = INTC_U128_ZERO;
            for (int i = 0; i < 64; i++)
                INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_rshift(zero, i), INTC_U128_ZERO));
            INTC_TESTS_END;
        }

        {
            INTC_TESTS_BEGIN("External.shift_left");
            bool     t   = true;
            bool     f   = false;
            intc_u8  u8  = 0xffULL;
            intc_u16 u16 = 0xffffULL;
            intc_u32 u32 = 0xffffffffULL;
            intc_u64 u64 = 0xffffffffffffffffULL;

            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_rshift(INTC_U64_TO_U128(t) , 0), INTC_U64_TO_U128(1)));
            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_rshift(INTC_U64_TO_U128(f) , 0), INTC_U128_ZERO));
            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_rshift(INTC_U64_TO_U128(u8), 0),  INTC_U64_TO_U128(u8)));
            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_rshift(INTC_U64_TO_U128(u16), 0), INTC_U64_TO_U128(u16)));
            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_rshift(INTC_U64_TO_U128(u32), 0), INTC_U64_TO_U128(u32)));
            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_rshift(INTC_U64_TO_U128(u64), 0), INTC_U64_TO_U128(u64)));

            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_rshift(INTC_U64_TO_U128(t), 1),   INTC_U64_TO_U128((intc_u64)t   >> 1)));
            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_rshift(INTC_U64_TO_U128(f), 1),   INTC_U64_TO_U128((intc_u64)f   >> 1)));
            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_rshift(INTC_U64_TO_U128(u8), 1),  INTC_U64_TO_U128((intc_u64)u8  >> 1)));
            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_rshift(INTC_U64_TO_U128(u16), 1), INTC_U64_TO_U128((intc_u64)u16 >> 1)));
            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_rshift(INTC_U64_TO_U128(u32), 1), INTC_U64_TO_U128((intc_u64)u32 >> 1)));
            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_rshift(INTC_U64_TO_U128(u64), 1), INTC_U64_TO_U128((intc_u64)u64 >> 1)));

            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_rshift(INTC_U64_TO_U128(u8), 7),   INTC_U64_TO_U128(1)));
            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_rshift(INTC_U64_TO_U128(u16), 15), INTC_U64_TO_U128(1)));
            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_rshift(INTC_U64_TO_U128(u32), 31), INTC_U64_TO_U128(1)));
            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_rshift(INTC_U64_TO_U128(u64), 63), INTC_U64_TO_U128(1)));
            INTC_TESTS_END;
        }
    }

    printf("\n  sub.cpp\n");
    {
        {
            INTC_TESTS_BEGIN("Arithmetic.subtract");
            struct intc_u128 const big   = INTC_U128(0xffffffffffffffffULL, 0xffffffffffffffffULL);
            struct intc_u128 const small = INTC_U128(0x0000000000000001ULL, 0x0000000000000000ULL);

            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_sub(small, small), INTC_U128_ZERO));
            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_sub(small, big),   INTC_U128(0x0000000000000002ULL, 0x0000000000000000ULL)));
            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_sub(big  , small), INTC_U128(0xfffffffffffffffeULL, 0xffffffffffffffffULL)));
            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_sub(big  , big),   INTC_U128_ZERO));
            INTC_TESTS_END;
        }

        {
            INTC_TESTS_BEGIN("External.subtract");
            bool             t    = true;
            bool             f    = false;
            intc_u8          u8   = 0xaaULL;
            intc_u16         u16  = 0xaaaaULL;
            intc_u32         u32  = 0xaaaaaaaaULL;
            intc_u64         u64  = 0xaaaaaaaaaaaaaaaaULL;
            struct intc_u128 val  = INTC_U128(0xf0f0f0f0f0f0f0f0ULL, 0xf0f0f0f0f0f0f0f0ULL);

            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_sub(INTC_U64_TO_U128(t)  , val),   INTC_U128(0x0f0f0f0f0f0f0f11ULL, 0x0f0f0f0f0f0f0f0fULL)));
            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_sub(INTC_U64_TO_U128(f)  , val),   INTC_U128(0x0f0f0f0f0f0f0f10ULL, 0x0f0f0f0f0f0f0f0fULL)));
            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_sub(INTC_U64_TO_U128(u8) , val),   INTC_U128(0x0f0f0f0f0f0f0fbaULL, 0x0f0f0f0f0f0f0f0fULL)));
            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_sub(INTC_U64_TO_U128(u16), val),   INTC_U128(0x0f0f0f0f0f0fb9baULL, 0x0f0f0f0f0f0f0f0fULL)));
            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_sub(INTC_U64_TO_U128(u32), val),   INTC_U128(0x0f0f0f0fb9b9b9baULL, 0x0f0f0f0f0f0f0f0fULL)));
            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_sub(INTC_U64_TO_U128(u64), val),   INTC_U128(0xb9b9b9b9b9b9b9baULL, 0x0f0f0f0f0f0f0f0fULL)));
            INTC_TESTS_END;
        }
    }

    printf("\n  typecast.cpp\n");
    {
        {
            INTC_TESTS_BEGIN("Typecast.all");
            struct intc_u128 const val = INTC_U128(0xaaaaaaaaaaaaaaaaULL, 0xaaaaaaaaaaaaaaaaULL);
            INTC_TESTS_ASSERT(intc_u128_as_bool(INTC_U64_TO_U128(true)) == true);
            INTC_TESTS_ASSERT(intc_u128_as_bool(INTC_U64_TO_U128(false)) == false);
            INTC_TESTS_ASSERT(intc_u128_as_u8(val)  == (intc_u8)0xaaULL);
            INTC_TESTS_ASSERT(intc_u128_as_u16(val) == (intc_u16)0xaaaaULL);
            INTC_TESTS_ASSERT(intc_u128_as_u32(val) == (intc_u32)0xaaaaaaaaULL);
            INTC_TESTS_ASSERT(intc_u128_as_u64(val) == (intc_u64)0xaaaaaaaaaaaaaaaaULL);
            INTC_TESTS_END;
        }
    }

    printf("\n  xor.cpp\n");
    {
        {
            INTC_TESTS_BEGIN("BitWise.xor");
            struct intc_u128 t   = INTC_U64_TO_U128((bool)     true);
            struct intc_u128 f   = INTC_U64_TO_U128((bool)     false);
            struct intc_u128 u8  = INTC_U64_TO_U128((intc_u8)  0xaaULL);
            struct intc_u128 u16 = INTC_U64_TO_U128((intc_u16) 0xaaaaULL);
            struct intc_u128 u32 = INTC_U64_TO_U128((intc_u32) 0xaaaaaaaaULL);
            struct intc_u128 u64 = INTC_U64_TO_U128((intc_u64) 0xaaaaaaaaaaaaaaaa);
            struct intc_u128 const val = INTC_U128(0xf0f0f0f0f0f0f0f0ULL, 0xf0f0f0f0f0f0f0f0ULL);

            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_xor(t  ,  val), INTC_U128(0xf0f0f0f0f0f0f0f1ULL, 0xf0f0f0f0f0f0f0f0ULL)));
            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_xor(f  ,  val), INTC_U128(0xf0f0f0f0f0f0f0f0ULL, 0xf0f0f0f0f0f0f0f0ULL)));
            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_xor(u8 ,  val), INTC_U128(0xf0f0f0f0f0f0f05aULL, 0xf0f0f0f0f0f0f0f0ULL)));
            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_xor(u16,  val), INTC_U128(0xf0f0f0f0f0f05a5aULL, 0xf0f0f0f0f0f0f0f0ULL)));
            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_xor(u32,  val), INTC_U128(0xf0f0f0f05a5a5a5aULL, 0xf0f0f0f0f0f0f0f0ULL)));
            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_xor(u64,  val), INTC_U128(0x5a5a5a5a5a5a5a5aULL, 0xf0f0f0f0f0f0f0f0ULL)));

            // zero
            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_xor(INTC_U128_ZERO, val), val));
            INTC_TESTS_END;
        }

        {
            INTC_TESTS_BEGIN("External.xor");
            bool     t   = true;
            bool     f   = false;
            intc_u8  u8  = 0xaaULL;
            intc_u16 u16 = 0xaaaaULL;
            intc_u32 u32 = 0xaaaaaaaaULL;
            intc_u64 u64 = 0xaaaaaaaaaaaaaaaaULL;
            struct intc_u128 const val = INTC_U128(0xf0f0f0f0f0f0f0f0ULL, 0xf0f0f0f0f0f0f0f0ULL);

            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_xor(INTC_U64_TO_U128(t  ), val), INTC_U128(0xf0f0f0f0f0f0f0f1ULL, 0xf0f0f0f0f0f0f0f0ULL)));
            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_xor(INTC_U64_TO_U128(f  ), val), INTC_U128(0xf0f0f0f0f0f0f0f0ULL, 0xf0f0f0f0f0f0f0f0ULL)));
            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_xor(INTC_U64_TO_U128(u8 ), val), INTC_U128(0xf0f0f0f0f0f0f05aULL, 0xf0f0f0f0f0f0f0f0ULL)));
            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_xor(INTC_U64_TO_U128(u16), val), INTC_U128(0xf0f0f0f0f0f05a5aULL, 0xf0f0f0f0f0f0f0f0ULL)));
            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_xor(INTC_U64_TO_U128(u32), val), INTC_U128(0xf0f0f0f05a5a5a5aULL, 0xf0f0f0f0f0f0f0f0ULL)));
            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_xor(INTC_U64_TO_U128(u64), val), INTC_U128(0x5a5a5a5a5a5a5a5aULL, 0xf0f0f0f0f0f0f0f0ULL)));
            INTC_TESTS_END;
        }
    }

    fputc('\n', stdout);
    return result;
}

#if !defined(INTC_NO_U256)
struct intc_test_state intc_u256_unit_tests(void)
{
    struct intc_test_state result = INTC_ZERO_INIT;
    printf(INTC_TESTS_COLOR_MAGENTA "intc_u256 unit tests" INTC_TESTS_COLOR_RESET);
    printf("\n  accessors.cpp\n");
    {
        {
            INTC_TESTS_BEGIN("Accessor.bits");
            struct intc_u256 value = INTC_U64_TO_U256(1);
            for (int i = 0; i < 127; i++)
            {
                INTC_TESTS_ASSERT(intc_u256_clz(value) == (i + 1));
                value = intc_u256_lshift(value, 1);
            }

            INTC_TESTS_ASSERT(intc_u256_clz(INTC_U256_ZERO) == 0);
            INTC_TESTS_END;
        }

        {
            INTC_TESTS_BEGIN("Accessor.data");
            const struct intc_u256 value = INTC_U256(INTC_U128(0x0123456789abcdefULL, 0xfedcba9876543210ULL), INTC_U128(0x0123456789abcdefULL, 0xfedcba9876543210ULL));
            INTC_TESTS_ASSERT_MSG(value.hi.hi == 0xfedcba9876543210ULL, "hi.hi: %llx\n", value.hi.hi);
            INTC_TESTS_ASSERT_MSG(value.hi.lo == 0x0123456789abcdefULL, "hi.lo: %llx\n", value.hi.lo);
            INTC_TESTS_ASSERT_MSG(value.lo.hi == 0xfedcba9876543210ULL, "lo.hi: %llx\n", value.lo.hi);
            INTC_TESTS_ASSERT_MSG(value.lo.lo == 0x0123456789abcdefULL, "lo.lo: %llx\n", value.lo.lo);
            INTC_TESTS_END;
        }
    }

    printf("\n  add.cpp\n");
    {
        {
            INTC_TESTS_BEGIN("Arithmetic.add");
            struct intc_u256 low  = INTC_U256(1, 0);
            struct intc_u256 high = INTC_U256(0, 1);

            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_add(low, low), INTC_U64_TO_U256(2)));
            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_add(low, high), INTC_U256(1, 1)));
            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_add(high, high), INTC_U256(0, 2)));
            INTC_TESTS_END;
        }

        {
            INTC_TESTS_BEGIN("External.add");
            bool             t    = true;
            bool             f    = false;
            intc_u8          u8   = 0xaaULL;
            intc_u16         u16  = 0xaaaaULL;
            intc_u32         u32  = 0xaaaaaaaaULL;
            intc_u64         u64  = 0xaaaaaaaaaaaaaaaaULL;
            struct intc_u128 u128 = INTC_U128(0xaaaaaaaaaaaaaaaaULL, 0xaaaaaaaaaaaaaaaaULL);
            struct intc_u256 val = INTC_U256(INTC_U128(0xf0f0f0f0f0f0f0f0ULL, 0xf0f0f0f0f0f0f0f0ULL), INTC_U128(0xf0f0f0f0f0f0f0f0ULL, 0xf0f0f0f0f0f0f0f0ULL));

            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_add_u64(val, t  ), INTC_U256(INTC_U128(0xf0f0f0f0f0f0f0f1ULL, 0xf0f0f0f0f0f0f0f0ULL), INTC_U128(0xf0f0f0f0f0f0f0f0ULL,  0xf0f0f0f0f0f0f0f0ULL))));
            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_add_u64(val, f  ), INTC_U256(INTC_U128(0xf0f0f0f0f0f0f0f0ULL, 0xf0f0f0f0f0f0f0f0ULL), INTC_U128(0xf0f0f0f0f0f0f0f0ULL,  0xf0f0f0f0f0f0f0f0ULL))));
            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_add_u64(val, u8 ), INTC_U256(INTC_U128(0xf0f0f0f0f0f0f19aULL, 0xf0f0f0f0f0f0f0f0ULL), INTC_U128(0xf0f0f0f0f0f0f0f0ULL,  0xf0f0f0f0f0f0f0f0ULL))));
            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_add_u64(val, u16), INTC_U256(INTC_U128(0xf0f0f0f0f0f19b9aULL, 0xf0f0f0f0f0f0f0f0ULL), INTC_U128(0xf0f0f0f0f0f0f0f0ULL,  0xf0f0f0f0f0f0f0f0ULL))));
            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_add_u64(val, u32), INTC_U256(INTC_U128(0xf0f0f0f19b9b9b9aULL, 0xf0f0f0f0f0f0f0f0ULL), INTC_U128(0xf0f0f0f0f0f0f0f0ULL,  0xf0f0f0f0f0f0f0f0ULL))));
            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_add_u64(val, u64), INTC_U256(INTC_U128(0x9b9b9b9b9b9b9b9aULL, 0xf0f0f0f0f0f0f0f1ULL), INTC_U128(0xf0f0f0f0f0f0f0f0ULL,  0xf0f0f0f0f0f0f0f0ULL))));
            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_add(val, INTC_U128_TO_U256(u128)), INTC_U256(INTC_U128(0x9b9b9b9b9b9b9b9aULL, 0x9b9b9b9b9b9b9b9bULL), INTC_U128(0xf0f0f0f0f0f0f0f1ULL,  0xf0f0f0f0f0f0f0f0ULL))));

            INTC_TESTS_ASSERT((bool)    (t   + intc_u256_as_bool(val)) == true);
            INTC_TESTS_ASSERT((bool)    (f   + intc_u256_as_bool(val)) == true);
            INTC_TESTS_ASSERT((intc_u8) (u8  + intc_u256_as_u8  (val)) == (intc_u8) 0x9aULL);
            INTC_TESTS_ASSERT((intc_u16)(u16 + intc_u256_as_u16 (val)) == (intc_u16)0x9b9aULL);
            INTC_TESTS_ASSERT((intc_u32)(u32 + intc_u256_as_u32 (val)) == (intc_u32)0x9b9b9b9aULL);
            INTC_TESTS_ASSERT((intc_u64)(u64 + intc_u256_as_u64 (val)) == (intc_u64)0x9b9b9b9b9b9b9b9aULL);
            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_add(u128, intc_u256_as_u128(val)), INTC_U128(0x9b9b9b9b9b9b9b9aULL, 0x9b9b9b9b9b9b9b9bULL)));
            INTC_TESTS_END;
        }
    }

    printf("\n  and.cpp\n");
    {
        {
            INTC_TESTS_BEGIN("Bitwise.and");
            const struct intc_u256 t   = INTC_U64_TO_U256((bool)     true);
            const struct intc_u256 f   = INTC_U64_TO_U256((bool)     false);
            const struct intc_u256 u8  = INTC_U64_TO_U256((intc_u8)  0xaaULL);
            const struct intc_u256 u16 = INTC_U64_TO_U256((intc_u16) 0xaaaaULL);
            const struct intc_u256 u32 = INTC_U64_TO_U256((intc_u32) 0xaaaaaaaaULL);
            const struct intc_u256 u64 = INTC_U64_TO_U256((intc_u64) 0xaaaaaaaaaaaaaaaaULL);
            const struct intc_u256 val = INTC_U128_TO_U256(INTC_U128(0xf0f0f0f0f0f0f0f0ULL, 0xf0f0f0f0f0f0f0f0ULL));

            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_and(t,   val), INTC_U64_TO_U256(0)));
            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_and(f,   val), INTC_U64_TO_U256(0)));
            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_and(u8,  val), INTC_U64_TO_U256(0xa0ULL)));
            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_and(u16, val), INTC_U64_TO_U256(0xa0a0ULL)));
            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_and(u32, val), INTC_U64_TO_U256(0xa0a0a0a0ULL)));
            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_and(u64, val), INTC_U64_TO_U256(0xa0a0a0a0a0a0a0a0ULL)));
            INTC_TESTS_END;
        }

        {
            INTC_TESTS_BEGIN("External.and");
            bool                   t   = true;
            bool                   f   = false;
            intc_u8                u8  = 0xaaULL;
            intc_u16               u16 = 0xaaaaULL;
            intc_u32               u32 = 0xaaaaaaaaULL;
            intc_u64               u64 = 0xaaaaaaaaaaaaaaaaULL;
            const struct intc_u256 val = INTC_U256(0xf0f0f0f0f0f0f0f0ULL, 0xf0f0f0f0f0f0f0f0ULL);

            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_and(INTC_U64_TO_U256(t  ), val), INTC_U64_TO_U256(0)));
            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_and(INTC_U64_TO_U256(f  ), val), INTC_U64_TO_U256(0)));
            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_and(INTC_U64_TO_U256(u8 ), val), INTC_U64_TO_U256(0xa0ULL)));
            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_and(INTC_U64_TO_U256(u16), val), INTC_U64_TO_U256(0xa0a0ULL)));
            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_and(INTC_U64_TO_U256(u32), val), INTC_U64_TO_U256(0xa0a0a0a0ULL)));
            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_and(INTC_U64_TO_U256(u64), val), INTC_U64_TO_U256(0xa0a0a0a0a0a0a0a0ULL)));

            // TODO: Why is this test result different from calccrypto and intc?
            //
            // true &= (0xF0F0F0 ...) == true?
            //
            // Original test line was
            //
            // EXPECT_EQ(t   &= val, false);
            //
            INTC_TESTS_ASSERT((bool)    (t   &= intc_u256_as_bool(val)) == true);

            INTC_TESTS_ASSERT((bool)    (f   &= intc_u256_as_bool(val)) == false);
            INTC_TESTS_ASSERT((intc_u8) (u8  &= intc_u256_as_u8  (val)) == (intc_u8) 0xa0ULL);
            INTC_TESTS_ASSERT((intc_u16)(u16 &= intc_u256_as_u16 (val)) == (intc_u16)0xa0a0ULL);
            INTC_TESTS_ASSERT((intc_u32)(u32 &= intc_u256_as_u32 (val)) == (intc_u32)0xa0a0a0a0ULL);
            INTC_TESTS_ASSERT((intc_u64)(u64 &= intc_u256_as_u64 (val)) == (intc_u64)0xa0a0a0a0a0a0a0a0ULL);

            // zero
            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_and(INTC_U256_ZERO, val), INTC_U256_ZERO));
            INTC_TESTS_END;
        }
    }

    printf("\n  assignment.cpp\n");
    {
        {
            INTC_TESTS_BEGIN("Assignment.all");
            const struct intc_u256 t_1   = INTC_U64_TO_U256(true);
            const struct intc_u256 f_1   = INTC_U64_TO_U256(false);
            const struct intc_u256 u8_1  = INTC_U64_TO_U256(0x01);
            const struct intc_u256 u16_1 = INTC_U64_TO_U256(0x0123);
            const struct intc_u256 u32_1 = INTC_U64_TO_U256(0x01234567);
            const struct intc_u256 u64_1 = INTC_U64_TO_U256(0x0123456789abcdef);

            struct intc_u256 t_2   = INTC_U64_TO_U256(0);
            struct intc_u256 f_2   = INTC_U64_TO_U256(0);
            struct intc_u256 u8_2  = INTC_U64_TO_U256(0);
            struct intc_u256 u16_2 = INTC_U64_TO_U256(0);
            struct intc_u256 u32_2 = INTC_U64_TO_U256(0);
            struct intc_u256 u64_2 = INTC_U64_TO_U256(0);

            t_2   = t_1;
            f_2   = f_1;
            u8_2  = u8_1;
            u16_2 = u16_1;
            u32_2 = u32_1;
            u64_2 = u64_1;

            INTC_TESTS_ASSERT(intc_u256_eq(t_1, t_2));
            INTC_TESTS_ASSERT(intc_u256_eq(f_1, f_2));
            INTC_TESTS_ASSERT(intc_u256_eq(u8_1, u8_2));
            INTC_TESTS_ASSERT(intc_u256_eq(u16_1, u16_2));
            INTC_TESTS_ASSERT(intc_u256_eq(u32_1, u32_2));
            INTC_TESTS_ASSERT(intc_u256_eq(u64_1, u64_2));
            INTC_TESTS_END;
        }
    }

    printf("\n  constructor.cpp\n");
    {
        {
            INTC_TESTS_BEGIN("Constructor.standard");
            struct intc_u256       value    = INTC_U64_TO_U256(0x0123456789abcdefULL);
            const struct intc_u256 original = value;

            INTC_TESTS_ASSERT(intc_u256_eq(value, original));
            INTC_TESTS_ASSERT(intc_u256_eq(value, INTC_U64_TO_U256(0x0123456789abcdefULL)));
            INTC_TESTS_END;
        }

        {
            INTC_TESTS_BEGIN("Constructor.data");
            {
                char const string[]                      = "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff";
                struct intc_u256_init_result init_result = intc_u256_init_hex_cstring(string, sizeof(string) - 1, 0 /*separator*/);
                INTC_TESTS_ASSERT(init_result.success);
                INTC_TESTS_ASSERT_MSG(intc_u256_eq(init_result.value, INTC_U256_MAX),
                                     "val:    %s\n"
                                     "expect: %s\n",
                                     intc_u256_readable_hex_str(init_result.value).data,
                                     intc_u256_readable_hex_str(INTC_U256_MAX).data);
            }

            {
                char const string[]                      = "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff";
                struct intc_u256_init_result init_result = intc_u256_init_hex_cstring(string, sizeof(string) - 1, 0 /*separator*/);
                INTC_TESTS_ASSERT(init_result.success);
                INTC_TESTS_ASSERT_MSG(intc_u256_eq(init_result.value, INTC_U256_MAX),
                                     "val:    %s\n"
                                     "expect: %s\n",
                                     intc_u256_readable_hex_str(init_result.value).data,
                                     intc_u256_readable_hex_str(INTC_U256_MAX).data);
            }

            {
                char const string[]                      = "0xffff'ffff'ffff'ffff'ffff'ffff'ffff'ffff'ffff'ffff'ffff'ffff'ffff'ffff'ffff'ffff";
                struct intc_u256_init_result init_result = intc_u256_init_hex_cstring(string, sizeof(string) - 1, '\'');
                INTC_TESTS_ASSERT(init_result.success);
                INTC_TESTS_ASSERT_MSG(intc_u256_eq(init_result.value, INTC_U256_MAX),
                                     "val:    %s\n"
                                     "expect: %s\n",
                                     intc_u256_readable_hex_str(init_result.value).data,
                                     intc_u256_readable_hex_str(INTC_U256_MAX).data);
            }

            {
                char const string[]                      = "ffff'ffff'ffff'ffff'ffff'ffff'ffff'ffff'ffff'ffff'ffff'ffff'ffff'ffff'ffff'ffff";
                struct intc_u256_init_result init_result = intc_u256_init_hex_cstring(string, sizeof(string) - 1, '\'');
                INTC_TESTS_ASSERT(init_result.success);
                INTC_TESTS_ASSERT_MSG(intc_u256_eq(init_result.value, INTC_U256_MAX),
                                     "val:    %s\n"
                                     "expect: %s\n",
                                     intc_u256_readable_hex_str(init_result.value).data,
                                     intc_u256_readable_hex_str(INTC_U256_MAX).data);
            }

            {
                char const string[]                      = "0xffff'ffff'ffff'ffff'ffff'ffff'ffff'ffff'ffff'ffff'ffff'ffff'ffff'ffff'ffff'ffff'f";
                struct intc_u256_init_result init_result = intc_u256_init_hex_cstring(string, sizeof(string) - 1, '\'');
                INTC_TESTS_ASSERT(!init_result.success);
            }

            {
                char const string[]                      = "0xffff'ffff'ffff'ffff'ffff'ffff'ffff'fffg'ffff'ffff'ffff'ffff'ffff'ffff'ffff'ffff";
                struct intc_u256_init_result init_result = intc_u256_init_hex_cstring(string, sizeof(string) - 1, '\'');
                INTC_TESTS_ASSERT(!init_result.success);
            }

            {
                char const string[]                      = "0x0";
                struct intc_u256_init_result init_result = intc_u256_init_hex_cstring(string, sizeof(string) - 1, 0 /*separator*/);
                INTC_TESTS_ASSERT_MSG(intc_u256_eq(init_result.value, INTC_U256_ZERO),
                                      "val: %s\n",
                                      intc_u256_readable_hex_str(init_result.value).data);
            }

            {
                char const       string[]                = "0x0123456789abcdef";
                struct intc_u256 expect                  = INTC_U64_TO_U256(0x0123456789abcdefULL);
                struct intc_u256_init_result init_result = intc_u256_init_hex_cstring(string, sizeof(string) - 1, 0 /*separator*/);
                INTC_TESTS_ASSERT_MSG(intc_u256_eq(init_result.value, expect),
                                     "val:    %s\n"
                                     "expect: %s\n",
                                     intc_u256_readable_hex_str(init_result.value).data,
                                     intc_u256_readable_hex_str(expect).data);
            }
            INTC_TESTS_END;
        }

        // TODO: I got rid of the base functions because I wasn't happy with how it was implemented.
        // Reimplement one day.
        #if 0
        {
            INTC_TESTS_BEGIN("Constructor.base_string");
            struct intc_u256 val;
            INTC_TESTS_ASSERT(intc_u256_init_hex_cstring_base("ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff", -1 /*size*/, 16 /*base*/, &val));
            INTC_TESTS_ASSERT_MSG(intc_u256_eq(val, INTC_U256_MAX),
                                 "val:    %s\n"
                                 "expect: %s\n",
                                 intc_u256_readable_hex_str(val).data,
                                 intc_u256_readable_hex_str(INTC_U256_MAX).data);

            INTC_TESTS_ASSERT(intc_u256_init_hex_cstring_base("115792089237316195423570985008687907853269984665640564039457584007913129639935", -1 /*size*/, 10 /*base*/, &val));
            INTC_TESTS_ASSERT_MSG(intc_u256_eq(val, INTC_U256_MAX),
                                 "val:    %s\n"
                                 "expect: %s\n",
                                 intc_u256_readable_hex_str(val).data,
                                 intc_u256_readable_hex_str(INTC_U256_MAX).data);

            INTC_TESTS_ASSERT(intc_u256_init_hex_cstring_base("1111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111", -1 /*size*/, 2 /*base*/, &val));
            INTC_TESTS_ASSERT_MSG(intc_u256_eq(val, INTC_U256_MAX),
                                 "val:    %s\n"
                                 "expect: %s\n",
                                 intc_u256_readable_hex_str(val).data,
                                 intc_u256_readable_hex_str(INTC_U256_MAX).data);

            INTC_TESTS_ASSERT(intc_u256_init_hex_cstring_base("0", -1 /*size*/, 10 /*base*/, &val));
            INTC_TESTS_ASSERT(intc_u256_eq(val, INTC_U256_ZERO));

            INTC_TESTS_ASSERT(intc_u256_init_hex_cstring_base("0x0123456789abcdef", -1 /*size*/, 16 /*base*/, &val));
            INTC_TESTS_ASSERT(intc_u256_eq(val, INTC_U64_TO_U256(0x0123456789abcdefULL)));

            INTC_TESTS_ASSERT(intc_u256_init_hex_cstring_base("755", -1 /*size*/, 8 /*base*/, &val));
            INTC_TESTS_ASSERT(intc_u256_eq(val, INTC_U64_TO_U256(0x01ed)));

            INTC_TESTS_ASSERT(intc_u256_init_hex_cstring_base("31415926", -1 /*size*/, 10 /*base*/, &val));
            INTC_TESTS_ASSERT(intc_u256_eq(val, INTC_U64_TO_U256(0x01df5e76ULL)));
            INTC_TESTS_END;
        }
        #endif

        {
            INTC_TESTS_BEGIN("Constructor.one");
            INTC_TESTS_ASSERT(intc_u128_as_bool(INTC_U64_TO_U256(true).hi)  == false);
            INTC_TESTS_ASSERT(intc_u128_as_bool(INTC_U64_TO_U256(true).lo)  == true);
            INTC_TESTS_ASSERT(intc_u128_as_bool(INTC_U64_TO_U256(false).hi) == false);
            INTC_TESTS_ASSERT(intc_u128_as_bool(INTC_U64_TO_U256(false).lo) == false);

            INTC_TESTS_ASSERT(intc_u128_eq(INTC_U64_TO_U256((intc_u8)0x01ULL).hi,                INTC_U64_TO_U128(0ULL)));
            INTC_TESTS_ASSERT(intc_u128_eq(INTC_U64_TO_U256((intc_u16)0x0123ULL).hi,             INTC_U64_TO_U128(0ULL)));
            INTC_TESTS_ASSERT(intc_u128_eq(INTC_U64_TO_U256((intc_u32)0x01234567ULL).hi,         INTC_U64_TO_U128(0ULL)));
            INTC_TESTS_ASSERT(intc_u128_eq(INTC_U64_TO_U256((intc_u64)0x0123456789abcdefULL).hi, INTC_U64_TO_U128(0ULL)));

            INTC_TESTS_ASSERT(intc_u128_eq(INTC_U64_TO_U256((intc_u8)0x01ULL).lo,                INTC_U64_TO_U128((intc_u8)0x01ULL)));
            INTC_TESTS_ASSERT(intc_u128_eq(INTC_U64_TO_U256((intc_u16)0x0123ULL).lo,             INTC_U64_TO_U128((intc_u16)0x0123ULL)));
            INTC_TESTS_ASSERT(intc_u128_eq(INTC_U64_TO_U256((intc_u32)0x01234567ULL).lo,         INTC_U64_TO_U128((intc_u32)0x01234567ULL)));
            INTC_TESTS_ASSERT(intc_u128_eq(INTC_U64_TO_U256((intc_u64)0x0123456789abcdefULL).lo, INTC_U64_TO_U128((intc_u64)0x0123456789abcdefULL)));
            INTC_TESTS_END;
        }

        {
            INTC_TESTS_BEGIN("Constructor.two");
            for (int hi = 0; hi < 2; hi++)
            {
                for (int lo = 0; lo < 2; lo++)
                {
                    struct intc_u256 const val = INTC_U256(INTC_U64_TO_U128((intc_u64)lo), INTC_U64_TO_U128((intc_u64)hi));
                    INTC_TESTS_ASSERT(intc_u128_eq(val.hi, INTC_U64_TO_U128((intc_u64)hi)));
                    INTC_TESTS_ASSERT(intc_u128_eq(val.lo, INTC_U64_TO_U128((intc_u64)lo)));
                }
            }

            INTC_TESTS_ASSERT(intc_u128_eq(INTC_U256(INTC_U64_TO_U128((intc_u8) 0x01ULL),               INTC_U64_TO_U128((intc_u8)0x01ULL)).hi,                INTC_U64_TO_U128((intc_u8)0x01ULL)));
            INTC_TESTS_ASSERT(intc_u128_eq(INTC_U256(INTC_U64_TO_U128((intc_u16)0x0123ULL),             INTC_U64_TO_U128((intc_u16)0x0123ULL)).hi,             INTC_U64_TO_U128((intc_u16)0x0123ULL)));
            INTC_TESTS_ASSERT(intc_u128_eq(INTC_U256(INTC_U64_TO_U128((intc_u32)0x01234567ULL),         INTC_U64_TO_U128((intc_u32)0x01234567ULL)).hi,         INTC_U64_TO_U128((intc_u32)0x01234567ULL)));
            INTC_TESTS_ASSERT(intc_u128_eq(INTC_U256(INTC_U64_TO_U128((intc_u64)0x0123456789abcdefULL), INTC_U64_TO_U128((intc_u64)0x0123456789abcdefULL)).hi, INTC_U64_TO_U128((intc_u64)0x0123456789abcdefULL)));

            INTC_TESTS_ASSERT(intc_u128_eq(INTC_U256(INTC_U64_TO_U128((intc_u8) 0x01ULL),               INTC_U64_TO_U128((intc_u8) 0x01ULL)).lo,               INTC_U64_TO_U128((intc_u8)0x01ULL)));
            INTC_TESTS_ASSERT(intc_u128_eq(INTC_U256(INTC_U64_TO_U128((intc_u16)0x0123ULL),             INTC_U64_TO_U128((intc_u16)0x0123ULL)).lo,             INTC_U64_TO_U128((intc_u16)0x0123ULL)));
            INTC_TESTS_ASSERT(intc_u128_eq(INTC_U256(INTC_U64_TO_U128((intc_u32)0x01234567ULL),         INTC_U64_TO_U128((intc_u32)0x01234567ULL)).lo,         INTC_U64_TO_U128((intc_u32)0x01234567ULL)));
            INTC_TESTS_ASSERT(intc_u128_eq(INTC_U256(INTC_U64_TO_U128((intc_u64)0x0123456789abcdefULL), INTC_U64_TO_U128((intc_u64)0x0123456789abcdefULL)).lo, INTC_U64_TO_U128((intc_u64)0x0123456789abcdefULL)));
            INTC_TESTS_END;
        }
    }

    printf("\n  div.cpp\n");
    {
        {
            INTC_TESTS_BEGIN("Arithmetic.divide");
            const struct intc_u256 big     = INTC_U64_TO_U256(0xfedbca9876543210ULL);
            const struct intc_u256 small   = INTC_U64_TO_U256(0xffffULL);

            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_div(small, small), INTC_U64_TO_U256(1)));
            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_div(small, big), INTC_U64_TO_U256(0)));
            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_div(big, big), INTC_U64_TO_U256(1)));

            // TODO(dqn): Add a way to catch divide by 0 assert nicely? Maybe?
            // INTC_TESTS_ASSERT(intc_u256_div(INTC_U64_TO_U256(1), INTC_U64_TO_U256(0)));
            INTC_TESTS_END;
        }

        {
            INTC_TESTS_BEGIN("External.divide");
            bool             t    = true;
            bool             f    = false;
            intc_u8          u8   = 0xaaULL;
            intc_u16         u16  = 0xaaaaULL;
            intc_u32         u32  = 0xaaaaaaaaULL;
            intc_u64         u64  = 0xaaaaaaaaaaaaaaaaULL;
            struct intc_u128 u128 = INTC_U128(0xaaaaaaaaaaaaaaaaULL, 0xaaaaaaaaaaaaaaaaULL);
            struct intc_u256 val  = INTC_U64_TO_U256(0x7bULL);

            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_div(INTC_U64_TO_U256(t),   val), INTC_U64_TO_U256(false)));
            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_div(INTC_U64_TO_U256(f),   val), INTC_U64_TO_U256(false)));
            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_div(INTC_U64_TO_U256(u8),  val), INTC_U64_TO_U256(0x1ULL)));
            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_div(INTC_U64_TO_U256(u16), val), INTC_U64_TO_U256(0x163ULL)));
            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_div(INTC_U64_TO_U256(u32), val), INTC_U64_TO_U256(0x163356bULL)));
            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_div(INTC_U64_TO_U256(u64), val), INTC_U64_TO_U256(0x163356b88ac0de0ULL)));
            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_div(u128, intc_u256_as_u128(val)), INTC_U128(0x163356b88ac0de01ULL, 0x163356b88ac0de0ULL)));
            INTC_TESTS_END;
        }
    }

    printf("\n  equals.cpp\n");
    {
        {
            INTC_TESTS_BEGIN("Comparison.equals");
            INTC_TESTS_ASSERT(intc_u256_eq(INTC_U64_TO_U256(0xdeadbeefULL), INTC_U64_TO_U256(0xdeadbeefULL)));
            INTC_TESTS_ASSERT(!intc_u256_eq(INTC_U64_TO_U256(0xdeadbeefULL), INTC_U64_TO_U256(0xfee1baadULL)));
            INTC_TESTS_END;
        }
    }

    printf("\n  fix.cpp\n");
    {
        {
            INTC_TESTS_BEGIN("Arithmetic.increment");
            struct intc_u256 value = INTC_U256_ZERO;
            value = intc_u256_add_u64(value, 1);
            INTC_TESTS_ASSERT(intc_u256_eq(value, INTC_U64_TO_U256(1)));
            INTC_TESTS_END;
        }

        {
            INTC_TESTS_BEGIN("Arithmetic.decrement");
            struct intc_u256 value = INTC_U256_ZERO;
            value = intc_u256_sub_u64(value, 1);
            INTC_TESTS_ASSERT(intc_u256_eq(value, INTC_U256(INTC_U128(0xffffffffffffffffULL, 0xffffffffffffffffULL), INTC_U128(0xffffffffffffffffULL, 0xffffffffffffffffULL))));
            INTC_TESTS_END;
        }
    }

    printf("\n  functions.cpp\n");
    {
        {
            INTC_TESTS_BEGIN("Function.data");
            // make sure all of the test strings create the ASCII version of the string
            struct intc_u256 const original = INTC_U64_TO_U256(2216002924);
            for (int test_index = 0; test_index < (int)(sizeof(INTC_TESTS_STRING_BASE_TESTS)/sizeof(INTC_TESTS_STRING_BASE_TESTS[0])); test_index++)
            {
                struct intc_base_to_string const *test_entry = INTC_TESTS_STRING_BASE_TESTS + test_index;
                struct intc_u256_string output = intc_u256_str(original, test_entry->base, 0 /*separate_every_n_chars*/, ' ' /*separate_ch*/);
                INTC_TESTS_ASSERT_MSG(strcmp(output.data, test_entry->expect) == 0,
                                     "output: %s\n"
                                     "expect: %s\n",
                                     output.data,
                                     test_entry->expect);
            }
            INTC_TESTS_END;
        }
    }

    printf("\n  gt.cpp\n");
    {
        {
            INTC_TESTS_BEGIN("Comparison.greater_than");
            struct intc_u256 const big   = INTC_U256(INTC_U64_TO_U128(0xffffffffffffffffULL), INTC_U64_TO_U128(0xffffffffffffffffULL));
            struct intc_u256 const small = INTC_U256(INTC_U64_TO_U128(0x0000000000000000ULL), INTC_U64_TO_U128(0x0000000000000000ULL));

            INTC_TESTS_ASSERT(intc_u256_gt(small, small) == false);
            INTC_TESTS_ASSERT(intc_u256_gt(small, big) ==  false);

            INTC_TESTS_ASSERT(intc_u256_gt(big, small) == true);
            INTC_TESTS_ASSERT(intc_u256_gt(big, big) == false);
            INTC_TESTS_END;
        }

        {
            INTC_TESTS_BEGIN("External.greater_than");
            for (int index = 0;
                 index < (int)(sizeof(INTC_TESTS_MAX_UNSIGNED_VALUES)/sizeof(INTC_TESTS_MAX_UNSIGNED_VALUES[0]));
                 index++)
            {
                struct intc_u256 const small = INTC_U256_ZERO;
                struct intc_u256 const big   = INTC_U64_TO_U256(INTC_TESTS_MAX_UNSIGNED_VALUES[index]);

                INTC_TESTS_ASSERT(intc_u256_gt(small, small) == false);
                INTC_TESTS_ASSERT(intc_u256_gt(small, big) ==  false);

                INTC_TESTS_ASSERT(intc_u256_gt(big, small) == true);
                INTC_TESTS_ASSERT(intc_u256_gt(big, big) == false);
            }

            INTC_TESTS_END;
        }
    }

    printf("\n  gte.cpp\n");
    {
        {
            INTC_TESTS_BEGIN("Comparison.greater_than_or_equals");
            struct intc_u256 const big   = INTC_U256(INTC_U64_TO_U128(0xffffffffffffffffULL), INTC_U64_TO_U128(0xffffffffffffffffULL));
            struct intc_u256 const small = INTC_U256(INTC_U64_TO_U128(0x0000000000000000ULL), INTC_U64_TO_U128(0x0000000000000000ULL));

            INTC_TESTS_ASSERT(intc_u256_gt_eq(small, small) == true);
            INTC_TESTS_ASSERT(intc_u256_gt_eq(small, big)   ==  false);

            INTC_TESTS_ASSERT(intc_u256_gt_eq(big, small) == true);
            INTC_TESTS_ASSERT(intc_u256_gt_eq(big, big) == true);
            INTC_TESTS_END;
        }

        {
            INTC_TESTS_BEGIN("External.greater_than_or_equals");
            for (int index = 0;
                 index < (int)(sizeof(INTC_TESTS_MAX_UNSIGNED_VALUES)/sizeof(INTC_TESTS_MAX_UNSIGNED_VALUES[0]));
                 index++)
            {
                struct intc_u256 const small = INTC_U256_ZERO;
                struct intc_u256 const big   = INTC_U64_TO_U256(INTC_TESTS_MAX_UNSIGNED_VALUES[index]);

                INTC_TESTS_ASSERT(intc_u256_gt_eq(small, small) == true);
                INTC_TESTS_ASSERT(intc_u256_gt_eq(small, big) ==  false);

                INTC_TESTS_ASSERT(intc_u256_gt_eq(big, small) == true);
                INTC_TESTS_ASSERT(intc_u256_gt_eq(big, big) == true);
            }
            INTC_TESTS_END;
        }
    }

    printf("\n  invert.cpp\n");
    {
        {
            INTC_TESTS_BEGIN("BitWise.invert");
            for (int hi_hi = 0; hi_hi < 2; hi_hi++)
            {
                for (int hi_lo = 0; hi_lo < 2; hi_lo++)
                {
                    for (int lo_hi = 0; lo_hi < 2; lo_hi++)
                    {
                        for (int lo_lo = 0; lo_lo < 2; lo_lo++)
                        {
                            struct intc_u256 const val =
                                intc_u256_negate(INTC_U256(
                                        INTC_U128(lo_lo ? 0xffffffffffffffffULL : 0x0000000000000000ULL, lo_hi ? 0xffffffffffffffffULL : 0x0000000000000000ULL),
                                        INTC_U128(hi_lo ? 0xffffffffffffffffULL : 0x0000000000000000ULL, hi_hi ? 0xffffffffffffffffULL : 0x0000000000000000ULL)));

                            INTC_TESTS_ASSERT(val.hi.hi == (intc_u64)hi_hi ? 0x0000000000000000ULL : 0xffffffffffffffffULL);
                            INTC_TESTS_ASSERT(val.hi.lo == (intc_u64)hi_lo ? 0x0000000000000000ULL : 0xffffffffffffffffULL);
                            INTC_TESTS_ASSERT(val.lo.hi == (intc_u64)lo_hi ? 0x0000000000000000ULL : 0xffffffffffffffffULL);
                            INTC_TESTS_ASSERT(val.lo.lo == (intc_u64)lo_lo ? 0x0000000000000000ULL : 0xffffffffffffffffULL);
                        }
                    }
                }
            }
            INTC_TESTS_END;
        }
    }

    printf("\n  leftshift.cpp\n");
    {
        {
            INTC_TESTS_BEGIN("BitShift.left");
            // operator<<
            struct intc_u256 val     = INTC_U64_TO_U256(0x1);
            intc_u64         exp_val = 1;
            for(int i = 0; i < 64; i++){
                INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_lshift(val, i), INTC_U64_TO_U256(exp_val << i)));
            }

            struct intc_u256 zero = INTC_U256_ZERO;
            for (int i = 0; i < 64; i++)
                INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_lshift(zero, i), INTC_U256_ZERO));
            INTC_TESTS_END;
        }

        {
            INTC_TESTS_BEGIN("External.shift_left");
            intc_u8                u8   = 0xffULL;
            intc_u16               u16  = 0xffffULL;
            intc_u32               u32  = 0xffffffffULL;
            intc_u64               u64  = 0xffffffffffffffffULL;
            struct intc_u128 const u128 = INTC_U128(0xffffffffffffffffULL, 0xffffffffffffffffULL);

            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_lshift(INTC_U64_TO_U256(u8), 7),      INTC_U64_TO_U256(0x7f80ULL)));
            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_lshift(INTC_U64_TO_U256(u16), 15),    INTC_U64_TO_U256(0x7fff8000ULL)));
            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_lshift(INTC_U64_TO_U256(u32), 31),    INTC_U64_TO_U256(0x7fffffff80000000ULL)));
            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_lshift(INTC_U64_TO_U256(u64), 63),    INTC_U256(INTC_U128(0x8000000000000000ULL, 0x7fffffffffffffffULL), INTC_U128(0x0000000000000000ULL, 0x0000000000000000ULL))));
            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_lshift(INTC_U128_TO_U256(u128), 127), INTC_U256(INTC_U128(0x0, 0x8000000000000000ULL), INTC_U128(0xffffffffffffffffULL, 0x7fffffffffffffffULL))));
            INTC_TESTS_END;
        }
    }

    printf("\n  lt.cpp\n");
    {
        {
            INTC_TESTS_BEGIN("Comparison.less_than");
            struct intc_u256 const big   = INTC_U256(INTC_U64_TO_U128(0xffffffffffffffffULL), INTC_U64_TO_U128(0xffffffffffffffffULL));
            struct intc_u256 const small = INTC_U256(INTC_U64_TO_U128(0x0000000000000000ULL), INTC_U64_TO_U128(0x0000000000000000ULL));

            INTC_TESTS_ASSERT(intc_u256_lt(small, small) == false);
            INTC_TESTS_ASSERT(intc_u256_lt(small, big)   == true);

            INTC_TESTS_ASSERT(intc_u256_lt(big, small) == false);
            INTC_TESTS_ASSERT(intc_u256_lt(big, big)   == false);
            INTC_TESTS_END;
        }

        {
            INTC_TESTS_BEGIN("External.less_than");
            for (int index = 0;
                 index < (int)(sizeof(INTC_TESTS_MAX_UNSIGNED_VALUES)/sizeof(INTC_TESTS_MAX_UNSIGNED_VALUES[0]));
                 index++)
            {
                struct intc_u256 const small = INTC_U256_ZERO;
                struct intc_u256 const big   = INTC_U64_TO_U256(INTC_TESTS_MAX_UNSIGNED_VALUES[index]);

                INTC_TESTS_ASSERT(intc_u256_lt(small, small) == false);
                INTC_TESTS_ASSERT(intc_u256_lt(small, big)   == true);

                INTC_TESTS_ASSERT(intc_u256_lt(big, small) == false);
                INTC_TESTS_ASSERT(intc_u256_lt(big, big)   == false);
            }
            INTC_TESTS_END;
        }
    }

    printf("\n  lte.cpp\n");
    {
        {
            INTC_TESTS_BEGIN("Comparison.less_than_or_equals");
            struct intc_u256 const big   = INTC_U256(INTC_U64_TO_U128(0xffffffffffffffffULL), INTC_U64_TO_U128(0xffffffffffffffffULL));
            struct intc_u256 const small = INTC_U256(INTC_U64_TO_U128(0x0000000000000000ULL), INTC_U64_TO_U128(0x0000000000000000ULL));

            INTC_TESTS_ASSERT(intc_u256_lt_eq(small, small) == true);
            INTC_TESTS_ASSERT(intc_u256_lt_eq(small, big)   == true);

            INTC_TESTS_ASSERT(intc_u256_lt_eq(big, small) == false);
            INTC_TESTS_ASSERT(intc_u256_lt_eq(big, big)   == true);
            INTC_TESTS_END;
        }

        {
            INTC_TESTS_BEGIN("External.less_than_or_equals");
            for (int index = 0;
                 index < (int)(sizeof(INTC_TESTS_MAX_UNSIGNED_VALUES)/sizeof(INTC_TESTS_MAX_UNSIGNED_VALUES[0]));
                 index++)
            {
                struct intc_u256 const small = INTC_U256_ZERO;
                struct intc_u256 const big   = INTC_U64_TO_U256(INTC_TESTS_MAX_UNSIGNED_VALUES[index]);

                INTC_TESTS_ASSERT(intc_u256_lt_eq(small, small) == true);
                INTC_TESTS_ASSERT(intc_u256_lt_eq(small, big)   == true);

                INTC_TESTS_ASSERT(intc_u256_lt_eq(big, small) == false);
                INTC_TESTS_ASSERT(intc_u256_lt_eq(big, big)   == true);
            }
            INTC_TESTS_END;
        }
    }

    printf("\n  mod.cpp\n");
    {
        {
            INTC_TESTS_BEGIN("Arithmetic.modulo");
            // has remainder
            struct intc_u256 const val     = INTC_U256(INTC_U128(0xffffffffffffffffULL, 0xffffffffffffffffULL), INTC_U128(0xffffffffffffffffULL, 0xffffffffffffffffULL));
            struct intc_u256 const val_mod = INTC_U64_TO_U256(0xfedcba9876543210ULL);

            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_mod(val, val_mod), INTC_U64_TO_U256(0x63794f9d55c8d29f)));

            // no remainder
            struct intc_u256 const val_0 = INTC_U256(INTC_U128(0, 0), INTC_U128(0, 0xfedcba9876543210));
            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_mod(val_0, val_mod), INTC_U256_ZERO));

            // TODO(dqn): Add a way to catch divide by 0 assert nicely? Maybe?
            // mod 0
            // EXPECT_THROW(uint256_t(1) % uint256_t(0), std::domain_error);
            INTC_TESTS_END;
        }

        {
            INTC_TESTS_BEGIN("External.modulo");
            bool                   t    = true;
            bool                   f    = false;
            intc_u8                u8   = 0xaaULL;
            intc_u16               u16  = 0xaaaaULL;
            intc_u32               u32  = 0xaaaaaaaaULL;
            intc_u64               u64  = 0xaaaaaaaaaaaaaaaaULL;
            struct intc_u128 const u128 = INTC_U128(0xaaaaaaaaaaaaaaaaULL, 0xaaaaaaaaaaaaaaaaULL);
            struct intc_u256 const val  = INTC_U64_TO_U256(0xd03ULL); // prime

            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_mod(INTC_U64_TO_U256(t), val),   INTC_U64_TO_U256(true)));
            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_mod(INTC_U64_TO_U256(f), val),   INTC_U64_TO_U256(false)));
            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_mod(INTC_U64_TO_U256(u8), val),  INTC_U64_TO_U256((intc_u8) 0xaaULL)));
            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_mod(INTC_U64_TO_U256(u16), val), INTC_U64_TO_U256((intc_u16)0x183ULL)));
            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_mod(INTC_U64_TO_U256(u32), val), INTC_U64_TO_U256((intc_u32)0x249ULL)));
            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_mod(INTC_U64_TO_U256(u64), val), INTC_U64_TO_U256((intc_u64)0xc7fULL)));
            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_mod(u128, intc_u256_as_u128(val)), INTC_U64_TO_U128(0x9fbULL)));
            INTC_TESTS_END;
        }
    }

    printf("\n  mult.cpp\n");
    {
        {
            INTC_TESTS_BEGIN("Arithmetic.multiply");
            struct intc_u256 const val = INTC_U64_TO_U256(0xfedbca9876543210ULL);
            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_mul(val, val), INTC_U256(INTC_U128(0x010e6cd7a44a4100ULL, 0xfdb8e2bacbfe7cefULL), INTC_U128(0x0000000000000000ULL, 0x0000000000000000ULL))));

            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_mul(val, INTC_U256_ZERO), INTC_U256_ZERO));
            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_mul(INTC_U256_ZERO, val), INTC_U256_ZERO));

            struct intc_u256 const one = INTC_U64_TO_U256(1);
            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_mul(val, one), val));
            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_mul(one, val), val));
            INTC_TESTS_END;
        }

        {
            INTC_TESTS_BEGIN("External.multiply");
            bool                   t    = true;
            bool                   f    = false;
            intc_u8                u8   = 0xaaULL;
            intc_u16               u16  = 0xaaaaULL;
            intc_u32               u32  = 0xaaaaaaaaULL;
            intc_u64               u64  = 0xaaaaaaaaaaaaaaaaULL;
            struct intc_u128 const u128 = INTC_U128(0xaaaaaaaaaaaaaaaaULL, 0xaaaaaaaaaaaaaaaaULL);
            struct intc_u256 const val  = INTC_U256(INTC_U128(0xf0f0f0f0f0f0f0f0ULL, 0xf0f0f0f0f0f0f0f0ULL), INTC_U128(0xf0f0f0f0f0f0f0f0ULL, 0xf0f0f0f0f0f0f0f0ULL));

            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_mul(INTC_U64_TO_U256(t), val),     INTC_U256(INTC_U128(0xf0f0f0f0f0f0f0f0ULL, 0xf0f0f0f0f0f0f0f0ULL), INTC_U128(0xf0f0f0f0f0f0f0f0ULL, 0xf0f0f0f0f0f0f0f0ULL))));
            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_mul(INTC_U64_TO_U256(f), val),     INTC_U256(INTC_U128(0x0000000000000000ULL, 0x0000000000000000ULL), INTC_U128(0x0000000000000000ULL, 0x0000000000000000ULL))));
            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_mul(INTC_U64_TO_U256(u8), val),    INTC_U256(INTC_U128(0xffffffffffffff60ULL, 0xffffffffffffffffULL), INTC_U128(0xffffffffffffffffULL, 0xffffffffffffffffULL))));
            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_mul(INTC_U64_TO_U256(u16), val),   INTC_U256(INTC_U128(0xffffffffffff5f60ULL, 0xffffffffffffffffULL), INTC_U128(0xffffffffffffffffULL, 0xffffffffffffffffULL))));
            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_mul(INTC_U64_TO_U256(u32), val),   INTC_U256(INTC_U128(0xffffffff5f5f5f60ULL, 0xffffffffffffffffULL), INTC_U128(0xffffffffffffffffULL, 0xffffffffffffffffULL))));
            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_mul(INTC_U64_TO_U256(u64), val),   INTC_U256(INTC_U128(0x5f5f5f5f5f5f5f60ULL, 0xffffffffffffffffULL), INTC_U128(0xffffffffffffffffULL, 0xffffffffffffffffULL))));
            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_mul(INTC_U128_TO_U256(u128), val), INTC_U256(INTC_U128(0x5f5f5f5f5f5f5f60ULL, 0x5f5f5f5f5f5f5f5fULL), INTC_U128(0xffffffffffffffffULL, 0xffffffffffffffffULL))));

            INTC_TESTS_ASSERT((intc_u8)(u8   * intc_u256_as_u8(val))  == (intc_u8)0x60ULL);
            INTC_TESTS_ASSERT((intc_u16)(u16 * intc_u256_as_u16(val)) == (intc_u16)0x5f60ULL);
            INTC_TESTS_ASSERT((intc_u32)(u32 * intc_u256_as_u32(val)) == (intc_u32)0x5f5f5f60ULL);
            INTC_TESTS_ASSERT((intc_u64)(u64 * intc_u256_as_u64(val)) == (intc_u64)0x5f5f5f5f5f5f5f60ULL);
            INTC_TESTS_ASSERT(intc_u128_eq(intc_u128_mul(u128, intc_u256_as_u128(val)), INTC_U128(0x5f5f5f5f5f5f5f60ULL, 0x5f5f5f5f5f5f5f5fULL)));
            INTC_TESTS_END;
        }
    }

    printf("\n  notequals.cpp\n");
    {
        {
            INTC_TESTS_BEGIN("Comparison.not_equals");
            INTC_TESTS_ASSERT(intc_u256_neq(INTC_U64_TO_U256(0xdeadbeefULL), INTC_U64_TO_U256(0xdeadbeefULL)) == false);
            INTC_TESTS_ASSERT(intc_u256_neq(INTC_U64_TO_U256(0xdeadbeefULL), INTC_U64_TO_U256(0xfee1baadULL)) == true);
            INTC_TESTS_END;
        }

        {
            INTC_TESTS_BEGIN("External.not_equals");
            bool     const t   = true;
            bool     const f   = false;
            intc_u8  const u8  = 0xaaULL;
            intc_u16 const u16 = 0xaaaaULL;
            intc_u32 const u32 = 0xaaaaaaaaULL;
            intc_u64 const u64 = 0xaaaaaaaaaaaaaaaaULL;

            INTC_TESTS_ASSERT(intc_u256_neq(INTC_U64_TO_U256(t)  , INTC_U64_TO_U256(f))   == true);
            INTC_TESTS_ASSERT(intc_u256_neq(INTC_U64_TO_U256(f)  , INTC_U64_TO_U256(t))   == true);
            INTC_TESTS_ASSERT(intc_u256_neq(INTC_U64_TO_U256(u8) , INTC_U64_TO_U256(u64)) == true);
            INTC_TESTS_ASSERT(intc_u256_neq(INTC_U64_TO_U256(u16), INTC_U64_TO_U256(u32)) == true);
            INTC_TESTS_ASSERT(intc_u256_neq(INTC_U64_TO_U256(u32), INTC_U64_TO_U256(u16)) == true);
            INTC_TESTS_ASSERT(intc_u256_neq(INTC_U64_TO_U256(u64), INTC_U64_TO_U256(u8))  == true);

            INTC_TESTS_ASSERT(intc_u256_neq(INTC_U64_TO_U256(t)  , INTC_U64_TO_U256(t))   == false);
            INTC_TESTS_ASSERT(intc_u256_neq(INTC_U64_TO_U256(f)  , INTC_U64_TO_U256(f))   == false);
            INTC_TESTS_ASSERT(intc_u256_neq(INTC_U64_TO_U256(u8) , INTC_U64_TO_U256(u8))  == false);
            INTC_TESTS_ASSERT(intc_u256_neq(INTC_U64_TO_U256(u16), INTC_U64_TO_U256(u16)) == false);
            INTC_TESTS_ASSERT(intc_u256_neq(INTC_U64_TO_U256(u32), INTC_U64_TO_U256(u32)) == false);
            INTC_TESTS_ASSERT(intc_u256_neq(INTC_U64_TO_U256(u64), INTC_U64_TO_U256(u64)) == false);
            INTC_TESTS_END;
        }
    }

    printf("\n  or.cpp\n");
    {
        {
            INTC_TESTS_BEGIN("BitWise.or");
            struct intc_u256 const t   = INTC_U64_TO_U256((bool)     true);
            struct intc_u256 const f   = INTC_U64_TO_U256((bool)     false);
            struct intc_u256 const u8  = INTC_U64_TO_U256((intc_u8)  0xaaULL);
            struct intc_u256 const u16 = INTC_U64_TO_U256((intc_u16) 0xaaaaULL);
            struct intc_u256 const u32 = INTC_U64_TO_U256((intc_u32) 0xaaaaaaaaULL);
            struct intc_u256 const u64 = INTC_U64_TO_U256((intc_u64) 0xaaaaaaaaaaaaaaaaULL);
            struct intc_u256 const val = INTC_U256(INTC_U64_TO_U128(0xf0f0f0f0f0f0f0f0ULL), INTC_U64_TO_U128(0xf0f0f0f0f0f0f0f0ULL));

            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_or(t  , val), INTC_U256(INTC_U64_TO_U128(0xf0f0f0f0f0f0f0f1ULL), INTC_U64_TO_U128(0xf0f0f0f0f0f0f0f0ULL))));
            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_or(f  , val), INTC_U256(INTC_U64_TO_U128(0xf0f0f0f0f0f0f0f0ULL), INTC_U64_TO_U128(0xf0f0f0f0f0f0f0f0ULL))));
            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_or(u8 , val), INTC_U256(INTC_U64_TO_U128(0xf0f0f0f0f0f0f0faULL), INTC_U64_TO_U128(0xf0f0f0f0f0f0f0f0ULL))));
            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_or(u16, val), INTC_U256(INTC_U64_TO_U128(0xf0f0f0f0f0f0fafaULL), INTC_U64_TO_U128(0xf0f0f0f0f0f0f0f0ULL))));
            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_or(u32, val), INTC_U256(INTC_U64_TO_U128(0xf0f0f0f0fafafafaULL), INTC_U64_TO_U128(0xf0f0f0f0f0f0f0f0ULL))));
            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_or(u64, val), INTC_U256(INTC_U64_TO_U128(0xfafafafafafafafaULL), INTC_U64_TO_U128(0xf0f0f0f0f0f0f0f0ULL))));

            // zero
            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_or(INTC_U256_ZERO, val), val));
            INTC_TESTS_END;
        }

        {
            INTC_TESTS_BEGIN("External.or");
            bool     t   = true;
            bool     f   = false;
            intc_u8  u8  = 0xaa;
            intc_u16 u16 = 0xaaaa;
            intc_u32 u32 = 0xaaaaaaaaULL;
            intc_u64 u64 = 0xaaaaaaaaaaaaaaaaULL;
            struct intc_u256 const val = INTC_U256(INTC_U64_TO_U128(0xf0f0f0f0f0f0f0f0ULL), INTC_U64_TO_U128(0xf0f0f0f0f0f0f0f0ULL));

            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_or(INTC_U64_TO_U256(t)  , val), INTC_U256(INTC_U64_TO_U128(0xf0f0f0f0f0f0f0f1ULL), INTC_U64_TO_U128(0xf0f0f0f0f0f0f0f0ULL))));
            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_or(INTC_U64_TO_U256(f)  , val), INTC_U256(INTC_U64_TO_U128(0xf0f0f0f0f0f0f0f0ULL), INTC_U64_TO_U128(0xf0f0f0f0f0f0f0f0ULL))));
            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_or(INTC_U64_TO_U256(u8) , val), INTC_U256(INTC_U64_TO_U128(0xf0f0f0f0f0f0f0faULL), INTC_U64_TO_U128(0xf0f0f0f0f0f0f0f0ULL))));
            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_or(INTC_U64_TO_U256(u16), val), INTC_U256(INTC_U64_TO_U128(0xf0f0f0f0f0f0fafaULL), INTC_U64_TO_U128(0xf0f0f0f0f0f0f0f0ULL))));
            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_or(INTC_U64_TO_U256(u32), val), INTC_U256(INTC_U64_TO_U128(0xf0f0f0f0fafafafaULL), INTC_U64_TO_U128(0xf0f0f0f0f0f0f0f0ULL))));
            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_or(INTC_U64_TO_U256(u64), val), INTC_U256(INTC_U64_TO_U128(0xfafafafafafafafaULL), INTC_U64_TO_U128(0xf0f0f0f0f0f0f0f0ULL))));
            INTC_TESTS_END;
        }
    }

    printf("\n  rightshift.cpp\n");
    {
        {
            INTC_TESTS_BEGIN("BitShift.right");
            // operator>>
            struct intc_u256 val = INTC_U64_TO_U256(0xffffffffffffffffULL);
            intc_u64         exp = 0xffffffffffffffffULL;

            for(int i = 0; i < 64; i++){
                INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_rshift(val, i), INTC_U64_TO_U256(exp >> i)));
            }

            struct intc_u256 zero = INTC_U256_ZERO;
            for (int i = 0; i < 64; i++)
                INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_rshift(zero, i), INTC_U256_ZERO));
            INTC_TESTS_END;
        }

        {
            INTC_TESTS_BEGIN("External.shift_left");
            bool     t   = true;
            bool     f   = false;
            intc_u8  u8  = 0xffULL;
            intc_u16 u16 = 0xffffULL;
            intc_u32 u32 = 0xffffffffULL;
            intc_u64 u64 = 0xffffffffffffffffULL;

            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_rshift(INTC_U64_TO_U256(t) , 0), INTC_U64_TO_U256(1)));
            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_rshift(INTC_U64_TO_U256(f) , 0), INTC_U256_ZERO));
            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_rshift(INTC_U64_TO_U256(u8), 0),  INTC_U64_TO_U256(u8)));
            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_rshift(INTC_U64_TO_U256(u16), 0), INTC_U64_TO_U256(u16)));
            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_rshift(INTC_U64_TO_U256(u32), 0), INTC_U64_TO_U256(u32)));
            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_rshift(INTC_U64_TO_U256(u64), 0), INTC_U64_TO_U256(u64)));

            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_rshift(INTC_U64_TO_U256(t), 1),   INTC_U64_TO_U256((intc_u64)t   >> 1)));
            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_rshift(INTC_U64_TO_U256(f), 1),   INTC_U64_TO_U256((intc_u64)f   >> 1)));
            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_rshift(INTC_U64_TO_U256(u8), 1),  INTC_U64_TO_U256((intc_u64)u8  >> 1)));
            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_rshift(INTC_U64_TO_U256(u16), 1), INTC_U64_TO_U256((intc_u64)u16 >> 1)));
            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_rshift(INTC_U64_TO_U256(u32), 1), INTC_U64_TO_U256((intc_u64)u32 >> 1)));
            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_rshift(INTC_U64_TO_U256(u64), 1), INTC_U64_TO_U256(u64 >> 1)));

            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_rshift(INTC_U64_TO_U256(u8), 7),   INTC_U64_TO_U256(1)));
            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_rshift(INTC_U64_TO_U256(u16), 15), INTC_U64_TO_U256(1)));
            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_rshift(INTC_U64_TO_U256(u32), 31), INTC_U64_TO_U256(1)));
            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_rshift(INTC_U64_TO_U256(u64), 63), INTC_U64_TO_U256(1)));
            INTC_TESTS_END;
        }
    }

    printf("\n  sub.cpp\n");
    {
        {
            INTC_TESTS_BEGIN("Arithmetic.subtract");
            struct intc_u256 const big   = INTC_U256(INTC_U128(0xffffffffffffffffULL, 0xffffffffffffffffULL), INTC_U128(0xffffffffffffffffULL, 0xffffffffffffffffULL));
            struct intc_u256 const small = INTC_U256(INTC_U64_TO_U128(0x0000000000000001ULL), INTC_U64_TO_U128(0x0000000000000000ULL));

            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_sub(small, small), INTC_U256_ZERO));
            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_sub(small, big),   INTC_U256(INTC_U128(0x0000000000000002ULL, 0x0000000000000000ULL), INTC_U128(0x0000000000000000ULL, 0x0000000000000000ULL))));
            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_sub(big  , small), INTC_U256(INTC_U128(0xfffffffffffffffeULL, 0xffffffffffffffffULL), INTC_U128(0xffffffffffffffffULL, 0xffffffffffffffffULL))));
            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_sub(big  , big),   INTC_U256_ZERO));
            INTC_TESTS_END;
        }

        {
            INTC_TESTS_BEGIN("External.subtract");
            bool             t    = true;
            bool             f    = false;
            intc_u8          u8   = 0xaaULL;
            intc_u16         u16  = 0xaaaaULL;
            intc_u32         u32  = 0xaaaaaaaaULL;
            intc_u64         u64  = 0xaaaaaaaaaaaaaaaaULL;
            struct intc_u128 u128 = INTC_U128(0xaaaaaaaaaaaaaaaaULL, 0xaaaaaaaaaaaaaaaaULL);
            struct intc_u256 val  = INTC_U256(INTC_U128(0xf0f0f0f0f0f0f0f0ULL, 0xf0f0f0f0f0f0f0f0ULL), INTC_U128(0xf0f0f0f0f0f0f0f0ULL, 0xf0f0f0f0f0f0f0f0ULL));

            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_sub(INTC_U64_TO_U256(t)  , val),   INTC_U256(INTC_U128(0x0f0f0f0f0f0f0f11ULL, 0x0f0f0f0f0f0f0f0fULL), INTC_U128(0x0f0f0f0f0f0f0f0fULL, 0x0f0f0f0f0f0f0f0fULL))));
            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_sub(INTC_U64_TO_U256(f)  , val),   INTC_U256(INTC_U128(0x0f0f0f0f0f0f0f10ULL, 0x0f0f0f0f0f0f0f0fULL), INTC_U128(0x0f0f0f0f0f0f0f0fULL, 0x0f0f0f0f0f0f0f0fULL))));
            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_sub(INTC_U64_TO_U256(u8) , val),   INTC_U256(INTC_U128(0x0f0f0f0f0f0f0fbaULL, 0x0f0f0f0f0f0f0f0fULL), INTC_U128(0x0f0f0f0f0f0f0f0fULL, 0x0f0f0f0f0f0f0f0fULL))));
            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_sub(INTC_U64_TO_U256(u16), val),   INTC_U256(INTC_U128(0x0f0f0f0f0f0fb9baULL, 0x0f0f0f0f0f0f0f0fULL), INTC_U128(0x0f0f0f0f0f0f0f0fULL, 0x0f0f0f0f0f0f0f0fULL))));
            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_sub(INTC_U64_TO_U256(u32), val),   INTC_U256(INTC_U128(0x0f0f0f0fb9b9b9baULL, 0x0f0f0f0f0f0f0f0fULL), INTC_U128(0x0f0f0f0f0f0f0f0fULL, 0x0f0f0f0f0f0f0f0fULL))));
            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_sub(INTC_U64_TO_U256(u64), val),   INTC_U256(INTC_U128(0xb9b9b9b9b9b9b9baULL, 0x0f0f0f0f0f0f0f0fULL), INTC_U128(0x0f0f0f0f0f0f0f0fULL, 0x0f0f0f0f0f0f0f0fULL))));
            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_sub(INTC_U128_TO_U256(u128), val), INTC_U256(INTC_U128(0xb9b9b9b9b9b9b9baULL, 0xb9b9b9b9b9b9b9b9ULL), INTC_U128(0x0f0f0f0f0f0f0f0fULL, 0x0f0f0f0f0f0f0f0fULL))));
            INTC_TESTS_END;
        }
    }

    printf("\n  typecast.cpp\n");
    {
        {
            INTC_TESTS_BEGIN("Typecast.all");
            struct intc_u256 const val = INTC_U256(INTC_U64_TO_U128(0xaaaaaaaaaaaaaaaaULL), INTC_U64_TO_U128(0xaaaaaaaaaaaaaaaaULL));
            INTC_TESTS_ASSERT(intc_u256_as_bool(INTC_U64_TO_U256(true)) == true);
            INTC_TESTS_ASSERT(intc_u256_as_bool(INTC_U64_TO_U256(false)) == false);
            INTC_TESTS_ASSERT(intc_u256_as_u8(val)  == (intc_u8)0xaaULL);
            INTC_TESTS_ASSERT(intc_u256_as_u16(val) == (intc_u16)0xaaaaULL);
            INTC_TESTS_ASSERT(intc_u256_as_u32(val) == (intc_u32)0xaaaaaaaaULL);
            INTC_TESTS_ASSERT(intc_u256_as_u64(val) == (intc_u64)0xaaaaaaaaaaaaaaaaULL);
            INTC_TESTS_END;
        }
    }

    printf("\n  xor.cpp\n");
    {
        {
            INTC_TESTS_BEGIN("BitWise.xor");
            struct intc_u256 t   = INTC_U64_TO_U256((bool)     true);
            struct intc_u256 f   = INTC_U64_TO_U256((bool)     false);
            struct intc_u256 u8  = INTC_U64_TO_U256((intc_u8)  0xaaULL);
            struct intc_u256 u16 = INTC_U64_TO_U256((intc_u16) 0xaaaaULL);
            struct intc_u256 u32 = INTC_U64_TO_U256((intc_u32) 0xaaaaaaaaULL);
            struct intc_u256 u64 = INTC_U64_TO_U256((intc_u64) 0xaaaaaaaaaaaaaaaa);
            struct intc_u256 const val = INTC_U256(INTC_U64_TO_U128(0xf0f0f0f0f0f0f0f0ULL), INTC_U64_TO_U128(0xf0f0f0f0f0f0f0f0ULL));

            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_xor(t  ,  val), INTC_U256(INTC_U64_TO_U128(0xf0f0f0f0f0f0f0f1ULL), INTC_U64_TO_U128(0xf0f0f0f0f0f0f0f0ULL))));
            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_xor(f  ,  val), INTC_U256(INTC_U64_TO_U128(0xf0f0f0f0f0f0f0f0ULL), INTC_U64_TO_U128(0xf0f0f0f0f0f0f0f0ULL))));
            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_xor(u8 ,  val), INTC_U256(INTC_U64_TO_U128(0xf0f0f0f0f0f0f05aULL), INTC_U64_TO_U128(0xf0f0f0f0f0f0f0f0ULL))));
            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_xor(u16,  val), INTC_U256(INTC_U64_TO_U128(0xf0f0f0f0f0f05a5aULL), INTC_U64_TO_U128(0xf0f0f0f0f0f0f0f0ULL))));
            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_xor(u32,  val), INTC_U256(INTC_U64_TO_U128(0xf0f0f0f05a5a5a5aULL), INTC_U64_TO_U128(0xf0f0f0f0f0f0f0f0ULL))));
            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_xor(u64,  val), INTC_U256(INTC_U64_TO_U128(0x5a5a5a5a5a5a5a5aULL), INTC_U64_TO_U128(0xf0f0f0f0f0f0f0f0ULL))));

            // zero
            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_xor(INTC_U256_ZERO, val), val));
            INTC_TESTS_END;
        }

        {
            INTC_TESTS_BEGIN("External.xor");
            bool     t   = true;
            bool     f   = false;
            intc_u8  u8  = 0xaaULL;
            intc_u16 u16 = 0xaaaaULL;
            intc_u32 u32 = 0xaaaaaaaaULL;
            intc_u64 u64 = 0xaaaaaaaaaaaaaaaaULL;
            struct intc_u256 const val = INTC_U256(INTC_U64_TO_U128(0xf0f0f0f0f0f0f0f0ULL), INTC_U64_TO_U128(0xf0f0f0f0f0f0f0f0ULL));

            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_xor(INTC_U64_TO_U256(t  ), val), INTC_U256(INTC_U64_TO_U128(0xf0f0f0f0f0f0f0f1ULL), INTC_U64_TO_U128(0xf0f0f0f0f0f0f0f0ULL))));
            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_xor(INTC_U64_TO_U256(f  ), val), INTC_U256(INTC_U64_TO_U128(0xf0f0f0f0f0f0f0f0ULL), INTC_U64_TO_U128(0xf0f0f0f0f0f0f0f0ULL))));
            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_xor(INTC_U64_TO_U256(u8 ), val), INTC_U256(INTC_U64_TO_U128(0xf0f0f0f0f0f0f05aULL), INTC_U64_TO_U128(0xf0f0f0f0f0f0f0f0ULL))));
            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_xor(INTC_U64_TO_U256(u16), val), INTC_U256(INTC_U64_TO_U128(0xf0f0f0f0f0f05a5aULL), INTC_U64_TO_U128(0xf0f0f0f0f0f0f0f0ULL))));
            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_xor(INTC_U64_TO_U256(u32), val), INTC_U256(INTC_U64_TO_U128(0xf0f0f0f05a5a5a5aULL), INTC_U64_TO_U128(0xf0f0f0f0f0f0f0f0ULL))));
            INTC_TESTS_ASSERT(intc_u256_eq(intc_u256_xor(INTC_U64_TO_U256(u64), val), INTC_U256(INTC_U64_TO_U128(0x5a5a5a5a5a5a5a5aULL), INTC_U64_TO_U128(0xf0f0f0f0f0f0f0f0ULL))));
            INTC_TESTS_END;
        }
    }

    fputc('\n', stdout);
    return result;
}
#endif // !defined(INTC_NO_U256)

void intc_test_state_print(char const *label, struct intc_test_state const *state)
{
    printf(INTC_TESTS_COLOR_MAGENTA "%s" INTC_TESTS_COLOR_RESET " -- %d/%d tests passsed", label, state->test_count - state->fail_count, state->test_count);
    if (state->fail_count)
        printf(INTC_TESTS_COLOR_RED " (%d tests failed)" INTC_TESTS_COLOR_RESET, state->fail_count);
    fputc('\n', stdout);
}

void intc_unit_tests(void)
{
    struct intc_test_state u128_state = intc_u128_unit_tests();
#if !defined(INTC_NO_U256)
    struct intc_test_state u256_state = intc_u256_unit_tests();
#endif

    intc_test_state_print("intc_u128_unit_tests", &u128_state);

#if !defined(INTC_NO_U256)
    intc_test_state_print("intc_u256_unit_tests", &u256_state);
#endif
}

#if defined(INTC_TESTS_WITH_MAIN)
int main(void)
{
    intc_unit_tests();
    return 0;
}
#endif
#endif // INTC_TESTS_IMPLEMENTATION
