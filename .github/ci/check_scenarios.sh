#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-only
#
# Render every scenario in tests/scenarios with the firmware in the emulator and compare the
# hashes with the shipped v1.71 reference (tests/scenarios/reference_v1.71.json).
#
#   .github/ci/check_scenarios.sh <lxr-emu binary> (--elf <fw.elf> | --img <LXRV2_update_v1.71.img>) [jobs]
#
# With --img every scenario must match. With --elf (the GCC 11.3 source build) the scenarios in
# KNOWN_ELF_DIFFS may differ (docs/ENGINE_CONTRACT.md section 9: `global_morph_no_target` reads
# ITCM code bytes, which differ between the two builds). Any other difference, a scenario missing
# from the reference ("NEW") or a render error fails.
set -euo pipefail

KNOWN_ELF_DIFFS=(global_morph_no_target)

emu=$1
kind=$2
fw=$3
jobs=${4:-$(getconf _NPROCESSORS_ONLN 2>/dev/null || echo 2)}
root=$(cd "$(dirname "$0")/../.." && pwd)
ref=$root/tests/scenarios/reference_v1.71.json

out=$(mktemp)
trap 'rm -f "$out"' EXIT
set +e
"$emu" check "$kind" "$fw" "$ref" "$root/tests/scenarios" --jobs "$jobs" | tee "$out"
rc=${PIPESTATUS[0]}
set -e
if [ "$rc" -ne 0 ] && [ "$rc" -ne 1 ]; then
    echo "::error::lxr-emu check failed with exit code $rc"
    exit "$rc"
fi
if ! grep -qE '^[0-9]+ scenarios, [0-9]+ differ$' "$out"; then
    echo "::error::lxr-emu check printed no summary (exit code $rc)"
    exit 1
fi

fail=0
if grep -q '^NEW ' "$out"; then
    echo "::error::scenarios without a reference hash (regenerate reference_v1.71.json with the shipped image)"
    fail=1
fi
while read -r _ name _; do
    name=${name%:}
    allowed=0
    if [ "$kind" = "--elf" ]; then
        for k in "${KNOWN_ELF_DIFFS[@]}"; do [ "$k" = "$name" ] && allowed=1; done
    fi
    if [ "$allowed" -eq 1 ]; then
        echo "known difference of the GCC 11.3 build: $name"
    else
        echo "::error::scenario $name differs from the v1.71 reference"
        fail=1
    fi
done < <(grep '^DIFF ' "$out" || true)

# Every reference entry must have been rendered.
want=$(grep -cE '^\s*"[^"]+": "[0-9a-f]{64}"' "$ref")
got=$(sed -nE 's/^([0-9]+) scenarios, [0-9]+ differ$/\1/p' "$out")
if [ "$want" != "$got" ]; then
    echo "::error::rendered $got scenarios, the reference has $want"
    fail=1
fi
exit "$fail"
