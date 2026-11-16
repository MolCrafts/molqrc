<div align="center">

<h1>
  <img src=".github/assets/moko.svg" alt="" height="48" align="absmiddle">
  &nbsp;molqrc
</h1>

<p><strong>High-quality QR Code generator library in Rust.</strong></p>

<p>
  <img src="https://img.shields.io/github/actions/workflow/status/MolCrafts/molqrc/test.yml?style=flat-square&logo=githubactions&logoColor=white&label=CI" alt="CI">
  <img src="https://img.shields.io/badge/rust-stable-orange?style=flat-square&logo=rust&logoColor=white" alt="Rust">
  <img src="https://img.shields.io/badge/license-MIT%20AND%20BSD--3--Clause-18432B?style=flat-square" alt="License">
</p>

<p>
  <a href="#quick-start"><b>Quick start</b></a> &nbsp;&middot;&nbsp;
  <a href="#molcrafts-ecosystem"><b>Ecosystem</b></a>
</p>

</div>

molqrc is a QR Code generator library for Rust, derived from
[Project Nayuki's qrcodegen](https://www.nayuki.io/page/qr-code-generator-library)
and conformance-tested against it.
It supports versions 1–40, all four error-correction levels, automatic mode
detection, manual mask selection, and outputs the raw module grid of the QR symbol.

## Capabilities

| Area | Capability |
|------|------------|
| Encoding | QR versions 1–40, automatic smallest-version selection within a range |
| Error correction | All four ECL levels (Low / Medium / Quartile / High), Reed-Solomon, optional ECL boost |
| Modes | Automatic mode detection — Numeric, Alphanumeric, Byte; explicit ECI |
| Masking | All 8 mask patterns, or automatic best-mask selection |
| Segment API | Mixed-mode encoding from explicit segments |
| Output | Raw modules via `qr.get_module(x, y)`; SVG / ASCII rendering in the WASM crate |

> Kanji mode is present in the `QrSegmentMode` enum but has no constructor,
> as in upstream's Rust library. CJK text is encoded as UTF-8 in
> Byte mode, which is what virtually every QR reader expects.

## Correctness

The engine is a modular Rust rewrite of Project Nayuki's
[`qrcodegen`](https://github.com/nayuki/QR-Code-generator). Correctness is
defined by agreement with that library, and `cargo test` checks it module for
module. The reference is a dev-dependency pinned to upstream commit
[`bcfe1b0`](https://github.com/nayuki/QR-Code-generator/tree/bcfe1b018cec5f1f91b5705eca5bb4cdd34e9f8d).

`tests/conformance.rs` runs every case through both libraries. They must return
the same version, ECC level, mask and modules, or the same error with the same
numbers. The cases are:

- **Capacity:** for every version × ECC level × mode (numeric, alphanumeric,
  byte), the longest payload that fits and one character more.
  The standard's absolute limits (7089 digits, 4296 alphanumeric, 2953 bytes)
  are also tested through `encode_text`/`encode_binary`.
- **Character-count fields:** 2<sup>n</sup>−1 and 2<sup>n</sup> characters for
  every mode, across the field-width steps at versions 9/10 and 26/27. This
  includes Kanji and ECI segments built with `QrSegment::new`.
- **Every version × level × mask** through `encode_codewords`, plus automatic
  masking of degenerate payloads (all 0x00, all 0xFF, 0xAA). Also 6000 sparse payloads
  where the dark/light balance rule decides the mask.
- **ECI designators** at each width edge (127/128, 16383/16384, 999999), segment
  factories and mode detection over the first 12288 code points, and random
  mixed segment lists.
- **A seeded copy of Nayuki's cross-language batch tester** (the trial
  distribution of the removed `qrcodegen-batch-test.py`).

`tests/panics.rs` checks that invalid arguments panic exactly where the
reference panics. Every test gate has been mutation-checked: injected bugs in
masks, format and version bits, Reed-Solomon, interleaving, capacity tables,
penalty rules and segment encoding each fail the suite.

`./scripts/verify-wasm-parity.sh` additionally encodes 32 text × ECC-level cases
through the compiled WebAssembly build and diffs every module against native
output. This catches a `wasm-bindgen` or `wasm-opt` regression that the native
tests cannot see.

## Install

Add to your `Cargo.toml`:

```toml
[dependencies]
molqrc = "0.1"
```

## Quick start

```rust
use molqrc::{QrCode, QrCodeEcc};

let qr = QrCode::encode_text("Hello, world!", QrCodeEcc::Medium).unwrap();
for y in 0..qr.size() {
    for x in 0..qr.size() {
        print!("{}", if qr.get_module(x, y) { "██" } else { "  " });
    }
    println!();
}
```

### Manual operation

```rust
use molqrc::{QrCode, QrCodeEcc, QrSegment, Version, Mask};

let text = "3141592653589793238462643383";
let segs = QrSegment::make_segments(text);
let qr = QrCode::encode_segments_advanced(
    &segs, QrCodeEcc::High,
    Version::new(5), Version::new(5), Some(Mask::new(2)), false,
).unwrap();
```

## WebAssembly

The `wasm/` crate (`molqrc-wasm`) exposes the generator to JavaScript via
`wasm-bindgen` and ships to npm as `@molcrafts/molqrc-wasm`:

```bash
npm install @molcrafts/molqrc-wasm
```

```js
import { qr_encode, qr_svg } from '@molcrafts/molqrc-wasm';

// Full control: ECC ordinal (0=L,1=M,2=Q,3=H), minimum version, mask
// (-1 = auto, 0..=7 = forced). No automatic ECC boost.
const res = qr_encode('Hello, world!', 1, 1, -1);
res.side;     // side length in modules
res.version;  // 1..=40
res.mask;     // 0..=7
res.modules;  // Uint8Array, row-major, 1 = dark — length side*side

// Standalone SVG (scale = pixels per module, 4-module quiet zone).
const svg = qr_svg('Hello, world!', 1, 8);
```

| Export | Returns |
|--------|---------|
| `qr_encode(text, ecl, versionMin, maskMode)` | `QrResult { side, version, mask, modules }` |
| `qr_modules(text, ecl)` | `Uint8Array` row-major grid (ECC boost enabled) |
| `qr_side(text, ecl)` | side length in modules |
| `qr_svg(text, ecl, scale)` | standalone SVG string |
| `qr_ascii(text, ecl, border)` | terminal preview — one line per module row, `##` = dark |

`qr_encode` honours the requested ECC level exactly; `qr_modules` / `qr_side` /
`qr_svg` / `qr_ascii` go through `encode_text`, which may raise the ECC level
when doing so is free. That is why the two can pick different masks for the same
input.

### Building

```bash
# npm package (bundler target)
wasm-pack build wasm --release --target bundler --scope molcrafts --out-name molqrc

# web demo (loaded directly as an ES module)
wasm-pack build wasm --release --target web --out-dir ../molqrc_web/pkg --out-name molqrc
```

## Web demo

Serve the page (a static server is required — ES modules and WebAssembly
do not load over `file://`):

```bash
python3 -m http.server -d molqrc_web 8000
# open http://localhost:8000
```

`molqrc_web/index.html` is an interactive QR designer that encodes text live in
the browser via the WASM core: theme presets, module styles (squares / rounded /
circles), custom colours, optional title/subtitle, an advanced panel (ECL,
minimum version, mask), and PNG (`toDataURL`) / SVG download.

## Development

```bash
cargo test --workspace                    # unit, conformance and WASM-crate tests
cargo clippy --workspace --all-targets    # lints
cargo fmt --all                           # format
./scripts/verify-wasm-parity.sh           # compiled wasm == native
```

The engine lives in `src/`, one module per concern:
- `segment.rs`, `bits.rs`: segments and bit packing;
- `capacity.rs`: version geometry and codeword tables;
- `reed_solomon.rs`: error correction and interleaving;
- `canvas.rs`: function patterns, codeword placement and masks;
- `penalty.rs`: automatic mask choice;
- `qrcode.rs`: the encoding pipeline.

`src/tests.rs` holds known-answer tests for the internals, with values from
Nayuki's C test suite.

## MolCrafts ecosystem

| Project | Role |
|---------|------|
| [molpy](https://github.com/MolCrafts/molpy)     | Python toolkit — the shared molecular data model & workflow layer |
| [molrs](https://github.com/MolCrafts/molrs)     | Rust core — molecular data structures & compute kernels (native + WASM) |
| [molpack](https://github.com/MolCrafts/molpack) | Packmol-grade molecular packing (Rust + Python) |
| [molvis](https://github.com/MolCrafts/molvis)   | WebGL molecular visualization & editing |
| [molexp](https://github.com/MolCrafts/molexp)   | Workflow & experiment-management platform |
| [molnex](https://github.com/MolCrafts/molnex)   | Molecular machine-learning framework |
| [molq](https://github.com/MolCrafts/molq)       | Unified job queue — local / SLURM / PBS / LSF |
| [molcfg](https://github.com/MolCrafts/molcfg)   | Layered configuration library |
| [mollog](https://github.com/MolCrafts/mollog)   | Structured logging, stdlib-compatible |
| [molhub](https://github.com/MolCrafts/molhub)   | Molecular dataset hub |
| [molmcp](https://github.com/MolCrafts/molmcp)   | MCP server for the ecosystem |
| [molrec](https://github.com/MolCrafts/molrec)   | Atomistic record specification |

## License

`MIT AND BSD-3-Clause` — licensed file by file:

| Scope | License | |
|-------|---------|--|
| QR Code engine — `src/` except `src/tests.rs` | **MIT**, Copyright (c) Project Nayuki | [LICENSE-MIT](LICENSE-MIT) |
| Everything else — `tests/`, `wasm/`, `molqrc_web/`, `scripts/`, `examples/`, docs | **BSD-3-Clause**, Copyright (c) 2024-2026 Roy Kid | [LICENSE-BSD-3-CLAUSE](LICENSE-BSD-3-CLAUSE) |

The engine is derived from Project Nayuki's `qrcodegen`, so it keeps that
library's MIT licence and copyright notice. Binary artifacts (the crates.io
package, the npm package, the `.wasm` files) contain both and are distributed
under `MIT AND BSD-3-Clause`.

See [LICENSE](LICENSE) for the full explanation.

<hr>

<div align="center">
<sub>Crafted with 💚 by <a href="https://github.com/MolCrafts">MolCrafts</a></sub>
</div>
