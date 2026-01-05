cmake_minimum_required(VERSION 3.23)

project(rook_lw_libcamera_capture
  VERSION 0.1.0
  LANGUAGES C CXX
)

option(ROOK_LW_LIBCAMERA_BUILD_EXAMPLE "Build camera_capturer_test example" ON)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

add_library(rook_lw_libcamera_capture STATIC
  src/rook_lw_libcamera_capture_core.cpp
  src/rook_lw_libcamera_capture_ffi.cpp
)
add_library(rook::lw_libcamera_capture ALIAS rook_lw_libcamera_capture)

target_include_directories(rook_lw_libcamera_capture
  PUBLIC
    $<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}/include>
    $<INSTALL_INTERFACE:include>
)

target_compile_features(rook_lw_libcamera_capture PUBLIC cxx_std_17)

target_compile_definitions(rook_lw_libcamera_capture
  PRIVATE
)

include(GNUInstallDirs)

find_package(PkgConfig REQUIRED)
pkg_check_modules(LIBCAMERA REQUIRED IMPORTED_TARGET libcamera)

find_package(Threads REQUIRED)

target_link_libraries(rook_lw_libcamera_capture
  PUBLIC
    PkgConfig::LIBCAMERA
    Threads::Threads
)

if(ROOK_LW_LIBCAMERA_BUILD_EXAMPLE)
  add_executable(camera_capturer_test tools/camera_capturer_test/main.cpp)
  target_link_libraries(camera_capturer_test PRIVATE rook_lw_libcamera_capture)
endif()

install(TARGETS rook_lw_libcamera_capture
  EXPORT rook_lw_libcamera_captureTargets
  LIBRARY DESTINATION ${CMAKE_INSTALL_LIBDIR}
  ARCHIVE DESTINATION ${CMAKE_INSTALL_LIBDIR}
  RUNTIME DESTINATION ${CMAKE_INSTALL_BINDIR}
)

install(DIRECTORY include/
  DESTINATION ${CMAKE_INSTALL_INCLUDEDIR}
)

install(EXPORT rook_lw_libcamera_captureTargets
  NAMESPACE rook::
  DESTINATION ${CMAKE_INSTALL_LIBDIR}/cmake/rook_lw_libcamera_capture
)
