set(CMAKE_SYSTEM_NAME Linux)
set(CMAKE_SYSTEM_PROCESSOR aarch64)

# Adjust these if your cross toolchain uses a different triple.
set(_ROOK_CROSS_TRIPLE "aarch64-linux-gnu")

set(CMAKE_C_COMPILER "${_ROOK_CROSS_TRIPLE}-gcc")
set(CMAKE_CXX_COMPILER "${_ROOK_CROSS_TRIPLE}-g++")

set(_ROOK_DEFAULT_SYSROOT "${CMAKE_CURRENT_LIST_DIR}/../../../var/sysroots/2026-01-04-debian-bookworm-12.12-arm64")

# Allow overriding sysroot path from the environment.
# Example:
#   export ROOK_PI_SYSROOT=/abs/path/to/sysroot
if(DEFINED ENV{ROOK_PI_SYSROOT} AND NOT "$ENV{ROOK_PI_SYSROOT}" STREQUAL "")
	set(CMAKE_SYSROOT "$ENV{ROOK_PI_SYSROOT}")
else()
	set(CMAKE_SYSROOT "${_ROOK_DEFAULT_SYSROOT}")
endif()

set(CMAKE_FIND_ROOT_PATH "${CMAKE_SYSROOT}")

set(CMAKE_FIND_ROOT_PATH_MODE_PROGRAM NEVER)
set(CMAKE_FIND_ROOT_PATH_MODE_LIBRARY ONLY)
set(CMAKE_FIND_ROOT_PATH_MODE_INCLUDE ONLY)
set(CMAKE_FIND_ROOT_PATH_MODE_PACKAGE ONLY)

# pkg-config is used to discover libcamera. The preset sets PKG_CONFIG_* env vars.
