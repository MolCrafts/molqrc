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

molqrc is a QR Code generator library for Rust — a faithful port of
[Project Nayuki's qrcodegen](https://www.nayuki.io/page/qr-code-generator-library).
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

> Kanji mode is present in the `QrSegmentMode` enum but has no constructor —
> upstream's Rust port does not implement one. CJK text is encoded as UTF-8 in
> Byte mode, which is what virtually every QR reader expects.

## Correctness

`src/lib.rs` is not "based on" Project Nayuki's `qrcodegen` — it **is** that
file, byte for byte, pinned at upstream commit
[`f40366c`](https://github.com/nayuki/QR-Code-generator/blob/f40366c40d8d1956081f7ec643d240c02a81df52/rust/src/lib.rs).
Two checks run in CI to keep that claim honest:

```bash
./scripts/verify-upstream-fidelity.sh   # src/lib.rs == upstream, modulo recorded diffs
./scripts/verify-wasm-parity.sh         # compiled wasm == native, module for module
```

The first fails on any unrecorded edit to the QR algorithm — including an
accidental `cargo fmt`. The second encodes 32 text × ECC-level cases through the
compiled WebAssembly build and diffs every module against native output, so a
`wasm-bindgen` or `wasm-opt` regression cannot slip through while native tests
stay green.

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
cargo test --workspace                    # core library + WASM crate
cargo clippy --workspace --all-targets    # lints
./scripts/fmt.sh                          # format (NOT `cargo fmt --all`)
./scripts/verify-upstream-fidelity.sh     # src/lib.rs == upstream qrcodegen
./scripts/verify-wasm-parity.sh           # compiled wasm == native
```

> **Do not run `cargo fmt --all`.** It reformats ~1080 of `src/lib.rs`'s 1288
> lines and breaks the byte-identity with upstream. `./scripts/fmt.sh` formats
> everything else. rustfmt's own `ignore` key is nightly-only, which is why the
> exclusion lives in a script. For the same reason, Clippy's style lints on
> `src/lib.rs` are silenced in `Cargo.toml` rather than fixed — "fixing" one
> would fork the file from upstream.

The suite is migrated from the original C and Python test suites: GF(256)
arithmetic, Reed-Solomon divisor/remainder, bit-buffer packing, data-codeword
capacity, alignment-pattern positions, character-count bits, the segment
constructors (with exact bit/byte outputs), encoding behaviour, and SVG/ASCII
rendering.

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
| QR Code engine — `src/lib.rs` | **MIT**, Copyright (c) Project Nayuki | [LICENSE-MIT](LICENSE-MIT) |
| Everything else — tests, `wasm/`, `molqrc_web/`, `scripts/`, docs | **BSD-3-Clause**, Copyright (c) 2024-2026 Roy Kid | [LICENSE-BSD-3-CLAUSE](LICENSE-BSD-3-CLAUSE) |

`src/lib.rs` is not a derivative rewrite — it is Project Nayuki's file, byte for
byte, so it keeps its original MIT licence and its copyright/permission notice
is retained verbatim in the file header. Binary artifacts (the crates.io
package, the npm package, the `.wasm` files) contain both and are distributed
under `MIT AND BSD-3-Clause`.

See [LICENSE](LICENSE) for the full explanation.

<hr>

<div align="center">
<sub>Crafted with 💚 by <a href="https://github.com/MolCrafts">MolCrafts</a></sub>
</div>
