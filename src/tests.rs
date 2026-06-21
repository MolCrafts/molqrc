// Test suite migrated from the original molqrc C and Python test suites
// (`tests/test_molqrc.c` and `tests/test_python_bindings.py`) onto the Rust API.
//
// Internal-primitive tests (GF multiply, Reed-Solomon, capacity, alignment,
// character-count bits, …) call the private functions of the ported
// Project Nayuki algorithm directly — possible because this module is a child
// of the crate root. Behavioural tests use the public API.

use super::*;

/// Packs a bit sequence MSB-first into bytes (last byte zero-padded on the low
/// bits), matching the byte layout asserted by the original C tests.
fn bits_to_bytes(bits: &[bool]) -> Vec<u8> {
	let mut bytes = vec![0u8; bits.len().div_ceil(8)];
	for (i, &bit) in bits.iter().enumerate() {
		if bit {
			bytes[i / 8] |= 0x80u8 >> (i % 8);
		}
	}
	bytes
}

const ECLS: [QrCodeEcc; 4] =
	[QrCodeEcc::Low, QrCodeEcc::Medium, QrCodeEcc::Quartile, QrCodeEcc::High];

/// Forces a QR Code at an exact version (for version-only internal checks).
fn qr_at_version(ver: u8) -> QrCode {
	let v = Version::new(ver);
	QrCode::encode_segments_advanced(&[], QrCodeEcc::Low, v, v, None, false).unwrap()
}

// ---- GF(256) multiply (test_gf_mul) -----------------------------------------
#[test]
fn gf_mul() {
	let cases: &[(u8, u8, u8)] = &[
		(0x00, 0x00, 0x00), (0x01, 0x01, 0x01), (0x02, 0x02, 0x04),
		(0x00, 0x6E, 0x00), (0xB2, 0xDD, 0xE6), (0x41, 0x11, 0x25),
		(0xB0, 0x1F, 0x11), (0x05, 0x75, 0xBC), (0x52, 0xB5, 0xAE),
		(0xA8, 0x20, 0xA4), (0x0E, 0x44, 0x9F), (0xD4, 0x13, 0xA0),
		(0x31, 0x10, 0x37), (0x6C, 0x58, 0xCB), (0xB6, 0x75, 0x3E),
		(0xFF, 0xFF, 0xE2),
	];
	for &(x, y, want) in cases {
		assert_eq!(QrCode::reed_solomon_multiply(x, y), want, "{x:#x} * {y:#x}");
	}
}

