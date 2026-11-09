#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-only
# Builds the VDrum Schwung module for Ableton Move: dist/vdrum-module.tar.gz, which extracts to
# vdrum/ (module.json, ui.js, help.json, dsp.so, empty kits/ and patterns/).
#
# Install on Move: copy vdrum/ to /data/UserData/schwung/modules/overtake/ (scp, or Schwung
# Manager's upload), then pick VDrum in Schwung's overtake menu.
#
# Needs the aarch64-unknown-linux-gnu Rust target and an aarch64 linker (aarch64-linux-gnu-gcc,
# Debian/Ubuntu package gcc-aarch64-linux-gnu). The binary needs glibc 2.34 or later on Move.
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"
target=aarch64-unknown-linux-gnu
export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER="${CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER:-aarch64-linux-gnu-gcc}"
cargo build --release --locked -p vdrum-move --target "$target"
out="$root/dist"
stage="$out/vdrum"
rm -rf "$stage"
mkdir -p "$stage/kits" "$stage/patterns"
cp crates/vdrum-move/module/module.json crates/vdrum-move/module/ui.js crates/vdrum-move/module/help.json "$stage/"
cp "target/$target/release/libvdrum_move.so" "$stage/dsp.so"
tar -czf "$out/vdrum-module.tar.gz" -C "$out" vdrum
echo "$out/vdrum-module.tar.gz"
