# Notes

Evolving decisions captured by `/mol:note`.

## Fidelity to upstream is the correctness argument (2026-08-06)

`src/lib.rs` is byte-identical to Project Nayuki's `qrcodegen` (Rust), pinned at
commit `f40366c`. That identity — verified mechanically by
`scripts/verify-upstream-fidelity.sh` in CI — is what justifies trusting the
encoder, far more than the test suite does.

Consequences, all of which have bitten already:

- `cargo fmt --all` rewrites ~1080 of its 1288 lines. Use `./scripts/fmt.sh`.
  rustfmt's own `ignore` key is nightly-only, so the exclusion cannot live in
  `rustfmt.toml`.
- Clippy raises 6 style lints there. They are silenced package-wide in
  `Cargo.toml` under `[lints.clippy]` rather than fixed, because fixing any of
  them forks the file. New code in `wasm/` is linted strictly.
- Optimising the engine (e.g. the `Vec::remove(0)` in
  `reed_solomon_compute_remainder`, or a GF(256) lookup table) would also fork
  it. Performance work belongs in `wasm/` or in a separate module.

## Licence declaration (2026-08-07)

The project is licensed **file by file**: `MIT AND BSD-3-Clause`.

- `src/lib.rs` — MIT, Copyright (c) Project Nayuki. It is upstream's file byte
  for byte, not a derivative rewrite, so it keeps upstream's licence.
- Everything else — BSD-3-Clause, Copyright (c) 2024-2026 Roy Kid.

Texts live in `LICENSE-MIT` and `LICENSE-BSD-3-CLAUSE`; `LICENSE` explains the
split. Both `Cargo.toml` manifests declare `license = "MIT AND BSD-3-Clause"`,
including the wasm crate, whose compiled `.wasm` statically links the MIT engine.

Supersedes the 2026-08-06 decision to declare the crate BSD-3-Clause only.

## Two independent verification layers (2026-08-06)

1. `verify-upstream-fidelity.sh` — the algorithm is the reference implementation.
2. `verify-wasm-parity.sh` — the compiled wasm encodes identically to native
   (32 text×ECL cases × 3 APIs), catching wasm-bindgen/wasm-opt regressions that
   native tests cannot see.
