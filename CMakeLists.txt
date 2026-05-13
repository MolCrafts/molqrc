cmake_minimum_required(VERSION 3.14)
project(molqrc LANGUAGES C)

set(CMAKE_C_STANDARD 99)
set(CMAKE_C_STANDARD_REQUIRED ON)

if(NOT CMAKE_BUILD_TYPE)
    set(CMAKE_BUILD_TYPE Debug)
endif()

if(CMAKE_C_COMPILER_ID MATCHES "GNU|Clang|AppleClang")
    add_compile_options(-Wall -Wextra -pedantic -Werror)
endif()

# Static library
add_library(molqrc STATIC
    src/rs.c
    src/qrc.c
    src/matrix.c
    src/segment.c
    src/molqrc.c
)

target_include_directories(molqrc PUBLIC include)

# Shared library (for Python ctypes bindings)
add_library(molqrc_shared SHARED
    src/rs.c
    src/qrc.c
    src/matrix.c
    src/segment.c
    src/molqrc.c
)

target_include_directories(molqrc_shared PUBLIC include)
set_target_properties(molqrc_shared PROPERTIES OUTPUT_NAME molqrc)

# Install shared library into Python package for scikit-build-core
install(TARGETS molqrc_shared LIBRARY DESTINATION molqrc)

# Tests
enable_testing()
add_executable(test_molqrc tests/test_molqrc.c)
target_link_libraries(test_molqrc molqrc)
target_include_directories(test_molqrc PRIVATE include src)
add_test(NAME molqrc_tests COMMAND test_molqrc)
