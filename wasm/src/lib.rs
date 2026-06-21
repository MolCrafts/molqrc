//! WebAssembly bindings for [`molqrc`] — the Rust port of Project Nayuki's
//! `qrcodegen`.
//!
//! The pure `render_*` functions hold all logic (and are unit-tested natively);
//! the `#[wasm_bindgen]` wrappers are thin adapters that surface them to
//! JavaScript. The SVG output mirrors the original molqrc Python exporter
//! (`scale` pixels per module, a 4-module quiet zone) and the ASCII output
//! mirrors the original terminal preview.

use molqrc::{Mask, QrCode, QrCodeEcc, QrSegment, Version};
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
pub fn render_svg(text: &str, ecl: u8, scale: u32) -> Result<String, String> {
    let scale = scale.max(1) as usize;
    let (side, modules) = render_matrix(text, ecl)?;
    let size = (side + 2 * QUIET_ZONE) * scale;

    let mut out = String::new();
    out.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    out.push_str(&format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{size}\" height=\"{size}\" viewBox=\"0 0 {size} {size}\">\n"
    ));
    out.push_str(&format!(
        "<rect width=\"{size}\" height=\"{size}\" fill=\"#fff\"/>\n"
    ));
    for r in 0..side {
        let y = (QUIET_ZONE + r) * scale;
        for c in 0..side {
            if modules[r * side + c] != 0 {
                let x = (QUIET_ZONE + c) * scale;
                out.push_str(&format!(
                    "<rect x=\"{x}\" y=\"{y}\" width=\"{scale}\" height=\"{scale}\" fill=\"#000\"/>\n"
                ));
            }
        }
    }
    out.push_str("</svg>");
    Ok(out)
}

/// Renders the QR Code as a multi-line ASCII string (`##` = dark, two spaces =
/// light), with `border` light modules of margin. Best-effort preview only.
pub fn render_ascii(text: &str, ecl: u8, border: usize) -> Result<String, String> {
    let (side, modules) = render_matrix(text, ecl)?;
    let (dark, light) = ("##", "  ");
    let full = side + 2 * border;
    let mut lines: Vec<String> = Vec::new();
    let mut push_row = |s: String| {
        lines.push(s.clone());
        lines.push(s);
    };
    for _ in 0..border {
        push_row(light.repeat(full));
    }
    for r in 0..side {
        let mut row = light.repeat(border);
        for c in 0..side {
            row.push_str(if modules[r * side + c] != 0 { dark } else { light });
        }
        row.push_str(&light.repeat(border));
        push_row(row);
    }
    for _ in 0..border {
        push_row(light.repeat(full));
    }
    Ok(lines.join("\n"))
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
    let qr = QrCode::encode_segments_advanced(
        &segs, ecl_from_u8(ecl), minv, Version::MAX, mask, false,
    )
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
    let (side, version, mask, modules) =
        js_err(render_encode(text, ecl, version_min, mask_mode))?;
    Ok(QrResult { side, version, mask, modules })
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
        assert!(modules.iter().any(|&b| b == 1));
        assert!(modules.iter().any(|&b| b == 0));
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

    // Migrated from Python TestPreview.
    #[test]
    fn ascii_is_nonempty_ascii_with_dark_modules() {
        let out = render_ascii("hello", 0, 1).unwrap();
        assert!(out.len() > 100);
        assert!(out.contains("##"));
        assert!(out.is_ascii());
        // border=1: 2 top + 21*2 matrix + 2 bottom = 46 lines.
        assert!(out.split('\n').count() >= 40);
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
