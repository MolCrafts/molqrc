#!/usr/bin/env bash
#
# Verifies that the compiled WebAssembly build encodes identically to the native
# library, module for module.
#
# The wasm crate is a thin adapter, but "thin" is an assumption worth testing:
# a wasm-bindgen or wasm-opt regression would silently corrupt the grid handed
# to downstream JavaScript while every native test stays green. This builds the
# nodejs target and diffs it against native output for a spread of inputs
# (empty, numeric, alphanumeric, URL, CJK, emoji, long) across all four ECC
# levels.
#
# Requires: wasm-pack, node.

set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."

work="$(mktemp -d)"
trap 'rm -rf "${work}"' EXIT

echo "Building nodejs wasm target ..."
wasm-pack build wasm --release --target nodejs \
	--out-dir "${work}/pkg" --out-name molqrc >/dev/null 2>&1

echo "Dumping native reference output ..."
cargo run --quiet --release --example dump_reference > "${work}/native_ref.json"

echo "Comparing ..."
node scripts/wasm-parity.mjs "${work}/pkg/molqrc.js" "${work}/native_ref.json"
