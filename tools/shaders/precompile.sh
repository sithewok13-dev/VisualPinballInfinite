#!/usr/bin/env bash
# Makes the player's shaders ahead again (README.md), with the shadercross setup.sh built.
set -euo pipefail
repo=$(cd "$(dirname "$0")/../.." && pwd)
prefix="$repo/vpx-build/shadercross/prefix"
export SHADERCROSS="$prefix/bin/shadercross"
export LD_LIBRARY_PATH="$prefix/lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
export DYLD_LIBRARY_PATH="$prefix/lib${DYLD_LIBRARY_PATH:+:$DYLD_LIBRARY_PATH}"
cd "$repo"
cargo run -q -p player --example precompile_shaders
cargo test -q -p player render::shaders
