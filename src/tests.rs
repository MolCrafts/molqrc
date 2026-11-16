// Known-answer tests for the internals that no public call exposes on their
// own, with values from Project Nayuki's C test suite (qrcodegen-test.c).
// Whole-symbol behaviour is checked against the reference library in
// tests/conformance.rs.

use crate::capacity::{alignment_pattern_positions, num_data_codewords, num_raw_data_modules};
use crate::reed_solomon::{compute_divisor, compute_remainder, multiply};
use crate::{QrCodeEcc, Version};

const ECLS: [QrCodeEcc; 4] = [
    QrCodeEcc::Low,
    QrCodeEcc::Medium,
    QrCodeEcc::Quartile,
    QrCodeEcc::High,
];

// ---- GF(256) multiply (test_gf_mul) -----------------------------------------
#[test]
fn gf_mul() {
    let cases: &[(u8, u8, u8)] = &[
        (0x00, 0x00, 0x00),
        (0x01, 0x01, 0x01),
        (0x02, 0x02, 0x04),
        (0x00, 0x6E, 0x00),
        (0xB2, 0xDD, 0xE6),
        (0x41, 0x11, 0x25),
        (0xB0, 0x1F, 0x11),
        (0x05, 0x75, 0xBC),
        (0x52, 0xB5, 0xAE),
        (0xA8, 0x20, 0xA4),
        (0x0E, 0x44, 0x9F),
        (0xD4, 0x13, 0xA0),
        (0x31, 0x10, 0x37),
        (0x6C, 0x58, 0xCB),
        (0xB6, 0x75, 0x3E),
        (0xFF, 0xFF, 0xE2),
    ];
    for &(x, y, want) in cases {
        assert_eq!(multiply(x, y), want, "{x:#x} * {y:#x}");
    }
}

// ---- RS divisor polynomial (test_rs_compute_divisor) ------------------------
#[test]
fn rs_compute_divisor() {
    assert_eq!(compute_divisor(1), vec![0x01]);
    assert_eq!(compute_divisor(2), vec![0x03, 0x02]);
    assert_eq!(compute_divisor(5), vec![0x1F, 0xC6, 0x3F, 0x93, 0x74]);
    let g = compute_divisor(30);
    assert_eq!(g.len(), 30);
    assert_eq!(g[0], 0xD4);
    assert_eq!(g[1], 0xF6);
    assert_eq!(g[5], 0xC0);
    assert_eq!(g[12], 0x16);
    assert_eq!(g[13], 0xD9);
    assert_eq!(g[20], 0x12);
    assert_eq!(g[27], 0x6A);
    assert_eq!(g[29], 0x96);
}

// ---- RS remainder (test_rs_compute_remainder) -------------------------------
#[test]
fn rs_compute_remainder() {
    // 3-byte divisor, zero-length data → all zeros.
    let g3 = compute_divisor(3);
    assert_eq!(compute_remainder(&[], &g3), vec![0, 0, 0]);

    // 2-byte data {0,1}, 5-byte divisor → remainder == divisor.
    let g5 = compute_divisor(5);
    assert_eq!(compute_remainder(&[0x00, 0x01], &g5), g5);

    // 5-byte data, 5-byte divisor.
    let rem = compute_remainder(&[0x03, 0x3A, 0x60, 0x12, 0xC7], &g5);
    assert_eq!(rem, vec![0xCB, 0x36, 0x16, 0xFA, 0x9D]);

    // 43-byte data, 30-byte divisor.
    let data: [u8; 43] = [
        0x38, 0x71, 0xDB, 0xF9, 0xD7, 0x28, 0xF6, 0x8E, 0xFE, 0x5E, 0xE6, 0x7D, 0x7D, 0xB2, 0xA5,
        0x58, 0xBC, 0x28, 0x23, 0x53, 0x14, 0xD5, 0x61, 0xC0, 0x20, 0x6C, 0xDE, 0xDE, 0xFC, 0x79,
        0xB0, 0x8B, 0x78, 0x6B, 0x49, 0xD0, 0x1A, 0xAD, 0xF3, 0xEF, 0x52, 0x7D, 0x9A,
    ];
    let g30 = compute_divisor(30);
    let rem = compute_remainder(&data, &g30);
    assert_eq!(rem.len(), 30);
    assert_eq!(rem[0], 0xCE);
    assert_eq!(rem[1], 0xF0);
    assert_eq!(rem[2], 0x31);
    assert_eq!(rem[3], 0xDE);
    assert_eq!(rem[8], 0xE1);
    assert_eq!(rem[12], 0xCA);
    assert_eq!(rem[17], 0xE3);
    assert_eq!(rem[19], 0x85);
    assert_eq!(rem[20], 0x50);
    assert_eq!(rem[24], 0xBE);
    assert_eq!(rem[29], 0xB3);
}

// ---- Data-codeword capacity (test_capacity) ---------------------------------
#[test]
fn capacity() {
    let cases: &[(u8, usize, usize)] = &[
        (3, 1, 44),
        (3, 2, 34),
        (3, 3, 26),
        (6, 0, 136),
        (7, 0, 156),
        (9, 0, 232),
        (9, 1, 182),
        (12, 3, 158),
        (15, 0, 523),
        (16, 2, 325),
        (19, 3, 341),
        (21, 0, 932),
        (22, 0, 1006),
        (22, 1, 782),
        (22, 3, 442),
        (24, 0, 1174),
        (24, 3, 514),
        (28, 0, 1531),
        (30, 3, 745),
        (32, 3, 845),
        (33, 0, 2071),
        (33, 3, 901),
        (35, 0, 2306),
        (35, 1, 1812),
        (35, 2, 1286),
        (36, 3, 1054),
        (37, 3, 1096),
        (39, 1, 2216),
        (40, 1, 2334),
    ];
    for &(v, ecl, want) in cases {
        assert_eq!(
            num_data_codewords(Version::new(v), ECLS[ecl]),
            want,
            "v{v} ecl{ecl}"
        );
    }
}

// ---- Raw data modules (test_raw_data_modules) -------------------------------
#[test]
fn raw_data_modules() {
    let cases: &[(u8, usize)] = &[
        (1, 208),
        (2, 359),
        (3, 567),
        (6, 1383),
        (7, 1568),
        (12, 3728),
        (15, 5243),
        (18, 7211),
        (22, 10068),
        (26, 13652),
        (32, 19723),
        (37, 25568),
        (40, 29648),
    ];
    for &(v, want) in cases {
        assert_eq!(num_raw_data_modules(Version::new(v)), want, "v{v}");
    }
}

// ---- Alignment-pattern positions (test_alignment_positions) -----------------
#[test]
fn alignment_positions() {
    let cases: &[(u8, &[usize])] = &[
        (1, &[]),
        (2, &[6, 18]),
        (3, &[6, 22]),
        (6, &[6, 34]),
        (7, &[6, 22, 38]),
        (8, &[6, 24, 42]),
        (16, &[6, 26, 50, 74]),
        (25, &[6, 32, 58, 84, 110]),
        (32, &[6, 34, 60, 86, 112, 138]),
        (33, &[6, 30, 58, 86, 114, 142]),
        (39, &[6, 26, 54, 82, 110, 138, 166]),
        (40, &[6, 30, 58, 86, 114, 142, 170]),
    ];
    for &(v, want) in cases {
        assert_eq!(alignment_pattern_positions(Version::new(v)), want, "v{v}");
    }
}
