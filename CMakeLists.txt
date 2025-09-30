cmake_minimum_required(VERSION 3.13.1)

set(CONF_FILE prj.conf)

find_package(Zephyr REQUIRED HINTS $ENV{ZEPHYR_BASE})
project(hito_firmware_zephyr)

# Glob all Rust source files to track dependencies
file(GLOB_RECURSE RUST_SOURCES 
    "${CMAKE_CURRENT_SOURCE_DIR}/rust-firmware/src/*.rs"
    "${CMAKE_CURRENT_SOURCE_DIR}/rust-firmware/build.rs"
    "${CMAKE_CURRENT_SOURCE_DIR}/rust-firmware/Cargo.toml"
    "${CMAKE_CURRENT_SOURCE_DIR}/rust-firmware/src/ui/*.slint"
)

set(RUST_LIB_PATH ${CMAKE_CURRENT_SOURCE_DIR}/rust-firmware/target/thumbv7em-none-eabi/release/libhito_firmware_rust.a)

# Custom command to build Rust library when source files change
add_custom_command(
    OUTPUT ${RUST_LIB_PATH}
    COMMAND cargo build --target thumbv7em-none-eabi --release --no-default-features --features zephyr
    WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}/rust-firmware
    DEPENDS ${RUST_SOURCES}
    COMMENT "Building Rust library"
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

# Add C driver files that the Rust library depends on
set(SOURCE_FILES
  src/main.c
  rust-firmware/src/drivers/zephyr/zephyr_drivers/ili9342_lcd.c
  rust-firmware/src/drivers/zephyr/zephyr_drivers/ft6336_ctp.c
  rust-firmware/src/drivers/zephyr/zephyr_drivers/hito_pin_config.c
  rust-firmware/src/drivers/zephyr/zephyr_drivers/timer.c
  rust-firmware/src/drivers/zephyr/zephyr_drivers/hito_battery.c
)

# Include directories for the C drivers
target_include_directories(app PRIVATE 
    rust-firmware/src/drivers/zephyr/zephyr_drivers
)

target_sources(app PRIVATE ${SOURCE_FILES})
target_link_libraries(app PUBLIC hito_firmware_rust)