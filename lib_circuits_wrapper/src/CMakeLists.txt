################################################################################

# TODO fix
find_path(RUST_CXX_INCLUDE_DIR rust/cxx.h
    PATHS "${PROJECT_SOURCE_DIR}/../target/cxxbridge/"
)

message(VERBOSE "RUST_CXX_INCLUDE_DIR: ${RUST_CXX_INCLUDE_DIR}")

# mostly to integrate/include rust/cxx.h
add_library(rust_cxx INTERFACE)

target_include_directories(rust_cxx
    INTERFACE
    ${RUST_CXX_INCLUDE_DIR}
)

################################################################################

add_library(rust_wrapper STATIC
    rust_wrapper.cpp
)

target_include_directories(rust_wrapper
    PRIVATE
    # for #include "lib-circuits-wrapper/src/rust_wrapper.h"
    ${PROJECT_BINARY_DIR}/include
)

target_link_libraries(rust_wrapper
    PUBLIC
    rust_cxx

    circuit_lib
)

export_all_target_libs(rust_wrapper)
