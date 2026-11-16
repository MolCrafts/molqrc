//! Invalid arguments: molqrc must panic exactly where Project Nayuki's
//! qrcodegen panics, and must not panic where it does not.
//!
//! Kept apart from `conformance.rs` because silencing the panic hook here
//! would also hide assertion messages of tests running alongside.

use std::panic::{catch_unwind, set_hook, UnwindSafe};

use molqrc as ours;
use qrcodegen as theirs;

fn panics(f: impl FnOnce() + UnwindSafe) -> bool {
    catch_unwind(f).is_err()
}

/// Each pair is the same call in both libraries.
fn assert_same_panic(
    ctx: &str,
    ours: impl FnOnce() + UnwindSafe,
    theirs: impl FnOnce() + UnwindSafe,
    expected: bool,
) {
    assert_eq!(panics(theirs), expected, "reference: {ctx}");
    assert_eq!(panics(ours), expected, "molqrc: {ctx}");
}

#[test]
fn invalid_arguments_panic_like_the_reference() {
    set_hook(Box::new(|_| {}));

    for v in [0u8, 1, 40, 41, 255] {
        let bad = !(1..=40).contains(&v);
        assert_same_panic(
            &format!("Version::new({v})"),
            move || {
                let _ = ours::Version::new(v);
            },
            move || {
                let _ = theirs::Version::new(v);
            },
            bad,
        );
    }
    for m in [0u8, 7, 8, 255] {
        assert_same_panic(
            &format!("Mask::new({m})"),
            move || {
                let _ = ours::Mask::new(m);
            },
            move || {
                let _ = theirs::Mask::new(m);
            },
            m > 7,
        );
    }

    for (text, bad) in [
        ("", false),
        ("0123", false),
        ("12a", true),
        ("١٢", true),
        (" ", true),
    ] {
        assert_same_panic(
            &format!("make_numeric({text:?})"),
            move || drop(ours::QrSegment::make_numeric(text)),
            move || drop(theirs::QrSegment::make_numeric(text)),
            bad,
        );
    }
    for (text, bad) in [
        ("", false),
        ("AB:/", false),
        ("abc", true),
        ("É", true),
        ("A,", true),
    ] {
        assert_same_panic(
            &format!("make_alphanumeric({text:?})"),
            move || drop(ours::QrSegment::make_alphanumeric(text)),
            move || drop(theirs::QrSegment::make_alphanumeric(text)),
            bad,
        );
    }
    for (value, bad) in [(999_999u32, false), (1_000_000, true), (u32::MAX, true)] {
        assert_same_panic(
            &format!("make_eci({value})"),
            move || drop(ours::QrSegment::make_eci(value)),
            move || drop(theirs::QrSegment::make_eci(value)),
            bad,
        );
    }

    for (val, len, bad) in [
        (0u32, 0u8, false),
        (1, 0, true),
        (2, 1, true),
        (u32::MAX >> 1, 31, false),
        (u32::MAX, 31, true),
        (0, 32, true),
        (0, 255, true),
    ] {
        assert_same_panic(
            &format!("append_bits({val}, {len})"),
            move || ours::BitBuffer(Vec::new()).append_bits(val, len),
            move || theirs::BitBuffer(Vec::new()).append_bits(val, len),
            bad,
        );
    }

    // minversion above maxversion.
    assert_same_panic(
        "encode_segments_advanced(min 5, max 4)",
        || {
            let _ = ours::QrCode::encode_segments_advanced(
                &[],
                ours::QrCodeEcc::Low,
                ours::Version::new(5),
                ours::Version::new(4),
                None,
                true,
            );
        },
        || {
            let _ = theirs::QrCode::encode_segments_advanced(
                &[],
                theirs::QrCodeEcc::Low,
                theirs::Version::new(5),
                theirs::Version::new(4),
                None,
                true,
            );
        },
        true,
    );

    // Version 1 at level L holds exactly 19 data codewords.
    for (len, bad) in [(18, true), (19, false), (20, true)] {
        assert_same_panic(
            &format!("encode_codewords with {len} codewords"),
            move || {
                drop(ours::QrCode::encode_codewords(
                    ours::Version::new(1),
                    ours::QrCodeEcc::Low,
                    &vec![0; len],
                    None,
                ))
            },
            move || {
                drop(theirs::QrCode::encode_codewords(
                    theirs::Version::new(1),
                    theirs::QrCodeEcc::Low,
                    &vec![0; len],
                    None,
                ))
            },
            bad,
        );
    }
}
