//! WebAssembly bindings for [`molqrc`] — the Rust port of Project Nayuki's
//! `qrcodegen`.
//!
//! The pure `render_*` functions hold all logic (and are unit-tested natively);
//! the `#[wasm_bindgen]` wrappers are thin adapters that surface them to
//! JavaScript. `scripts/verify-wasm-parity.sh` checks that the compiled build
//! encodes identically to the native library, module for module.

use molqrc::{Mask, QrCode, QrCodeEcc, QrSegment, Version};
use std::fmt::Write as _;
use wasm_bindgen::prelude::*;

const QUIET_ZONE: usize = 4;

/// Maps an error-correction level ordinal (0=L, 1=M, 2=Q, 3=H) to [`QrCodeEcc`].
fn ecl_from_u8(ecl: u8) -> QrCodeEcc {
    match ecl {
        1 => QrCodeEcc::Medium,
        2 => QrCodeEcc::Quartile,
        3 => QrCodeEcc::High,
        _ => QrCodeEcc::Low,
    }
}

fn encode(text: &str, ecl: u8) -> Result<QrCode, String> {
    QrCode::encode_text(text, ecl_from_u8(ecl)).map_err(|e| e.to_string())
}

/// Returns `(side, modules)` where `modules` is a row-major grid of `side*side`
/// bytes (`1` = dark, `0` = light).
pub fn render_matrix(text: &str, ecl: u8) -> Result<(usize, Vec<u8>), String> {
    let qr = encode(text, ecl)?;
    let side = qr.size() as usize;
    let mut modules = Vec::with_capacity(side * side);
    for y in 0..qr.size() {
        for x in 0..qr.size() {
            modules.push(u8::from(qr.get_module(x, y)));
        }
    }
    Ok((side, modules))
}

/// Renders the QR Code as a standalone SVG string. Each module is `scale`
/// pixels and a 4-module quiet zone surrounds the symbol.
///
/// The dark modules are emitted as a single `<path>` over a module-unit
/// coordinate system rather than one `<rect>` per module. A version-40 symbol
/// therefore produces one element instead of ~15k, and `scale` only affects the
/// `width`/`height` attributes — the geometry itself stays resolution
/// independent, so the same document scales cleanly to any size.
pub fn render_svg(text: &str, ecl: u8, scale: u32) -> Result<String, String> {
    let scale = scale.max(1) as usize;
    let (side, modules) = render_matrix(text, ecl)?;
    let grid = side + 2 * QUIET_ZONE; // side length in modules
    let px = grid * scale; // side length in pixels

    // Roughly 12 bytes of path data per dark module, ~⅔ of modules are light.
    let mut out = String::with_capacity(256 + side * side * 8);
    out.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    let _ = writeln!(
        out,
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{px}\" height=\"{px}\" \
         viewBox=\"0 0 {grid} {grid}\" shape-rendering=\"crispEdges\">"
    );
    let _ = writeln!(
        out,
        "<rect width=\"{grid}\" height=\"{grid}\" fill=\"#fff\"/>"
    );
    out.push_str("<path fill=\"#000\" d=\"");
    for r in 0..side {
        let y = QUIET_ZONE + r;
        for c in 0..side {
            if modules[r * side + c] != 0 {
                let x = QUIET_ZONE + c;
                let _ = write!(out, "M{x} {y}h1v1h-1z");
            }
        }
    }
    out.push_str("\"/>\n</svg>");
    Ok(out)
}

/// Renders the QR Code as a multi-line ASCII string (`##` = dark, two spaces =
/// light), with `border` light modules of margin.
///
/// Each module is two characters wide and **one** row tall. Terminal cells are
/// roughly twice as tall as they are wide, so that renders close to square —
/// one line per module row, `2 * (side + 2 * border)` columns per line.
///
/// Intended for terminal preview and debugging. Whether it actually scans
/// depends on the terminal's font, spacing and colours; use [`render_svg`] for
/// output that has to be read by a scanner.
pub fn render_ascii(text: &str, ecl: u8, border: usize) -> Result<String, String> {
    let (side, modules) = render_matrix(text, ecl)?;
    const DARK: &str = "##";
    const LIGHT: &str = "  ";
    let full = side + 2 * border;

    // One pre-sized buffer: `full` rows of `full` two-character modules, each
    // followed by a newline.
    let mut out = String::with_capacity((full * 2 + 1) * full);
    let blank = LIGHT.repeat(full);
    let margin = LIGHT.repeat(border);

    for _ in 0..border {
        out.push_str(&blank);
        out.push('\n');
    }
    for r in 0..side {
        out.push_str(&margin);
        for c in 0..side {
            out.push_str(if modules[r * side + c] != 0 {
                DARK
            } else {
                LIGHT
            });
        }
        out.push_str(&margin);
        out.push('\n');
    }
    for _ in 0..border {
        out.push_str(&blank);
        out.push('\n');
    }
    out.pop(); // trailing newline
    Ok(out)
}

