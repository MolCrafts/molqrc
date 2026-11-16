# Changelog

All notable changes to this project are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) (pre-1.0:
breaking changes may land in a minor release).

## [Unreleased]

### Changed

- **The engine is a modular Rust rewrite, no longer a vendored copy of
  `qrcodegen`.** The single 1288-line `src/lib.rs` is split by concern:
  segments, bit buffer, capacity tables, Reed-Solomon and interleaving, the
  module canvas, the mask penalty and the encoding pipeline. Output is unchanged:
  every symbol, error and panic is identical to Project Nayuki's `qrcodegen`.
- `QrSegment::data()` returns `&[bool]` instead of `&Vec<bool>`.
- `QrCode`, `QrSegment`, `QrSegmentMode`, `QrCodeEcc`, `Version`, `Mask` and
  `BitBuffer` implement `Debug` and `Hash` where they did not.
  - `DataTooLong` is now `Copy` and `PartialEq`.
  - `BitBuffer` is now `Default`.
- The engine keeps its MIT licence as a derivative of `qrcodegen`.

### Added

- **`tests/conformance.rs`** checks every case module for module against
  `qrcodegen`, a git dev-dependency pinned to upstream `bcfe1b0`. The cases:
  - every capacity boundary (version × ECC level × mode);
  - character-count field limits around versions 9/10 and 26/27, Kanji and ECI included;
  - every version × level × mask through `encode_codewords`;
  - degenerate payloads under automatic masking, and sparse payloads where
    the balance rule decides the mask;
  - ECI designator widths;
  - mode detection over the first 12288 code points;
  - mixed segment lists;
  - a seeded copy of Nayuki's cross-language batch tester.
- **`tests/panics.rs`** checks that invalid arguments panic exactly where
  `qrcodegen` panics.

### Removed

- `scripts/verify-upstream-fidelity.sh`, `scripts/upstream.expected.diff` and
  `scripts/fmt.sh`, together with the `test / upstream fidelity` CI job and the
  Clippy/rustc allow-list in `Cargo.toml`. `cargo fmt --all` and full Clippy now
  apply to the whole crate.
- Encoding tests that only checked `is_ok()`; the conformance suite covers them exactly.

## [0.2.0] - 2026-08-06

Complete rewrite: molqrc is now a Rust library with WebAssembly bindings. The
previous C core with Python `ctypes` bindings is gone.

### Changed — breaking

- **Rewritten in Rust.** The C library (`rs.c`, `qrc.c`, `matrix.c`,
  `segment.c`, `molqrc.c`), the `include/molqrc.h` header and the Python package
  (`QRCode`, `cli.py`, `export.py`, `_png.py`, `preview.py`, `web.py`) are
  removed. There is no `molqrc` CLI and no Python API in this release.
- The public surface is now `QrCode`, `QrSegment`, `QrSegmentMode`, `QrCodeEcc`,
  `Version`, `Mask`, `BitBuffer` and `DataTooLong`.
- `qr_svg` now emits the dark modules as a **single `<path>`** in a module-unit
  coordinate system, instead of one `<rect>` per module in pixel coordinates.
  Rendering is unchanged; the document is far smaller and resolution
  independent, and `scale` now only drives the `width`/`height` attributes.
  Consumers that parsed the previous per-module `<rect>` elements must adapt.
- Crate metadata: `repository`/`homepage` now point at `MolCrafts/molqrc` (the
  canonical repository) rather than the personal fork.
- **Licensing is now `MIT AND BSD-3-Clause`**, applied file by file: the QR
  engine (`src/lib.rs`) keeps Project Nayuki's original MIT licence, since it is
  upstream's file byte for byte rather than a derivative rewrite; everything
  else stays BSD-3-Clause. Previously the crate declared BSD-3-Clause alone.
  See [LICENSE](LICENSE), [LICENSE-MIT](LICENSE-MIT) and
  [LICENSE-BSD-3-CLAUSE](LICENSE-BSD-3-CLAUSE).
- `qr_ascii` / `render_ascii` no longer emit each row twice. Modules were
  already two characters wide, so the doubling made the preview render about
  twice as tall as it is wide; output is now one line per module row. This
  halves the line count for a given input.

### Added

- **WebAssembly bindings** (`molqrc-wasm`), published to npm as
  `@molcrafts/molqrc-wasm`: `qr_encode` (returning `QrResult` with side,
  version, mask and the module grid), `qr_modules`, `qr_side`, `qr_svg` and
  `qr_ascii`.
- **Interactive web designer** (`molqrc_web/index.html`) — theme presets, module
  styles, custom colours, optional title/subtitle, an advanced ECL/version/mask
  panel, and PNG/SVG download.
- **`scripts/verify-upstream-fidelity.sh`** — verifies that `src/lib.rs` is
  byte-identical to Project Nayuki's `qrcodegen` at pinned commit `f40366c`,
  failing CI on any unrecorded divergence (including an accidental `cargo fmt`).
- **`scripts/verify-wasm-parity.sh`** — encodes 32 text × ECC-level cases
  through the compiled WebAssembly build and diffs every module against native
  output, catching `wasm-bindgen`/`wasm-opt` regressions that native tests
  cannot see.
- **`scripts/fmt.sh`** — formats every Rust source except the vendored
  `src/lib.rs`.
- CI now runs formatting, Clippy (`-D warnings`), the fidelity guard and the
  wasm parity check; the release workflow gates publishing on them and publishes
  to both crates.io and npm.
- `wasm-opt -Oz` in the wasm release profile (~4% smaller binary).

### Fixed

- README no longer claims Kanji mode support. `QrSegmentMode::Kanji` exists in
  the vendored enum, but upstream's Rust port ships no constructor for it, so
  CJK text is encoded as UTF-8 in Byte mode.
- `.claude/notes/architecture.md` described the deleted C and Python modules;
  it now documents the Rust + WASM layout.
- The published crate no longer bundles the web demo's compiled `.wasm`,
  `CLAUDE.md` or `.DS_Store`.
- `cargo publish` now runs with `--locked`.

### Performance

- `render_svg`: one `<path>` instead of ~15k `<rect>` elements at version 40,
  and a single pre-sized buffer instead of a `format!` allocation per module.
- `render_ascii`: builds into one pre-sized buffer instead of allocating and
  cloning a `String` per row, and emits half as many rows.

### Notes

- `src/lib.rs` remains verbatim upstream code under the MIT License, with the
  notice retained in the file header and enforced by CI. See [LICENSE](LICENSE)
  for how the two licences apply.

## [0.1.1] - earlier

- UI polish and project description cleanup.

## [0.1.0] - earlier

- Initial release (C core + Python bindings).

[0.2.0]: https://github.com/MolCrafts/molqrc/releases/tag/v0.2.0
[0.1.1]: https://github.com/MolCrafts/molqrc/releases/tag/v0.1.1
[0.1.0]: https://github.com/MolCrafts/molqrc/releases/tag/v0.1.0
