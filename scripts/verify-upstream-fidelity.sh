#!/usr/bin/env bash
#
# Verifies that src/lib.rs remains a faithful copy of Project Nayuki's
# qrcodegen (Rust).
#
# molqrc's correctness argument is not "we tested it" but "it *is* the
# reference implementation". That property is only worth anything if it is
# mechanically enforced, so this script fetches the pinned upstream revision
# and asserts that the only differences are the ones we deliberately keep
# (recorded in scripts/upstream.expected.diff).
#
# If you intentionally change src/lib.rs, regenerate the baseline:
#
#     ./scripts/verify-upstream-fidelity.sh --update
#
# and justify the new divergence in the commit message.

set -euo pipefail

# Upstream revision this port is pinned to. Bumping it is a deliberate act:
# re-run with --update and review the resulting diff line by line.
UPSTREAM_SHA="f40366c40d8d1956081f7ec643d240c02a81df52"
UPSTREAM_SHA256="19486a9fa8ba7923ad5fad8a34532eb5de07fcaceb2a4404aed267edd3d64109"
UPSTREAM_URL="https://raw.githubusercontent.com/nayuki/QR-Code-generator/${UPSTREAM_SHA}/rust/src/lib.rs"

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
local_lib="${repo_root}/src/lib.rs"
expected="${repo_root}/scripts/upstream.expected.diff"

tmp="$(mktemp -d)"
trap 'rm -rf "${tmp}"' EXIT

echo "Fetching upstream qrcodegen @ ${UPSTREAM_SHA:0:7} ..."
curl -sSfL -o "${tmp}/upstream.rs" "${UPSTREAM_URL}"

actual_sha256="$(shasum -a 256 "${tmp}/upstream.rs" | cut -d' ' -f1)"
if [[ "${actual_sha256}" != "${UPSTREAM_SHA256}" ]]; then
	echo "ERROR: upstream file checksum mismatch." >&2
	echo "  expected ${UPSTREAM_SHA256}" >&2
	echo "  actual   ${actual_sha256}" >&2
	echo "The pinned revision should be immutable; refusing to continue." >&2
	exit 2
fi

# The only blanket transformation we accept: upstream is published as the
# `qrcodegen` crate, we vendor it as `molqrc`.
sed 's/qrcodegen/molqrc/g' "${tmp}/upstream.rs" > "${tmp}/upstream.norm.rs"

# Drop the ---/+++ header lines, which carry local paths and mtimes.
diff -u "${tmp}/upstream.norm.rs" "${local_lib}" | tail -n +3 > "${tmp}/actual.diff" || true

if [[ "${1:-}" == "--update" ]]; then
	cp "${tmp}/actual.diff" "${expected}"
	echo "Baseline updated: scripts/upstream.expected.diff"
	exit 0
fi

if diff -u "${expected}" "${tmp}/actual.diff" > "${tmp}/delta" 2>&1; then
	echo "OK: src/lib.rs matches upstream qrcodegen @ ${UPSTREAM_SHA:0:7}"
	echo "    (only the recorded intentional divergences are present)"
	exit 0
fi

echo "ERROR: src/lib.rs diverges from upstream in unrecorded ways." >&2
echo >&2
echo "--- unexpected divergence (- = expected baseline, + = current) ---" >&2
cat "${tmp}/delta" >&2
echo >&2
echo "The QR algorithm must stay byte-identical to Project Nayuki's" >&2
echo "reference implementation. Either revert the edit, or -- if the change" >&2
echo "is deliberate -- run ./scripts/verify-upstream-fidelity.sh --update" >&2
echo "and explain the divergence in your commit message." >&2
exit 1
