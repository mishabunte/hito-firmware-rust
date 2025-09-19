set(MyLibrary_INCLUDE_DIRS "include")
set(MyLibrary_LIBRARIES "lib")

export(TARGETS crypt0
       FILE "${CMAKE_CURRENT_BINARY_DIR}/crypt0Targets.cmake")
export(PACKAGE crypt0)