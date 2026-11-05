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
# Built by .github/workflows/ci.yml and release.yml on GitHub's macOS runners.

cmake_minimum_required(VERSION 3.24)
set(CMAKE_OSX_DEPLOYMENT_TARGET 10.13 CACHE STRING "Minimum macOS version")
set(CMAKE_OSX_ARCHITECTURES "x86_64;arm64" CACHE STRING "Architectures (must match the CLAP)")

project(vdrum-au VERSION 0.1.0 LANGUAGES C CXX)

# The AudioUnit SDK that clap-wrapper downloads needs C++17 (std::byte, class template argument
# deduction); without this the Xcode generator builds it as C++98/11 and fails.
set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

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

# clap-wrapper compiles its own sources with -Werror. Its bundled fmt triggers deprecation warnings
# with the libc++ of current Xcode (char_traits<char8_type>, literal operator spacing), which
# would fail the build; keep them as warnings. -Wno-error is appended after clap-wrapper's -Werror.
if (TARGET clap-wrapper-compile-options)
    target_compile_options(clap-wrapper-compile-options INTERFACE -Wno-error)
endif ()

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
)

# Embed VDrum.clap in Contents/PlugIns of both bundles, where the wrappers look for it first.
# clap-wrapper's own MACOS_EMBEDDED_CLAP_LOCATION is not used: it copies relative to the
# LIBRARY_OUTPUT_DIRECTORY property, which is unset here, so with the Xcode generator the copy
# landed in the source folder (au/VDrum.component) instead of the built bundle.
foreach (tgt VDrum_auv2 VDrum_standalone)
    add_custom_command(TARGET ${tgt} POST_BUILD
        COMMAND ${CMAKE_COMMAND} -E rm -rf "$<TARGET_BUNDLE_DIR:${tgt}>/Contents/PlugIns/VDrum.clap"
        COMMAND ${CMAKE_COMMAND} -E make_directory "$<TARGET_BUNDLE_DIR:${tgt}>/Contents/PlugIns"
        COMMAND ${CMAKE_COMMAND} -E copy_directory "${VDRUM_CLAP}" "$<TARGET_BUNDLE_DIR:${tgt}>/Contents/PlugIns/VDrum.clap"
        VERBATIM)
endforeach ()
