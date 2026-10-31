#!/usr/bin/env bash
# Builds SDL_shadercross's command line (zlib licence), the tool that makes the player's shaders
# ahead as SPIR-V and MSL (README.md), on Linux (on Windows, from WSL), into vpx-build/shadercross
# (git-ignored):
#
#   1. DXC, Microsoft's HLSL compiler: its own release build (the one SDL_shadercross's scripts
#      fetch), checked against its SHA-256.
#   2. SPIRV-Cross, which turns SPIR-V into MSL, and SDL 3 from our own copy (vendor/sdl3-src),
#      each built from source at the commit below.
#   3. SDL_shadercross at the commit below, against them.
#
# Then: tools/shaders/precompile.sh. Safe to re-run: what's built is kept. Needs git, curl, CMake,
# Ninja and a C++ compiler.

set -euo pipefail

SHADERCROSS_COMMIT=1ff05bec573988a98ef9e0260b4da44f512b8367
SPIRV_CROSS_COMMIT=57e4efcc84112abcd426dc5ec248b5339bee04f2
DXC_LINUX_URL=https://github.com/microsoft/DirectXShaderCompiler/releases/download/v1.9.2602/linux_dxc_2026_02_20.x86_64.tar.gz
DXC_LINUX_SHA256=a1d3e3b5e1c5685b3eb27d5e8890e41d87df45def05112a2d6f1a63a931f7d60

repo=$(cd "$(dirname "$0")/../.." && pwd)
root="$repo/vpx-build/shadercross"
prefix="$root/prefix"
jobs=$(getconf _NPROCESSORS_ONLN 2>/dev/null || echo 4)
mkdir -p "$root"
cd "$root"

# a repository at a commit, fetched alone
checkout() { # url dir commit
    if [ ! -d "$2/.git" ]; then
        git init -q "$2"
        git -C "$2" remote add origin "$1"
    fi
    if [ "$(git -C "$2" rev-parse HEAD 2>/dev/null)" != "$3" ]; then
        git -C "$2" fetch -q --depth 1 origin "$3"
        git -C "$2" checkout -q --detach FETCH_HEAD
    fi
}

cmake_install() { # source build [options...]
    local source=$1 build=$2
    shift 2
    cmake -S "$source" -B "$build" -G Ninja -DCMAKE_BUILD_TYPE=Release -DCMAKE_INSTALL_PREFIX="$prefix" -DCMAKE_PREFIX_PATH="$prefix" "$@" > /dev/null
    cmake --build "$build" --parallel "$jobs" > /dev/null
    cmake --install "$build" > /dev/null
}

checkout https://github.com/libsdl-org/SDL_shadercross.git SDL_shadercross "$SHADERCROSS_COMMIT"
checkout https://github.com/KhronosGroup/SPIRV-Cross.git SPIRV-Cross "$SPIRV_CROSS_COMMIT"

# 1. DXC
if [ "$(uname -s)" != Linux ]; then
    echo "Linux only so far (on Windows, from WSL)" >&2
    exit 1
fi
if [ ! -f dxc/lib/libdxcompiler.so ]; then
    curl -sSfL -o dxc.tar.gz "$DXC_LINUX_URL"
    echo "$DXC_LINUX_SHA256  dxc.tar.gz" | sha256sum -c - > /dev/null
    mkdir -p dxc
    tar xzf dxc.tar.gz -C dxc
fi

# 2. SPIRV-Cross and SDL
[ -f "$prefix/lib/cmake/SDL3/SDL3Config.cmake" ] || cmake_install "$repo/vendor/sdl3-src/SDL" build-sdl -DSDL_TESTS=OFF -DSDL_EXAMPLES=OFF
[ -d "$prefix/share/spirv_cross_c_shared" ] || cmake_install SPIRV-Cross build-spirv-cross -DSPIRV_CROSS_SHARED=ON -DSPIRV_CROSS_ENABLE_TESTS=OFF

# 3. SDL_shadercross
cmake_install SDL_shadercross build-shadercross -DSDLSHADERCROSS_INSTALL=ON \
    -DDirectXShaderCompiler_INCLUDE_PATH="$root/dxc/include/dxc" \
    -DDirectXShaderCompiler_dxcompiler_LIBRARY="$root/dxc/lib/libdxcompiler.so" \
    -DDirectXShaderCompiler_dxil_LIBRARY="$root/dxc/lib/libdxil.so"
cp dxc/lib/*.so "$prefix/lib/"

echo "built: $prefix/bin/shadercross"