// ---- RS divisor polynomial (test_rs_compute_divisor) ------------------------
#[test]
fn rs_compute_divisor() {
	assert_eq!(QrCode::reed_solomon_compute_divisor(1), vec![0x01]);
	assert_eq!(QrCode::reed_solomon_compute_divisor(2), vec![0x03, 0x02]);
	assert_eq!(
		QrCode::reed_solomon_compute_divisor(5),
		vec![0x1F, 0xC6, 0x3F, 0x93, 0x74]
	);
	let g = QrCode::reed_solomon_compute_divisor(30);
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
	let g3 = QrCode::reed_solomon_compute_divisor(3);
	assert_eq!(QrCode::reed_solomon_compute_remainder(&[], &g3), vec![0, 0, 0]);

	// 2-byte data {0,1}, 5-byte divisor → remainder == divisor.
	let g5 = QrCode::reed_solomon_compute_divisor(5);
	assert_eq!(
		QrCode::reed_solomon_compute_remainder(&[0x00, 0x01], &g5),
		g5
	);

	// 5-byte data, 5-byte divisor.
	let rem = QrCode::reed_solomon_compute_remainder(
		&[0x03, 0x3A, 0x60, 0x12, 0xC7], &g5);
	assert_eq!(rem, vec![0xCB, 0x36, 0x16, 0xFA, 0x9D]);

	// 43-byte data, 30-byte divisor.
	let data: [u8; 43] = [
		0x38, 0x71, 0xDB, 0xF9, 0xD7, 0x28, 0xF6, 0x8E, 0xFE, 0x5E, 0xE6,
		0x7D, 0x7D, 0xB2, 0xA5, 0x58, 0xBC, 0x28, 0x23, 0x53, 0x14, 0xD5,
		0x61, 0xC0, 0x20, 0x6C, 0xDE, 0xDE, 0xFC, 0x79, 0xB0, 0x8B, 0x78,
		0x6B, 0x49, 0xD0, 0x1A, 0xAD, 0xF3, 0xEF, 0x52, 0x7D, 0x9A,
	];
	let g30 = QrCode::reed_solomon_compute_divisor(30);
	let rem = QrCode::reed_solomon_compute_remainder(&data, &g30);
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

// ---- Bit buffer append (test_bitbuf_append) ---------------------------------
#[test]
fn bitbuf_append() {
	let mut bb = BitBuffer(Vec::new());
	bb.append_bits(0, 0);
	assert_eq!(bb.0.len(), 0);
	bb.append_bits(1, 1);
	assert_eq!(bb.0.len(), 1);
	assert_eq!(bits_to_bytes(&bb.0)[0], 0x80);
	bb.append_bits(0, 1);
	assert_eq!(bb.0.len(), 2);
	assert_eq!(bits_to_bytes(&bb.0)[0], 0x80);
	bb.append_bits(5, 3);
	assert_eq!(bb.0.len(), 5);
	assert_eq!(bits_to_bytes(&bb.0)[0], 0xA8);
	bb.append_bits(6, 3);
	assert_eq!(bb.0.len(), 8);
	assert_eq!(bits_to_bytes(&bb.0)[0], 0xAE);

	let mut bb = BitBuffer(Vec::new());
	bb.append_bits(16942, 16);
	assert_eq!(bb.0.len(), 16);
	let bytes = bits_to_bytes(&bb.0);
	assert_eq!((bytes[0], bytes[1]), (0x42, 0x2E));
	bb.append_bits(10, 7);
	assert_eq!(bb.0.len(), 23);
	assert_eq!(bits_to_bytes(&bb.0)[2], 0x14);
	bb.append_bits(15, 4);
	assert_eq!(bb.0.len(), 27);
	let bytes = bits_to_bytes(&bb.0);
	assert_eq!((bytes[2], bytes[3]), (0x15, 0xE0));
	bb.append_bits(26664, 15);
	assert_eq!(bb.0.len(), 42);
	assert_eq!(bits_to_bytes(&bb.0), vec![0x42, 0x2E, 0x15, 0xFA, 0x0A, 0x00]);
}

// ---- Data-codeword capacity (test_capacity) ---------------------------------
#[test]
fn capacity() {
	let cases: &[(u8, usize, usize)] = &[
		(3, 1, 44), (3, 2, 34), (3, 3, 26), (6, 0, 136), (7, 0, 156),
		(9, 0, 232), (9, 1, 182), (12, 3, 158), (15, 0, 523), (16, 2, 325),
		(19, 3, 341), (21, 0, 932), (22, 0, 1006), (22, 1, 782), (22, 3, 442),
		(24, 0, 1174), (24, 3, 514), (28, 0, 1531), (30, 3, 745), (32, 3, 845),
		(33, 0, 2071), (33, 3, 901), (35, 0, 2306), (35, 1, 1812), (35, 2, 1286),
		(36, 3, 1054), (37, 3, 1096), (39, 1, 2216), (40, 1, 2334),
	];
	for &(v, ecl, want) in cases {
		assert_eq!(
			QrCode::get_num_data_codewords(Version::new(v), ECLS[ecl]),
			want, "v{v} ecl{ecl}"
		);
	}
}

// ---- Raw data modules (test_raw_data_modules) -------------------------------
#[test]
fn raw_data_modules() {
	let cases: &[(u8, usize)] = &[
		(1, 208), (2, 359), (3, 567), (6, 1383), (7, 1568),
		(12, 3728), (15, 5243), (18, 7211), (22, 10068), (26, 13652),
		(32, 19723), (37, 25568), (40, 29648),
	];
	for &(v, want) in cases {
		assert_eq!(QrCode::get_num_raw_data_modules(Version::new(v)), want, "v{v}");
	}
}

// ---- Alignment-pattern positions (test_alignment_positions) -----------------
#[test]
fn alignment_positions() {
	let cases: &[(u8, &[i32])] = &[
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
		let qr = qr_at_version(v);
		assert_eq!(qr.get_alignment_pattern_positions(), want.to_vec(), "v{v}");
	}
}

// ---- Version ⇄ side length (test_version_to_side) ---------------------------
#[test]
fn version_to_side() {
	for v in 1..=40u8 {
		let qr = qr_at_version(v);
		assert_eq!(qr.size(), 17 + i32::from(v) * 4);
		assert_eq!(qr.size(), 21 + 4 * (i32::from(v) - 1));
	}
}

// ---- is_numeric / is_alphanumeric (test_is_numeric / test_is_alphanumeric) --
#[test]
fn is_numeric() {
	for s in ["", "0", "79068"] {
		assert!(QrSegment::is_numeric(s), "{s:?}");
	}
	for s in ["A", "a", " ", ".", "*", ",", "|", "@", "XYZ", "XYZ!",
		"+123 ABC$", "\x01", "\x7F", "\u{80}", "\u{C0}", "\u{FF}"] {
		assert!(!QrSegment::is_numeric(s), "{s:?}");
	}
}

#[test]
fn is_alphanumeric() {
	for s in ["", "0", "A", " ", ".", "*", "XYZ", "79068", "+123 ABC$"] {
		assert!(QrSegment::is_alphanumeric(s), "{s:?}");
	}
	for s in ["a", ",", "|", "@", "XYZ!", "\x01", "\x7F",
		"\u{80}", "\u{C0}", "\u{FF}"] {
		assert!(!QrSegment::is_alphanumeric(s), "{s:?}");
	}
}

// ---- Segment data-bit lengths (test_calc_segment_bit_length, data portion) --
#[test]
fn segment_data_bit_lengths() {
	// Numeric: 1→4, 2→7, 3→10, 4→14, 5→17, 6→20.
	for (n, bits) in [(1, 4), (2, 7), (3, 10), (4, 14), (5, 17), (6, 20)] {
		assert_eq!(QrSegment::make_numeric(&"1".repeat(n)).data().len(), bits);
	}
	// Alphanumeric: 1→6, 2→11, 3→17, 4→22, 5→28, 6→33.
	for (n, bits) in [(1, 6), (2, 11), (3, 17), (4, 22), (5, 28), (6, 33)] {
		assert_eq!(QrSegment::make_alphanumeric(&"A".repeat(n)).data().len(), bits);
	}
	// Byte: n → 8n.
	for n in [0, 1, 2, 3, 100] {
		assert_eq!(QrSegment::make_bytes(&vec![0u8; n]).data().len(), 8 * n);
	}
}

// ---- get_total_bits: per-version length-field limit -------------------------
#[test]
fn total_bits_length_field_limit() {
	// At version 1 the numeric char-count field is 10 bits → limit 1024 chars.
	let seg = QrSegment::make_numeric(&"1".repeat(1024));
	assert_eq!(QrSegment::get_total_bits(std::slice::from_ref(&seg), Version::new(1)), None);
	// The same segment fits at version 40 (14-bit field, limit 16384).
	assert!(QrSegment::get_total_bits(std::slice::from_ref(&seg), Version::new(40)).is_some());
	// 4 (mode) + 10 (count) + data bits for a small numeric segment at v1.
	let small = QrSegment::make_numeric("314159");
	assert_eq!(
		QrSegment::get_total_bits(std::slice::from_ref(&small), Version::new(1)),
		Some(4 + 10 + small.data().len())
	);
}

// ---- Char-count-indicator bits (test_cci_bits) ------------------------------
#[test]
fn cci_bits() {
	use QrSegmentMode::*;
	let v = Version::new;
	assert_eq!(Numeric.num_char_count_bits(v(1)), 10);
	assert_eq!(Numeric.num_char_count_bits(v(9)), 10);
	assert_eq!(Numeric.num_char_count_bits(v(10)), 12);
	assert_eq!(Numeric.num_char_count_bits(v(26)), 12);
	assert_eq!(Numeric.num_char_count_bits(v(27)), 14);
	assert_eq!(Numeric.num_char_count_bits(v(40)), 14);
	assert_eq!(Alphanumeric.num_char_count_bits(v(1)), 9);
	assert_eq!(Alphanumeric.num_char_count_bits(v(10)), 11);
	assert_eq!(Alphanumeric.num_char_count_bits(v(27)), 13);
	assert_eq!(Byte.num_char_count_bits(v(1)), 8);
	assert_eq!(Byte.num_char_count_bits(v(9)), 8);
	assert_eq!(Byte.num_char_count_bits(v(10)), 16);
	assert_eq!(Byte.num_char_count_bits(v(40)), 16);
	assert_eq!(Eci.num_char_count_bits(v(1)), 0);
}

// ---- Segment constructors (test_make_bytes/numeric/alphanumeric/eci) --------
#[test]
fn make_bytes() {
	let seg = QrSegment::make_bytes(&[]);
	assert_eq!(seg.mode(), QrSegmentMode::Byte);
	assert_eq!(seg.num_chars(), 0);
	assert_eq!(seg.data().len(), 0);

	let seg = QrSegment::make_bytes(&[0x00]);
	assert_eq!(seg.num_chars(), 1);
	assert_eq!(seg.data().len(), 8);
	assert_eq!(bits_to_bytes(seg.data())[0], 0x00);

	let seg = QrSegment::make_bytes(&[0xEF, 0xBB, 0xBF]);
	assert_eq!(seg.num_chars(), 3);
	assert_eq!(seg.data().len(), 24);
	assert_eq!(bits_to_bytes(seg.data()), vec![0xEF, 0xBB, 0xBF]);
}

#[test]
fn make_numeric() {
	let seg = QrSegment::make_numeric("");
	assert_eq!(seg.mode(), QrSegmentMode::Numeric);
	assert_eq!(seg.num_chars(), 0);
	assert_eq!(seg.data().len(), 0);

	let seg = QrSegment::make_numeric("9");
	assert_eq!(seg.num_chars(), 1);
	assert_eq!(seg.data().len(), 4);
	assert_eq!(bits_to_bytes(seg.data())[0], 0x90);

	let seg = QrSegment::make_numeric("81");
	assert_eq!(seg.num_chars(), 2);
	assert_eq!(seg.data().len(), 7);
	assert_eq!(bits_to_bytes(seg.data())[0], 0xA2);

	let seg = QrSegment::make_numeric("673");
	assert_eq!(seg.num_chars(), 3);
	assert_eq!(seg.data().len(), 10);
	assert_eq!(bits_to_bytes(seg.data())[..2], [0xA8, 0x40]);

	let seg = QrSegment::make_numeric("3141592653");
	assert_eq!(seg.num_chars(), 10);
	assert_eq!(seg.data().len(), 34);
	assert_eq!(bits_to_bytes(seg.data())[..5], [0x4E, 0x89, 0xF4, 0x24, 0xC0]);
}

#[test]
fn make_alphanumeric() {
	let seg = QrSegment::make_alphanumeric("");
	assert_eq!(seg.mode(), QrSegmentMode::Alphanumeric);
	assert_eq!(seg.num_chars(), 0);
	assert_eq!(seg.data().len(), 0);

	let seg = QrSegment::make_alphanumeric("A");
	assert_eq!(seg.num_chars(), 1);
	assert_eq!(seg.data().len(), 6);
	assert_eq!(bits_to_bytes(seg.data())[0], 0x28);

	let seg = QrSegment::make_alphanumeric("%:");
	assert_eq!(seg.num_chars(), 2);
	assert_eq!(seg.data().len(), 11);
	assert_eq!(bits_to_bytes(seg.data())[..2], [0xDB, 0x40]);

	let seg = QrSegment::make_alphanumeric("Q R");
	assert_eq!(seg.num_chars(), 3);
	assert_eq!(seg.data().len(), 17);
	assert_eq!(bits_to_bytes(seg.data())[..3], [0x96, 0xCD, 0x80]);
}

#[test]
fn make_eci() {
	let seg = QrSegment::make_eci(127);
	assert_eq!(seg.mode(), QrSegmentMode::Eci);
	assert_eq!(seg.num_chars(), 0);
	assert_eq!(bits_to_bytes(seg.data())[0], 0x7F);

	let seg = QrSegment::make_eci(10345);
	assert_eq!(bits_to_bytes(seg.data())[..2], [0xA8, 0x69]);

	let seg = QrSegment::make_eci(999999);
	assert_eq!(bits_to_bytes(seg.data())[..3], [0xCF, 0x42, 0x3F]);
}

// ---- encode_text (test_encode_text + Python TestQRCode) ---------------------
#[test]
fn encode_text_basics() {
	// "hello" → version 1, side 21 (matches Python QRCode("hello")).
	let qr = QrCode::encode_text("hello", QrCodeEcc::Low).unwrap();
	assert_eq!(qr.version().value(), 1);
	assert_eq!(qr.size(), 21);

	// Empty string is accepted → version 1, side 21.
	let qr = QrCode::encode_text("", QrCodeEcc::Low).unwrap();
	assert_eq!(qr.version().value(), 1);
	assert_eq!(qr.size(), 21);

	// Long text upgrades the version.
	let qr = QrCode::encode_text(&"A".repeat(200), QrCodeEcc::Low).unwrap();
	assert!(qr.version().value() > 1);
	assert!(qr.size() > 21);

	// Numeric / alphanumeric auto-detection both succeed.
	assert!(QrCode::encode_text("1234567890", QrCodeEcc::Low).is_ok());
	assert!(QrCode::encode_text("HELLO WORLD", QrCodeEcc::Medium).is_ok());

	// All four ECLs succeed.
	for ecl in ECLS {
		assert!(QrCode::encode_text("hello world", ecl).is_ok());
	}
}

#[test]
fn encode_version_range_and_mask() {
	// Force version ≥ 10.
	let qr = QrCode::encode_segments_advanced(
		&QrSegment::make_segments("hello"), QrCodeEcc::Medium,
		Version::new(10), Version::MAX, None, true).unwrap();
	assert!(qr.version().value() >= 10);
	assert!(qr.size() >= 21 + 4 * 9);

	// Manual mask is honoured.
	for m in 0..8u8 {
		let qr = QrCode::encode_segments_advanced(
			&QrSegment::make_segments("hi"), QrCodeEcc::Medium,
			Version::MIN, Version::MAX, Some(Mask::new(m)), false).unwrap();
		assert_eq!(qr.mask().value(), m);
	}
}

#[test]
fn encode_too_long_errors() {
	// 3000 bytes exceeds every version's capacity → error (Python test_too_long_raises).
	assert!(QrCode::encode_binary(&[0u8; 3000], QrCodeEcc::High).is_err());
}

// ---- encode_segments (test_encode_segments) ---------------------------------
#[test]
fn encode_segments_variants() {
	let one = |seg: QrSegment| {
		QrCode::encode_segments(&[seg], QrCodeEcc::Medium)
	};
	assert!(one(QrSegment::make_bytes(b"hello")).is_ok());
	assert!(one(QrSegment::make_numeric("12345678901234567890")).is_ok());
	assert!(one(QrSegment::make_alphanumeric("HELLO WORLD")).is_ok());

	// Mixed: ECI + NUMERIC + ALPHANUMERIC + BYTE.
	let segs = vec![
		QrSegment::make_eci(127),
		QrSegment::make_numeric("1234567"),
		QrSegment::make_alphanumeric("A"),
		QrSegment::make_bytes(b"test"),
	];
	assert!(QrCode::encode_segments(&segs, QrCodeEcc::Quartile).is_ok());

	// 200 bytes cannot fit version 1 → error.
	let big = QrSegment::make_bytes(&vec![b'A'; 200]);
	assert!(QrCode::encode_segments_advanced(
		std::slice::from_ref(&big), QrCodeEcc::Low,
		Version::MIN, Version::new(1), None, false).is_err());
}

// ---- Matrix integrity (test_matrix_integrity + module counting / draw) ------
#[test]
fn matrix_integrity_all_versions() {
	for v in 1..=40u8 {
		let qr = QrCode::encode_segments_advanced(
			&QrSegment::make_segments("test"), QrCodeEcc::Low,
			Version::new(v), Version::new(v), None, false).unwrap();
		assert_eq!(qr.size(), 17 + i32::from(v) * 4);
		let mut light = 0u32;
		let mut dark = 0u32;
		for y in 0..qr.size() {
			for x in 0..qr.size() {
				if qr.get_module(x, y) { dark += 1 } else { light += 1 }
			}
		}
		assert!(light > 0 && dark > 0, "v{v}: light={light} dark={dark}");
	}
}
