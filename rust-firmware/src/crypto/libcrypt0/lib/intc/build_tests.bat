@echo off

set script_dir=%~dp0
if not exist build mkdir build

where /q cl || (
    echo MSVC's cl compiler must be available on the path to build
    goto :eof
)

REM MSVC cl build
echo [SCRIPT] Building tests via cl to build\intc_tests_cpp_msvc.exe
pushd build
cl -nologo -fsanitize=address -W4 -O2 -D INTC_TESTS_WITH_MAIN -I %script_dir% -TP %script_dir%\intc_tests.c /Fe:intc_tests_cpp_msvc /link /DEBUG

echo [SCRIPT] Building tests via cl to build\intc_tests_c_msvc.exe
cl -nologo -fsanitize=address -W4 -O2 -D INTC_TESTS_WITH_MAIN -I %script_dir% -TC %script_dir%\intc_tests.c /Fe:intc_tests_c_msvc /link /DEBUG
popd

REM Optional clang-cl build if we have the compiler on the path
where /q clang-cl || goto :eof
echo [SCRIPT] Building tests via clang-cl to build\intc_tests_cpp_clang.exe
pushd build
clang-cl -nologo -fsanitize=address -W4 -O2 -D INTC_TESTS_WITH_MAIN -I %script_dir% -TP %script_dir%\intc_tests.c /Fe:intc_tests_cpp_clang /link /DEBUG
popd build

