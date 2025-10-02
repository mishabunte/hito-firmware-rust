cmake_minimum_required(VERSION 3.13.1)

set(CONF_FILE prj.conf)

find_package(Zephyr REQUIRED HINTS $ENV{ZEPHYR_BASE})
project(hito_firmware_zephyr)

# Directory variables
set(RUST_FIRMWARE_DIR ${CMAKE_CURRENT_SOURCE_DIR}/rust-firmware)
set(RUST_SOURCE_DIR ${RUST_FIRMWARE_DIR}/src)
set(ZEPHYR_DRIVERS_DIR ${RUST_SOURCE_DIR}/drivers/zephyr/zephyr_drivers)

# Build configuration variables
set(RUST_TARGET thumbv7em-none-eabi)
set(RUST_BUILD_TYPE release)
set(RUST_TARGET_DIR ${RUST_FIRMWARE_DIR}/target/${RUST_TARGET}/${RUST_BUILD_TYPE})
set(RUST_LIB_NAME libhito_firmware_rust.a)
set(RUST_LIB_PATH ${RUST_TARGET_DIR}/${RUST_LIB_NAME})

# Build features and flags
set(RUST_FEATURES zephyr)
set(RUST_BUILD_FLAGS --target ${RUST_TARGET} --${RUST_BUILD_TYPE} --no-default-features --features ${RUST_FEATURES})

# Glob all Rust source files to track dependencies
file(GLOB_RECURSE RUST_SOURCES 
    "${RUST_SOURCE_DIR}/*.rs"
    "${RUST_SOURCE_DIR}/vault/*.rs"
    "${RUST_SOURCE_DIR}/vault/vault.c"
    "${RUST_FIRMWARE_DIR}/build.rs"
    "${RUST_FIRMWARE_DIR}/Cargo.toml"
    "${RUST_SOURCE_DIR}/ui/*.slint"
)

# Custom command to build Rust library when source files change
add_custom_command(
    OUTPUT ${RUST_LIB_PATH}
    COMMAND cargo build ${RUST_BUILD_FLAGS}
    WORKING_DIRECTORY ${RUST_FIRMWARE_DIR}
    DEPENDS ${RUST_SOURCES}
    COMMENT "Building Rust library for ${RUST_TARGET}"
    VERBATIM
)

# Custom target to ensure Rust library is built
add_custom_target(build_rust_lib DEPENDS ${RUST_LIB_PATH})

# Import the built Rust static library
add_library(hito_firmware_rust STATIC IMPORTED)
set_target_properties(hito_firmware_rust PROPERTIES 
    IMPORTED_LOCATION ${RUST_LIB_PATH})

# Make sure the Rust library is built before the imported library is used
add_dependencies(hito_firmware_rust build_rust_lib)

# C driver source files
set(DRIVER_SOURCES
  ${ZEPHYR_DRIVERS_DIR}/ili9342_lcd.c
  ${ZEPHYR_DRIVERS_DIR}/ft6336_ctp.c
  ${ZEPHYR_DRIVERS_DIR}/hito_pin_config.c
  ${ZEPHYR_DRIVERS_DIR}/timer.c
  ${ZEPHYR_DRIVERS_DIR}/hito_battery.c
)

# Main application sources
set(SOURCE_FILES
  src/main.c
  ${DRIVER_SOURCES}
)

# Add libcrypt0 and related C sources
set(LIBCRYPT0_DIR ${RUST_SOURCE_DIR}/crypto/libcrypt0)
set(LIBCRYPT0_SOURCES
  ${LIBCRYPT0_DIR}/src/crypt0.c
  ${LIBCRYPT0_DIR}/src/crypt0_aes_ccm.c
  ${LIBCRYPT0_DIR}/src/crypt0_bech32.c
  ${LIBCRYPT0_DIR}/src/crypt0_bip32.c
  ${LIBCRYPT0_DIR}/src/crypt0_bip39.c
  ${LIBCRYPT0_DIR}/src/crypt0_crc.c
  ${LIBCRYPT0_DIR}/src/crypt0_ed25519.c
  ${LIBCRYPT0_DIR}/src/crypt0_hmac.c
  ${LIBCRYPT0_DIR}/src/crypt0_hw_keys.c
  ${LIBCRYPT0_DIR}/src/crypt0_key.c
  ${LIBCRYPT0_DIR}/src/crypt0_log.c
  ${LIBCRYPT0_DIR}/src/crypt0_pbkdf2.c
  ${LIBCRYPT0_DIR}/src/crypt0_ripemd160.c
  ${LIBCRYPT0_DIR}/src/crypt0_rlp.c
  ${LIBCRYPT0_DIR}/src/crypt0_rng.c
  ${LIBCRYPT0_DIR}/src/crypt0_secp256k1.c
  ${LIBCRYPT0_DIR}/src/crypt0_sha.c
  ${LIBCRYPT0_DIR}/src/intc_impl.c
  ${RUST_SOURCE_DIR}/crypto/crc16_ccitt/crc16_ccitt.c
  ${RUST_SOURCE_DIR}/vault/vault.c
)

# Include directories for the C drivers and crypto libs
target_include_directories(app PRIVATE 
  ${ZEPHYR_DRIVERS_DIR}
  ${LIBCRYPT0_DIR}/include
  ${LIBCRYPT0_DIR}/lib/secp256k1/src
  ${LIBCRYPT0_DIR}/lib/secp256k1/include
  ${LIBCRYPT0_DIR}/lib/ripemd160
  ${LIBCRYPT0_DIR}/lib/SHA3IUF
  ${LIBCRYPT0_DIR}/lib/base58
  ${LIBCRYPT0_DIR}/lib/bech32
  ${LIBCRYPT0_DIR}/lib/intc
  ${RUST_SOURCE_DIR}/crypto/crc16_ccitt
)

target_sources(app PRIVATE ${SOURCE_FILES} ${LIBCRYPT0_SOURCES})
target_link_libraries(app PUBLIC hito_firmware_rust)