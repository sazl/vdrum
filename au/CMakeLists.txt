# SPDX-License-Identifier: GPL-3.0-only
#
# macOS only: wraps the VDrum CLAP (built by `cargo xtask bundle vdrum --release`) into an
# Audio Unit v2 (`VDrum.component`) and a standalone app (`VDrum.app`) with clap-wrapper.
# Both embed a copy of VDrum.clap in Contents/PlugIns.
#
#   cargo xtask bundle-universal vdrum --release      # or `bundle` for the native arch only
#   cmake -S au -B target/au -G Xcode -DVDRUM_CLAP=$PWD/target/bundled/VDrum.clap
#   cmake --build target/au --config Release
#
# UNTESTED: this project has not been built yet (no macOS machine/CI was available).

cmake_minimum_required(VERSION 3.21)
set(CMAKE_OSX_DEPLOYMENT_TARGET 10.13 CACHE STRING "Minimum macOS version")
set(CMAKE_OSX_ARCHITECTURES "x86_64;arm64" CACHE STRING "Architectures (must match the CLAP)")

project(vdrum-au VERSION 0.1.0 LANGUAGES C CXX)

if (NOT APPLE)
    message(FATAL_ERROR "The AUv2/standalone wrapper project is for macOS only")
endif ()
enable_language(OBJC)
enable_language(OBJCXX)

set(VDRUM_CLAP "${CMAKE_CURRENT_SOURCE_DIR}/../target/bundled/VDrum.clap" CACHE PATH "The VDrum.clap bundle to wrap")
if (NOT EXISTS "${VDRUM_CLAP}")
    message(FATAL_ERROR "VDRUM_CLAP (${VDRUM_CLAP}) does not exist; run `cargo xtask bundle vdrum --release` first")
endif ()

# clap-wrapper, pinned to the v0.9.1 tag. It downloads the CLAP SDK, the AudioUnit SDK and the
# other SDKs it needs (CPM) at configure time.
set(CLAP_WRAPPER_DOWNLOAD_DEPENDENCIES TRUE CACHE BOOL "" FORCE)
set(CLAP_WRAPPER_DONT_ADD_TARGETS TRUE CACHE BOOL "" FORCE)
include(FetchContent)
FetchContent_Declare(
    clap-wrapper
    GIT_REPOSITORY https://github.com/free-audio/clap-wrapper.git
    GIT_TAG eb8b1901defd9588fa4a5e66edb0d4c302760ff2 # v0.9.1
)
FetchContent_MakeAvailable(clap-wrapper)

# Audio Unit v2 instrument ('aumu'). The codes are VDrum's own (4 characters, the manufacturer code
# needs an upper case letter).
add_library(VDrum_auv2 MODULE)
target_add_auv2_wrapper(
    TARGET VDrum_auv2
    OUTPUT_NAME "VDrum"
    BUNDLE_IDENTIFIER "io.github.sazl.vdrum.auv2"
    BUNDLE_VERSION "${PROJECT_VERSION}"
    INSTRUMENT_TYPE "aumu"
    MANUFACTURER_NAME "VDrum"
    MANUFACTURER_CODE "VDrm"
    SUBTYPE_CODE "VDr1"
    MACOS_EMBEDDED_CLAP_LOCATION "${VDRUM_CLAP}"
)

# Standalone app hosting the CLAP.
add_executable(VDrum_standalone)
target_add_standalone_wrapper(
    TARGET VDrum_standalone
    OUTPUT_NAME "VDrum"
    BUNDLE_IDENTIFIER "io.github.sazl.vdrum.standalone"
    BUNDLE_VERSION "${PROJECT_VERSION}"
    HOSTED_CLAP_NAME "VDrum"
    PLUGIN_INDEX 0
    MACOS_EMBEDDED_CLAP_LOCATION "${VDRUM_CLAP}"
)