/// Encodes with explicit minimum version and mask control, returning
/// `(side, version, mask, modules)`. `mask_mode` < 0 selects the mask
/// automatically; `0..=7` forces a specific mask. The explicit error-correction
/// level is respected (no automatic boost).
pub fn render_encode(
    text: &str,
    ecl: u8,
    version_min: u8,
    mask_mode: i32,
) -> Result<(usize, u8, u8, Vec<u8>), String> {
    let segs = QrSegment::make_segments(text);
    let minv = Version::new(version_min.clamp(1, 40));
    let mask = if mask_mode < 0 {
        None
    } else {
        Some(Mask::new((mask_mode as u8).min(7)))
    };
    let qr =
        QrCode::encode_segments_advanced(&segs, ecl_from_u8(ecl), minv, Version::MAX, mask, false)
            .map_err(|e| e.to_string())?;
    let side = qr.size() as usize;
    let mut modules = Vec::with_capacity(side * side);
    for y in 0..qr.size() {
        for x in 0..qr.size() {
            modules.push(u8::from(qr.get_module(x, y)));
        }
    }
    Ok((side, qr.version().value(), qr.mask().value(), modules))
}

/*---- wasm-bindgen surface ----*/

fn js_err<T>(r: Result<T, String>) -> Result<T, JsValue> {
    r.map_err(|e| JsValue::from_str(&e))
}

/// Encoding result with metadata, returned to JavaScript by [`qr_encode`].
#[wasm_bindgen]
pub struct QrResult {
    side: usize,
    version: u8,
    mask: u8,
    modules: Vec<u8>,
}

#[wasm_bindgen]
impl QrResult {
    /// Side length in modules.
    #[wasm_bindgen(getter)]
    pub fn side(&self) -> usize {
        self.side
    }
    /// Chosen QR Code version (1–40).
    #[wasm_bindgen(getter)]
    pub fn version(&self) -> u8 {
        self.version
    }
    /// Applied mask pattern (0–7).
    #[wasm_bindgen(getter)]
    pub fn mask(&self) -> u8 {
        self.mask
    }
    /// Row-major module grid (`1`=dark, `0`=light), length `side*side`.
    #[wasm_bindgen(getter)]
    pub fn modules(&self) -> Vec<u8> {
        self.modules.clone()
    }
}

/// Encodes `text` with explicit ECL ordinal, minimum version, and mask mode
/// (`-1` = automatic, `0..=7` = forced), returning matrix + metadata.
#[wasm_bindgen]
pub fn qr_encode(
    text: &str,
    ecl: u8,
    version_min: u8,
    mask_mode: i32,
) -> Result<QrResult, JsValue> {
    let (side, version, mask, modules) = js_err(render_encode(text, ecl, version_min, mask_mode))?;
    Ok(QrResult {
        side,
        version,
        mask,
        modules,
    })
}

/// Row-major module grid (`1`=dark, `0`=light); the side length is the integer
/// square root of the returned length.
#[wasm_bindgen]
pub fn qr_modules(text: &str, ecl: u8) -> Result<Vec<u8>, JsValue> {
    js_err(render_matrix(text, ecl).map(|(_, m)| m))
}

/// QR Code side length in modules.
#[wasm_bindgen]
pub fn qr_side(text: &str, ecl: u8) -> Result<usize, JsValue> {
    js_err(render_matrix(text, ecl).map(|(s, _)| s))
}

/// Standalone SVG string (`scale` pixels per module, 4-module quiet zone).
#[wasm_bindgen]
pub fn qr_svg(text: &str, ecl: u8, scale: u32) -> Result<String, JsValue> {
    js_err(render_svg(text, ecl, scale))
}

/// Multi-line ASCII preview.
#[wasm_bindgen]
pub fn qr_ascii(text: &str, ecl: u8, border: usize) -> Result<String, JsValue> {
    js_err(render_ascii(text, ecl, border))
}

#[cfg(test)]
mod tests {
    use super::*;

