<div align="center">

<h1>
  <img src=".github/assets/moko.svg" alt="" height="48" align="absmiddle">
  &nbsp;molqrc
</h1>

<p><strong>High-quality QR Code generator library in Rust.</strong></p>

<p>
  <img src="https://img.shields.io/github/actions/workflow/status/MolCrafts/molqrc/ci.yml?style=flat-square&logo=githubactions&logoColor=white&label=CI" alt="CI">
  <img src="https://img.shields.io/badge/rust-stable-orange?style=flat-square&logo=rust&logoColor=white" alt="Rust">
  <img src="https://img.shields.io/badge/license-BSD--3--Clause-18432B?style=flat-square" alt="License">
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
| Modes | Automatic mode detection — Numeric, Alphanumeric, Byte, Kanji; explicit ECI |
| Masking | All 8 mask patterns, or automatic best-mask selection |
| Segment API | Mixed-mode encoding from explicit segments |
| Output | Raw modules via `qr.get_module(x, y)`; see the demo for SVG / ASCII rendering |

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

## WebAssembly & web demo

The `wasm/` crate (`molqrc-wasm`) exposes the generator to JavaScript via
`wasm-bindgen`: `qr_modules`, `qr_side`, `qr_svg`, and `qr_ascii`.

Build the WebAssembly module into the web directory:

```bash
wasm-pack build wasm --target web --out-dir ../molqrc_web/pkg --out-name molqrc
```

Then serve the page (a static server is required — ES modules and WebAssembly
do not load over `file://`):

```bash
python3 -m http.server -d molqrc_web 8000
# open http://localhost:8000
```

`molqrc_web/index.html` is an interactive QR designer that encodes text live in
the browser via the WASM core: theme presets, module styles (squares / rounded /
circles), custom colours, optional title/subtitle, an advanced panel (ECL,
minimum version, mask), and PNG (`toDataURL`) / SVG download.

## Tests

```bash
cargo test            # core library + WASM crate
```

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

molqrc is licensed under the **BSD-3-Clause** license — see [LICENSE](LICENSE).

The QR Code engine (`src/lib.rs`) is a port of Project Nayuki's qrcodegen, which
upstream is distributed under the MIT License; that copyright and permission
notice is retained verbatim in the file's header.

<hr>

<div align="center">
<sub>Crafted with 💚 by <a href="https://github.com/MolCrafts">MolCrafts</a></sub>
</div>
