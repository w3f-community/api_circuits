cmake_minimum_required(VERSION 3.22)

project(rust_wrapper)

################################################################################

include(deps/lib_circuits/deps/cmake/coverage.cmake)
include(deps/lib_circuits/deps/cmake/options.cmake)
include(deps/lib_circuits/deps/cmake/sanitizers.cmake)
include(deps/lib_circuits/deps/cmake/compile_flags.cmake)
include(deps/lib_circuits/deps/cmake/export_libs.cmake)

################################################################################

option(lib_circuits_BUILD_TESTS "Build tests" OFF)  # default ON
add_subdirectory(deps/lib_circuits/)

################################################################################

add_subdirectory(src)

################################################################################