    // Migrated from Python TestQRCode — encoding shape.
    #[test]
    fn matrix_hello_is_21x21() {
        let (side, modules) = render_matrix("hello", 0).unwrap();
        assert_eq!(side, 21);
        assert_eq!(modules.len(), 441);
        assert!(modules.contains(&1));
        assert!(modules.contains(&0));
    }

    // Migrated from Python TestExport.
    #[test]
    fn svg_has_structure_and_quiet_zone() {
        let svg = render_svg("hello", 0, 10).unwrap();
        assert!(svg.contains("<svg"));
        assert!(svg.contains("<rect"));
        assert!(svg.contains("viewBox"));
        // width = (21 + 8) * 10 = 290; output is square.
        assert!(svg.contains("width=\"290\""));
        assert!(svg.contains("height=\"290\""));
    }

    #[test]
    fn svg_scale_changes_dimensions() {
        let svg = render_svg("hello", 0, 5).unwrap();
        assert!(svg.contains("width=\"145\"")); // (21 + 8) * 5
    }

    #[test]
    fn svg_emits_one_path_for_all_dark_modules() {
        let (side, modules) = render_matrix("hello", 0).unwrap();
        let dark = modules.iter().filter(|&&b| b == 1).count();
        let svg = render_svg("hello", 0, 10).unwrap();

        // One <path> for the whole symbol, one <rect> for the background.
        assert_eq!(svg.matches("<path").count(), 1);
        assert_eq!(svg.matches("<rect").count(), 1);
        // Exactly one "M" subpath per dark module ('M' occurs nowhere else).
        assert_eq!(svg.matches('M').count(), dark);
        // Geometry lives in module units, so scale only drives width/height.
        let grid = side + 2 * QUIET_ZONE;
        assert!(svg.contains(&format!("viewBox=\"0 0 {grid} {grid}\"")));
    }

    #[test]
    fn svg_geometry_is_scale_invariant() {
        // Same path data at every scale — only the pixel dimensions differ.
        let d = |svg: &str| svg.split("d=\"").nth(1).unwrap().to_string();
        assert_eq!(
            d(&render_svg("hello", 0, 3).unwrap()),
            d(&render_svg("hello", 0, 17).unwrap())
        );
    }

    // Migrated from Python TestPreview.
    #[test]
    fn ascii_is_nonempty_ascii_with_dark_modules() {
        let out = render_ascii("hello", 0, 1).unwrap();
        assert!(out.contains("##"));
        assert!(out.is_ascii());

        // border=1 around a 21-module symbol: 23 rows, one line per module row.
        let lines: Vec<&str> = out.split('\n').collect();
        assert_eq!(lines.len(), 23);
        // Two characters per module, so every line is 23 * 2 = 46 columns.
        assert!(
            lines.iter().all(|l| l.len() == 46),
            "ragged rows: {lines:?}"
        );
    }

    #[test]
    fn ascii_rows_are_not_duplicated() {
        // Regression guard: the preview used to emit every row twice while
        // already being two characters wide per module, so it rendered about
        // twice as tall as it was wide.
        let out = render_ascii("hello", 0, 0).unwrap();
        let lines: Vec<&str> = out.split('\n').collect();
        assert_eq!(lines.len(), 21);
        // Row 0 is the top of the finder patterns, row 1 their inner border —
        // if rows were being duplicated these would be equal.
        assert_ne!(lines[0], lines[1]);
    }

    #[test]
    fn ascii_border_widens_and_heightens_symmetrically() {
        for border in 0..4 {
            let lines = render_ascii("hello", 0, border).unwrap();
            let lines: Vec<&str> = lines.split('\n').collect();
            assert_eq!(lines.len(), 21 + 2 * border);
            assert!(lines.iter().all(|l| l.len() == (21 + 2 * border) * 2));
        }
    }

    #[test]
    fn encode_respects_ecl_version_and_mask() {
        let (side, version, mask, modules) = render_encode("hello", 1, 1, -1).unwrap();
        assert_eq!(side, 21);
        assert_eq!(version, 1);
        assert!(mask <= 7);
        assert_eq!(modules.len(), 441);
        // Manual mask is honoured.
        let (.., mask3, _) = render_encode("hello", 1, 1, 3).unwrap();
        assert_eq!(mask3, 3);
        // Minimum version is honoured.
        let (_, v10, ..) = render_encode("hello", 1, 10, -1).unwrap();
        assert!(v10 >= 10);
    }

    #[test]
    fn invalid_input_is_error_not_panic() {
        // 3000 bytes exceeds every version's capacity.
        let big = "\u{0}".repeat(3000);
        assert!(render_matrix(&big, 3).is_err());
    }
}
