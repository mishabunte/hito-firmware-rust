# intc
A MIT licensed C/C++ uint128/256 type library, originally adapted from [calccrypto's uint128/256 library](https://github.com/calccrypto/uint256_t) turned into a primarily C first library and opt out C++ features when compiling in C++ mode.

The overarching motivation of the library can be summed up as follows

- Single header, drop in - no build configuration required.
- Minimal dependencies (only depends on stdbool.h when compiling in C).
- Zero allocations (in other libs typically when converting to strings).
- Ability to rename the API via the pre-processor for tighter integration into C code bases
- C++ features are optional and can be opt out via the pre-processor, i.e. operator overloading and constructors.

## Usage
Include `intc.h` and defined `INTC_IMPLEMENTATION` in one and only one file to enable the implementation in that translation unit.

```cpp
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
    printf("%.*s\n", string.size, string.str); // 64,000,000,000,000
#else
    intc_u256 value = INTC_U64_TO_U256(32);
    value = intc_u256_add_u64(value, 32);
    if (intc_u256_eq_u64(value, 64))
        value = intc_u256_mul(value, INTC_U64_TO_U256(1'000'000'000'000));
    intc_u256_string string = intc_u256_readable_int_str(value);
    printf("%.*s\n", string.size, string.str); // 64,000,000,000,000
#endif
    return 0;
}
```

## Tests
Tests are adapted from calccrypto's test suite and can be built by running `build_tests.bat` for Windows and `build_tests.sh` for Unix. For Windows MSVC's cl compiler must be available on the path and GCC must be available on Unix.
