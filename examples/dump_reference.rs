//! Dumps native encoder output as JSON so that the compiled WebAssembly build
//! can be diffed against it module-for-module.
//!
//! Two references are emitted, because the two wasm entry points deliberately
//! differ in their ECC handling:
//!
//! - `boost` — `QrCode::encode_text`, which may raise the ECC level when that is
//!   free. The wasm counterpart is `qr_modules` / `qr_svg`.
//! - `noboost` — `encode_segments_advanced(.., boostecl = false)`, which honours
//!   the requested level exactly. The counterpart is `qr_encode`.
//!
//! Comparing the wrong pair produces spurious mask differences, since a boosted
//! ECC level changes both the format bits and the winning mask.
//!
//! Used by scripts/verify-wasm-parity.sh.

use molqrc::{QrCode, QrCodeEcc, QrSegment, Version};

const ECLS: [QrCodeEcc; 4] = [
    QrCodeEcc::Low,
    QrCodeEcc::Medium,
    QrCodeEcc::Quartile,
    QrCodeEcc::High,
];

fn main() {
    let long = "A".repeat(500);
    let cases: [&str; 8] = [
        "hello",
        "",
        "HELLO WORLD",
        "3141592653589793238462643383",
        "https://github.com/MolCrafts/molqrc",
        "日本語テキスト",
        "\u{1F600} emoji + mixed 123 ABC",
        &long,
    ];

    let mut entries: Vec<String> = Vec::new();
    for text in cases {
        for (e, &ecl) in ECLS.iter().enumerate() {
            let boost = QrCode::encode_text(text, ecl).unwrap();
            let noboost = QrCode::encode_segments_advanced(
                &QrSegment::make_segments(text),
                ecl,
                Version::MIN,
                Version::MAX,
                None,
                false,
            )
            .unwrap();
            entries.push(format!(
                "{{\"text\":{},\"ecl\":{},\"boost\":{},\"noboost\":{}}}",
                json_str(text),
                e,
                describe(&boost),
                describe(&noboost)
            ));
        }
    }
    println!("[\n{}\n]", entries.join(",\n"));
}

fn describe(qr: &QrCode) -> String {
    let mut bits = String::with_capacity((qr.size() * qr.size()) as usize);
    for y in 0..qr.size() {
        for x in 0..qr.size() {
            bits.push(if qr.get_module(x, y) { '1' } else { '0' });
        }
    }
    format!(
        "{{\"side\":{},\"version\":{},\"mask\":{},\"bits\":\"{}\"}}",
        qr.size(),
        qr.version().value(),
        qr.mask().value(),
        bits
    )
}

/// Minimal JSON string escaping — this crate has no serde dependency.
fn json_str(s: &str) -> String {
    let mut r = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => r.push_str("\\\""),
            '\\' => r.push_str("\\\\"),
            c if (c as u32) < 0x20 => r.push_str(&format!("\\u{:04x}", c as u32)),
            c => r.push(c),
        }
    }
    r.push('"');
    r
}
