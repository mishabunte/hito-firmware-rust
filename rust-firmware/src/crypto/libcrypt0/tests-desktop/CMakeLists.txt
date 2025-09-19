cmake_minimum_required(VERSION 3.12)

project(libcrypt0-desktop)

# dogshit for enable output in testing
if (CMAKE_CONFIGURATION_TYPES)
    add_custom_target(check COMMAND ${CMAKE_CTEST_COMMAND}
        --force-new-ctest-process --output-on-failure
        --build-config "$<CONFIGURATION>")
else()
    add_custom_target(check COMMAND ${CMAKE_CTEST_COMMAND}
        --force-new-ctest-process --output-on-failure)
endif()

# Set the output directory for the built executable and libraries
set(CMAKE_RUNTIME_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/build)
set(CMAKE_LIBRARY_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/build)
set(CMAKE_ARCHIVE_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/build)

set(CMAKE_CXX_STANDARD 14)

set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -Wall -Werror")

# Enable compile_commands.json generation
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

# Set the OpenSSL search path
set(OPENSSL_ROOT_DIR /opt/homebrew/opt/openssl)

# Find OpenSSL package
find_package(OpenSSL REQUIRED)

include_directories( 
    src
    include
    lib/secp256k1/include
    lib/secp256k1/src
    lib/SHA3IUF
    lib/ripemd160
    lib/bech32
    lib/intc
)

# Set source files
set(SOURCES
    src/crypt0.c
    src/crypt0_rlp.c
    src/crypt0_log.c
    src/crypt0_sha.c
    src/crypt0_hmac.c
    src/crypt0_ripemd160.c
    src/crypt0_bech32.c
    src/crypt0_pbkdf2.c
    src/crypt0_bip32.c
    src/crypt0_bip39.c
    src/crypt0_secp256k1.c
    src/crypt0_ed25519.c
    src/crypt0_aes_ccm.c
    src/crypt0_ed25519.c
    #'DumpHex.c'
    src/intc_impl.c
    lib/SHA3IUF/sha3.c
    lib/base58/base58.c
    lib/DumpHex/DumpHex.c
    lib/bech32/segwit_addr.c
    lib/secp256k1/src/secp256k1.c
    lib/secp256k1/src/precomputed_ecmult.c
    lib/secp256k1/src/precomputed_ecmult_gen.c
)


add_definitions(-DENABLE_MODULE_RECOVERY)# -DSTATIC_IN_RELEASE=static)
add_definitions(-DENABLE_MODULE_ECDH)
add_definitions(-DINTC_NO_CPP_FEATURES)

# Set header files
set(HEADERS
    include/crypt0.h
)

# Create a static library from source files
add_library(crypt0 OBJECT ${SOURCES} ${HEADERS})

# Link the library with OpenSSL
target_link_libraries(crypt0 OpenSSL::SSL)

# Set the executable target and link it with the library
add_executable(test-aes-ccm src-tests/test_aes_ccm.cpp)
target_link_libraries(test-aes-ccm crypt0)

add_executable(test-extended-pubkey src-tests/test_extended_pubkey.cpp)
target_link_libraries(test-extended-pubkey crypt0)

add_executable(test-key-hashing src-tests/test_key_hashing.cpp)
target_link_libraries(test-key-hashing crypt0)

add_executable(test-segwit-addr lib/bech32/tests.c)
target_link_libraries(test-segwit-addr crypt0)

# Add a test for the library
add_test(NAME Test-AES-CCM COMMAND test-aes-ccm)
#add_test(NAME Test-Near COMMAND test-near)
add_test(NAME Test-Key-Hashing COMMAND test-key-hashing)
add_test(NAME Test-Extended-Pubkey COMMAND test-extended-pubkey)
add_test(NAME Test-Segwit-Addr COMMAND test-segwit-addr)

enable_testing()

