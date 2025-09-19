#!/bin/bash
script_dir="$(cd "$(dirname "$BASH_SOURCE")" && pwd)"
mkdir --parents build

# GCC build
echo "[SCRIPT] Building tests via gcc to build/intc_tests_cpp_gcc"
pushd build
g++ -O2 -D INTC_TESTS_WITH_MAIN -I ${script_dir} ${script_dir}/intc_tests.c -o intc_tests_cpp_gcc

echo "[SCRIPT] Building tests via gcc to build/intc_tests_c_gcc"
gcc -std=c99 -O2 -D INTC_TESTS_WITH_MAIN -I ${script_dir} ${script_dir}/intc_tests.c -o intc_tests_c_gcc
popd

