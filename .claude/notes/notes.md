# Notes

Evolving decisions captured by `/mol:note`.

## Conformance to upstream is the correctness argument (2026-11-16)

Supersedes the 2026-08-06 "byte-identical vendored file" decision. The user
asked for a refactor of molqrc in Rust that passes every corner case of Nayuki's
QR-Code-generator.
- **Engine:** now a modular rewrite in `src/`.
- **Correctness:** defined by agreement with `qrcodegen` itself, pulled in as a
  git dev-dependency pinned to upstream `bcfe1b0`. Do not use crates.io 1.8.0:
  it predates several upstream changes.
- **What `tests/conformance.rs` compares:** version, level, mask and every module
  on success; error variant, numbers and message on failure.
- **Cases:**
  - every capacity boundary (version × level × mode);
  - character-count field limits at 9/10 and 26/27;
  - every mask through `encode_codewords`;
  - sparse payloads where the N4 balance rule decides the mask (random dense
    payloads almost never reach it; found when an N4 mutant survived);
  - ECI widths;
  - mode detection;
  - mixed segments;
  - the trial distribution of Nayuki's removed `qrcodegen-batch-test.py`.
- **Panics:** `tests/panics.rs` checks panic-for-panic parity.
- **Mutation check:** 51 injected single-line bugs, each killed by the suite.

The fidelity script, `upstream.expected.diff`, `fmt.sh` and the Clippy allow-list
are gone. `cargo fmt --all` and full Clippy apply to the whole crate.

## Licence declaration (2026-08-07)

The project is licensed **file by file**: `MIT AND BSD-3-Clause`.

- `src/` (engine) — MIT, Copyright (c) Project Nayuki, as a derivative of
  upstream's library (updated 2026-11-16; was "byte for byte").
- Everything else — BSD-3-Clause, Copyright (c) 2024-2026 Roy Kid.

Texts live in `LICENSE-MIT` and `LICENSE-BSD-3-CLAUSE`; `LICENSE` explains the
split. Both `Cargo.toml` manifests declare `license = "MIT AND BSD-3-Clause"`,
including the wasm crate, whose compiled `.wasm` statically links the MIT engine.

Supersedes the 2026-08-06 decision to declare the crate BSD-3-Clause only.

## Two independent verification layers (2026-08-06)

1. `tests/conformance.rs` — output equals the reference implementation.
2. `verify-wasm-parity.sh` — the compiled wasm encodes identically to native
   (32 text×ECL cases × 3 APIs), catching wasm-bindgen/wasm-opt regressions that
   native tests cannot see.
