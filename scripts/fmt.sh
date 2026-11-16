#!/usr/bin/env bash
#
# Formats every Rust source in this repo EXCEPT src/lib.rs.
#
# src/lib.rs is vendored byte-for-byte from Project Nayuki's qrcodegen and must
# stay that way (see scripts/verify-upstream-fidelity.sh). rustfmt would rewrite
# ~1080 of its 1288 lines. rustfmt's own `ignore` key is nightly-only, so the
# exclusion is expressed here instead.
#
#   ./scripts/fmt.sh            format in place
#   ./scripts/fmt.sh --check    verify formatting (used by CI)
#
# Do NOT run `cargo fmt --all` in this repo.

set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."

exec rustfmt --edition 2021 "$@" \
	src/tests.rs \
	wasm/src/lib.rs \
	examples/dump_reference.rs
