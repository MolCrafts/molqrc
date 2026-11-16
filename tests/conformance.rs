//! Conformance against Project Nayuki's qrcodegen (pinned in Cargo.toml).
//!
//! Every case runs through both libraries with the same arguments, and the
//! results must agree completely:
//! - **Success:** version, size, level, mask and every module, plus the light
//!   modules just outside the symbol.
//! - **Failure:** the same error variant, the same numbers and the same message.
//!
//! The cases fall into two groups:
//! - **Boundaries,** enumerated exhaustively: every capacity limit, every
//!   character count field limit, every version, level and mask, and every ECI
//!   width.
//! - **A seeded random batch** with the distribution of Nayuki's own
//!   cross-language batch tester (`python/qrcodegen-batch-test.py`, removed
//!   upstream in 2261248).

use molqrc as ours;
use qrcodegen as theirs;

// ---- Paired construction ---------------------------------------------------

const LEVELS: [(ours::QrCodeEcc, theirs::QrCodeEcc); 4] = [
    (ours::QrCodeEcc::Low, theirs::QrCodeEcc::Low),
    (ours::QrCodeEcc::Medium, theirs::QrCodeEcc::Medium),
    (ours::QrCodeEcc::Quartile, theirs::QrCodeEcc::Quartile),
    (ours::QrCodeEcc::High, theirs::QrCodeEcc::High),
];

const NUMERIC: &[u8] = b"0123456789";
const ALPHANUMERIC: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ $%*+-./:";

/// One segment, buildable in both libraries.
#[derive(Clone, Debug)]
enum Seg {
    Numeric(String),
    Alphanumeric(String),
    Bytes(Vec<u8>),
    Eci(u32),
    /// `QrSegment::new` with mode index (0 numeric, 1 alphanumeric, 2 byte,
    /// 3 kanji, 4 ECI), character count and raw bits.
    Raw(usize, usize, Vec<bool>),
}

impl Seg {
    fn ours(&self) -> ours::QrSegment {
        use ours::QrSegmentMode as M;
        match self {
            Seg::Numeric(s) => ours::QrSegment::make_numeric(s),
            Seg::Alphanumeric(s) => ours::QrSegment::make_alphanumeric(s),
            Seg::Bytes(b) => ours::QrSegment::make_bytes(b),
            Seg::Eci(v) => ours::QrSegment::make_eci(*v),
            Seg::Raw(m, n, bits) => {
                let mode = [M::Numeric, M::Alphanumeric, M::Byte, M::Kanji, M::Eci][*m];
                ours::QrSegment::new(mode, *n, bits.clone())
            }
        }
    }

    fn theirs(&self) -> theirs::QrSegment {
        use theirs::QrSegmentMode as M;
        match self {
            Seg::Numeric(s) => theirs::QrSegment::make_numeric(s),
            Seg::Alphanumeric(s) => theirs::QrSegment::make_alphanumeric(s),
            Seg::Bytes(b) => theirs::QrSegment::make_bytes(b),
            Seg::Eci(v) => theirs::QrSegment::make_eci(*v),
            Seg::Raw(m, n, bits) => {
                let mode = [M::Numeric, M::Alphanumeric, M::Byte, M::Kanji, M::Eci][*m];
                theirs::QrSegment::new(mode, *n, bits.clone())
            }
        }
    }
}

/// Arguments of `encode_segments_advanced`.
#[derive(Clone, Debug)]
struct Case {
    segs: Vec<Seg>,
    ecl: usize,
    min: u8,
    max: u8,
    /// `None` = automatic mask.
    mask: Option<u8>,
    boost: bool,
}

impl Case {
    fn new(segs: Vec<Seg>, ecl: usize, min: u8, max: u8, mask: Option<u8>, boost: bool) -> Self {
        Self {
            segs,
            ecl,
            min,
            max,
            mask,
            boost,
        }
    }

    fn run_ours(&self) -> Result<ours::QrCode, ours::DataTooLong> {
        let segs: Vec<_> = self.segs.iter().map(Seg::ours).collect();
        ours::QrCode::encode_segments_advanced(
            &segs,
            LEVELS[self.ecl].0,
            ours::Version::new(self.min),
            ours::Version::new(self.max),
            self.mask.map(ours::Mask::new),
            self.boost,
        )
    }

    fn run_theirs(&self) -> Result<theirs::QrCode, theirs::DataTooLong> {
        let segs: Vec<_> = self.segs.iter().map(Seg::theirs).collect();
        theirs::QrCode::encode_segments_advanced(
            &segs,
            LEVELS[self.ecl].1,
            theirs::Version::new(self.min),
            theirs::Version::new(self.max),
            self.mask.map(theirs::Mask::new),
            self.boost,
        )
    }

    /// Runs both and asserts identical outcomes; returns whether they encoded.
    fn check(&self) -> bool {
        let reference = self.run_theirs();
        assert_same(&self.summary(), &self.run_ours(), &reference);
        reference.is_ok()
    }

    fn summary(&self) -> String {
        let segs: Vec<String> = self
            .segs
            .iter()
            .map(|s| match s {
                Seg::Numeric(t) => format!("numeric({})", t.len()),
                Seg::Alphanumeric(t) => format!("alphanumeric({})", t.len()),
                Seg::Bytes(b) => format!("bytes({})", b.len()),
                Seg::Eci(v) => format!("eci({v})"),
                Seg::Raw(m, n, bits) => format!("raw(mode {m}, {n} chars, {} bits)", bits.len()),
            })
            .collect();
        format!(
            "segs=[{}] ecl={} versions={}..={} mask={:?} boost={}",
            segs.join(", "),
            self.ecl,
            self.min,
            self.max,
            self.mask,
            self.boost
        )
    }
}

// ---- Comparison ------------------------------------------------------------

fn ecl_index(ecl: ours::QrCodeEcc) -> usize {
    LEVELS.iter().position(|l| l.0 == ecl).unwrap()
}

fn their_ecl_index(ecl: theirs::QrCodeEcc) -> usize {
    LEVELS.iter().position(|l| l.1 == ecl).unwrap()
}

fn assert_same_symbol(ctx: &str, ours: &ours::QrCode, theirs: &theirs::QrCode) {
    assert_eq!(
        ours.version().value(),
        theirs.version().value(),
        "version: {ctx}"
    );
    assert_eq!(ours.size(), theirs.size(), "size: {ctx}");
    assert_eq!(
        ecl_index(ours.error_correction_level()),
        their_ecl_index(theirs.error_correction_level()),
        "level: {ctx}"
    );
    assert_eq!(ours.mask().value(), theirs.mask().value(), "mask: {ctx}");
    let size = theirs.size();
    for y in -1..=size {
        for x in -1..=size {
            assert_eq!(
                ours.get_module(x, y),
                theirs.get_module(x, y),
                "module ({x}, {y}): {ctx}"
            );
        }
    }
    for (x, y) in [(i32::MIN, 0), (0, i32::MIN), (i32::MAX, 0), (0, i32::MAX)] {
        assert!(!ours.get_module(x, y), "far outside ({x}, {y}): {ctx}");
    }
}

fn assert_same_error(ctx: &str, ours: &ours::DataTooLong, theirs: &theirs::DataTooLong) {
    let ours_value = match *ours {
        ours::DataTooLong::SegmentTooLong => None,
        ours::DataTooLong::DataOverCapacity(n, cap) => Some((n, cap)),
    };
    let theirs_value = match *theirs {
        theirs::DataTooLong::SegmentTooLong => None,
        theirs::DataTooLong::DataOverCapacity(n, cap) => Some((n, cap)),
    };
    assert_eq!(ours_value, theirs_value, "error: {ctx}");
    assert_eq!(ours.to_string(), theirs.to_string(), "error message: {ctx}");
}

fn assert_same(
    ctx: &str,
    ours: &Result<ours::QrCode, ours::DataTooLong>,
    theirs: &Result<theirs::QrCode, theirs::DataTooLong>,
) {
    match (ours, theirs) {
        (Ok(o), Ok(t)) => assert_same_symbol(ctx, o, t),
        (Err(o), Err(t)) => assert_same_error(ctx, o, t),
        (Ok(_), Err(t)) => panic!("we encoded, reference failed with {t}: {ctx}"),
        (Err(o), Ok(_)) => panic!("we failed with {o}, reference encoded: {ctx}"),
    }
}

fn assert_same_segment(ctx: &str, ours: &ours::QrSegment, theirs: &theirs::QrSegment) {
    let mode = |m: ours::QrSegmentMode| format!("{m:?}");
    let their_mode = |m: theirs::QrSegmentMode| format!("{m:?}");
    assert_eq!(mode(ours.mode()), their_mode(theirs.mode()), "mode: {ctx}");
    assert_eq!(ours.num_chars(), theirs.num_chars(), "num_chars: {ctx}");
    assert_eq!(ours.data(), &theirs.data()[..], "data: {ctx}");
}

// ---- Seeded generator (SplitMix64) -----------------------------------------

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Uniform in `0..n`.
    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }

    /// Uniform in [0, 1).
    fn unit(&mut self) -> f64 {
        (self.next() >> 11) as f64 / (1u64 << 53) as f64
    }

    fn bytes(&mut self, len: usize, alphabet: &[u8]) -> Vec<u8> {
        (0..len)
            .map(|_| alphabet[self.below(alphabet.len())])
            .collect()
    }

    fn text(&mut self, len: usize, alphabet: &[u8]) -> String {
        String::from_utf8(self.bytes(len, alphabet)).unwrap()
    }

    fn any_bytes(&mut self, len: usize) -> Vec<u8> {
        (0..len).map(|_| self.next() as u8).collect()
    }
}

// ---- Capacity --------------------------------------------------------------

/// A segment of `len` random characters of one mode (0 numeric,
/// 1 alphanumeric, 2 byte).
fn filled(rng: &mut Rng, mode: usize, len: usize) -> Seg {
    match mode {
        0 => Seg::Numeric(rng.text(len, NUMERIC)),
        1 => Seg::Alphanumeric(rng.text(len, ALPHANUMERIC)),
        _ => Seg::Bytes(rng.any_bytes(len)),
    }
}

/// The longest single segment of `mode` that fits exactly `version` at `ecl`,
/// by bisection on the reference implementation.
fn max_len(mode: usize, version: u8, ecl: usize) -> usize {
    let fits = |len: usize| {
        let seg = match mode {
            0 => Seg::Numeric("0".repeat(len)),
            1 => Seg::Alphanumeric("0".repeat(len)),
            _ => Seg::Bytes(vec![0; len]),
        };
        Case::new(vec![seg], ecl, version, version, Some(0), false)
            .run_theirs()
            .is_ok()
    };
    let (mut lo, mut hi) = (0, 7090); // fits(lo) and !fits(hi)
    while hi - lo > 1 {
        let mid = (lo + hi) / 2;
        if fits(mid) {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    lo
}

/// At every version, level and text mode: the longest payload that fits, and
/// one character more. Each must give the same symbol or error at that
/// version alone, and the same choice of version when searching from version 1.
#[test]
fn capacity_boundary_of_every_version_level_and_mode() {
    let mut rng = Rng(1);
    for version in 1..=40 {
        for ecl in 0..4 {
            for mode in 0..3 {
                let len = max_len(mode, version, ecl);
                let fitting = filled(&mut rng, mode, len);
                let over = filled(&mut rng, mode, len + 1);

                let exact = Case::new(vec![fitting.clone()], ecl, version, version, None, false);
                assert!(exact.check(), "{}", exact.summary());
                let boosted = Case::new(vec![fitting], ecl, version, version, Some(3), true);
                assert!(boosted.check(), "{}", boosted.summary());

                let overflow = Case::new(vec![over.clone()], ecl, version, version, Some(5), false);
                assert!(!overflow.check(), "{}", overflow.summary());

                let max = if version < 40 { version + 1 } else { 40 };
                let search = Case::new(vec![over], ecl, 1, max, Some(1), true);
                search.check();
            }
        }
    }
}

/// The largest payloads the standard allows, and one more, through the
/// high-level entry points.
#[test]
fn absolute_capacity_limits_through_the_high_level_api() {
    let mut rng = Rng(2);
    for (alphabet, limit) in [(NUMERIC, 7089), (ALPHANUMERIC, 4296)] {
        for len in [limit, limit + 1] {
            let text = rng.text(len, alphabet);
            let ctx = format!("encode_text len {len}");
            assert_same(
                &ctx,
                &ours::QrCode::encode_text(&text, LEVELS[0].0),
                &theirs::QrCode::encode_text(&text, LEVELS[0].1),
            );
        }
    }
    for len in [2953, 2954] {
        let data = rng.any_bytes(len);
        assert_same(
            &format!("encode_binary len {len}"),
            &ours::QrCode::encode_binary(&data, LEVELS[0].0),
            &theirs::QrCode::encode_binary(&data, LEVELS[0].1),
        );
    }
}

// ---- Character count fields ------------------------------------------------

/// A count of 2^bits - 1 fits the field and 2^bits does not, for every
/// mode, around the field-width steps between versions 9/10 and 26/27.
#[test]
fn character_count_field_limits() {
    let mut rng = Rng(3);
    // (mode, field width at versions 1-9, 10-26, 27-40)
    let widths = [
        (0, [10, 12, 14]),
        (1, [9, 11, 13]),
        (2, [8, 16, 16]),
        (3, [8, 10, 12]),
    ];
    let ranges = [
        (1, 9),
        (9, 9),
        (9, 10),
        (10, 10),
        (10, 26),
        (26, 27),
        (27, 27),
        (1, 40),
    ];
    for (mode, field) in widths {
        for bits in field {
            for count in [(1usize << bits) - 1, 1 << bits] {
                // Raw segments keep the data short, so only the count field can overflow.
                let data: Vec<bool> = (0..16).map(|_| rng.below(2) == 1).collect();
                for (min, max) in ranges {
                    for ecl in [0, 3] {
                        Case::new(
                            vec![Seg::Raw(mode, count, data.clone())],
                            ecl,
                            min,
                            max,
                            Some(2),
                            false,
                        )
                        .check();
                    }
                }
            }
        }
    }
    // An ECI segment's count field is zero bits wide: any count but 0 overflows it.
    for count in [0, 1] {
        Case::new(
            vec![Seg::Raw(4, count, vec![false; 8])],
            0,
            1,
            40,
            Some(0),
            false,
        )
        .check();
    }
}

/// Real numeric, alphanumeric and byte segments whose character count needs
/// the wider field of a higher version.
#[test]
fn segments_that_need_a_wider_count_field() {
    let mut rng = Rng(4);
    for (mode, count) in [(0, 1024), (1, 512), (2, 256), (0, 4096), (1, 2048)] {
        for (min, max) in [(1, 9), (1, 10), (1, 26), (1, 40), (10, 40), (27, 40)] {
            for ecl in 0..4 {
                Case::new(
                    vec![filled(&mut rng, mode, count)],
                    ecl,
                    min,
                    max,
                    None,
                    true,
                )
                .check();
            }
        }
    }
}

// ---- Every version, level and mask -----------------------------------------

/// The data capacity in codewords, read from the reference's overflow error.
fn data_codewords(version: u8, ecl: usize) -> usize {
    // Below version 10 the byte count field is 8 bits, so stay under 256 bytes.
    let len = if version < 10 { 255 } else { 3000 };
    let too_big = Case::new(
        vec![Seg::Bytes(vec![0; len])],
        ecl,
        version,
        version,
        Some(0),
        false,
    );
    match too_big.run_theirs() {
        Err(theirs::DataTooLong::DataOverCapacity(_, cap_bits)) => cap_bits / 8,
        _ => unreachable!("the payload overflows version {version}"),
    }
}

/// `encode_codewords` at every version, level and mask, plus the automatic
/// mask. The payloads include degenerate ones (all 0x00, all 0xFF), which push
/// the run, block and balance penalties to their extremes.
#[test]
fn codewords_at_every_version_level_and_mask() {
    let mut rng = Rng(5);
    for version in 1..=40 {
        for (ecl, &(our_ecl, their_ecl)) in LEVELS.iter().enumerate() {
            let n = data_codewords(version, ecl);
            let payloads = [
                rng.any_bytes(n),
                vec![0x00; n],
                vec![0xFF; n],
                vec![0xAA; n],
            ];
            for (p, data) in payloads.iter().enumerate() {
                let masks: Vec<Option<u8>> = if p == 0 {
                    (0..8).map(Some).chain([None]).collect()
                } else {
                    vec![None]
                };
                for mask in masks {
                    let ctx =
                        format!("encode_codewords v{version} ecl{ecl} payload{p} mask {mask:?}");
                    let ours = ours::QrCode::encode_codewords(
                        ours::Version::new(version),
                        our_ecl,
                        data,
                        mask.map(ours::Mask::new),
                    );
                    let theirs = theirs::QrCode::encode_codewords(
                        theirs::Version::new(version),
                        their_ecl,
                        data,
                        mask.map(theirs::Mask::new),
                    );
                    assert_same_symbol(&ctx, &ours, &theirs);
                }
            }
        }
    }
}

/// Automatic masking of sparse payloads (about one bit in eight set) in small
/// symbols. For these the eight masks' dark-module shares differ by whole 5%
/// steps. The balance rule then separates the best two masks in about one
/// symbol in 700, which dense random payloads almost never reach.
#[test]
fn automatic_mask_where_the_balance_rule_decides() {
    let mut rng = Rng(9);
    let capacity: Vec<Vec<usize>> = (3..=5)
        .map(|v| (0..4).map(|ecl| data_codewords(v, ecl)).collect())
        .collect();
    for trial in 0..6000 {
        let v = trial % 3;
        let ecl = trial / 3 % 4;
        let data: Vec<u8> = (0..capacity[v][ecl])
            .map(|_| (rng.next() & rng.next() & rng.next()) as u8)
            .collect();
        let version = v as u8 + 3;
        let ours =
            ours::QrCode::encode_codewords(ours::Version::new(version), LEVELS[ecl].0, &data, None);
        let theirs = theirs::QrCode::encode_codewords(
            theirs::Version::new(version),
            LEVELS[ecl].1,
            &data,
            None,
        );
        assert_same_symbol(&format!("sparse trial {trial}"), &ours, &theirs);
    }
}

// ---- Segments --------------------------------------------------------------

/// All three ECI designator widths at their edges, alone and before data.
#[test]
fn eci_designators() {
    for value in [0, 1, 127, 128, 16_383, 16_384, 999_999] {
        let ctx = format!("make_eci({value})");
        assert_same_segment(
            &ctx,
            &ours::QrSegment::make_eci(value),
            &theirs::QrSegment::make_eci(value),
        );
        Case::new(vec![Seg::Eci(value)], 0, 1, 40, None, true).check();
        Case::new(
            vec![Seg::Eci(value), Seg::Bytes("€".into())],
            2,
            1,
            40,
            Some(4),
            false,
        )
        .check();
    }
}

/// Segment factories and mode detection over every character class, including
/// empty text, partial groups (1 or 2 digits, 1 alphanumeric) and multi-byte UTF-8.
#[test]
fn segment_factories_and_mode_detection() {
    let mut rng = Rng(6);
    let mut texts: Vec<String> = vec![
        String::new(),
        "0".into(),
        "A".into(),
        "a".into(),
        " ".into(),
        "Hello, world!".into(),
        "HELLO WORLD".into(),
        "ÀÉ漢字🦀".into(),
        "\u{0}\u{7F}\u{80}".into(),
        String::from_utf8(ALPHANUMERIC.to_vec()).unwrap(),
    ];
    for len in 0..12 {
        texts.push(rng.text(len, NUMERIC));
        texts.push(rng.text(len, ALPHANUMERIC));
    }
    for text in &texts {
        let ctx = format!("{text:?}");
        assert_eq!(
            ours::QrSegment::is_numeric(text),
            theirs::QrSegment::is_numeric(text),
            "{ctx}"
        );
        assert_eq!(
            ours::QrSegment::is_alphanumeric(text),
            theirs::QrSegment::is_alphanumeric(text),
            "{ctx}"
        );
        let ours_segs = ours::QrSegment::make_segments(text);
        let theirs_segs = theirs::QrSegment::make_segments(text);
        assert_eq!(ours_segs.len(), theirs_segs.len(), "{ctx}");
        for (o, t) in ours_segs.iter().zip(&theirs_segs) {
            assert_same_segment(&ctx, o, t);
        }
        if theirs::QrSegment::is_numeric(text) {
            assert_same_segment(
                &ctx,
                &ours::QrSegment::make_numeric(text),
                &theirs::QrSegment::make_numeric(text),
            );
        }
        if theirs::QrSegment::is_alphanumeric(text) {
            assert_same_segment(
                &ctx,
                &ours::QrSegment::make_alphanumeric(text),
                &theirs::QrSegment::make_alphanumeric(text),
            );
        }
        assert_same_segment(
            &ctx,
            &ours::QrSegment::make_bytes(text.as_bytes()),
            &theirs::QrSegment::make_bytes(text.as_bytes()),
        );
        for (ecl, (o, t)) in LEVELS.iter().enumerate() {
            assert_same(
                &format!("encode_text {ctx} ecl{ecl}"),
                &ours::QrCode::encode_text(text, *o),
                &theirs::QrCode::encode_text(text, *t),
            );
        }
    }
    for c in (0..0x3000).filter_map(char::from_u32) {
        let s = c.to_string();
        assert_eq!(
            ours::QrSegment::is_numeric(&s),
            theirs::QrSegment::is_numeric(&s),
            "{c:?}"
        );
        assert_eq!(
            ours::QrSegment::is_alphanumeric(&s),
            theirs::QrSegment::is_alphanumeric(&s),
            "{c:?}"
        );
    }
}

/// Random mixtures of segments of every kind, empty segments included.
#[test]
fn mixed_segment_lists() {
    let mut rng = Rng(7);
    for _ in 0..400 {
        let segs: Vec<Seg> = (0..rng.below(6))
            .map(|_| {
                let len = rng.below(60);
                match rng.below(4) {
                    3 => Seg::Eci(rng.below(1_000_000) as u32),
                    mode => filled(&mut rng, mode, len),
                }
            })
            .collect();
        let (a, b) = (rng.below(40) as u8 + 1, rng.below(40) as u8 + 1);
        let mask = (rng.below(2) == 0).then(|| rng.below(8) as u8);
        Case::new(
            segs,
            rng.below(4),
            a.min(b),
            a.max(b),
            mask,
            rng.below(2) == 0,
        )
        .check();
    }
    // No segments at all.
    for ecl in 0..4 {
        Case::new(Vec::new(), ecl, 1, 40, None, true).check();
        Case::new(Vec::new(), ecl, 40, 40, Some(7), false).check();
    }
}

// ---- Nayuki's batch test ---------------------------------------------------

/// The trial distribution of Nayuki's cross-language batch tester:
/// - **Mode:** numeric, alphanumeric, ASCII or arbitrary bytes.
/// - **Length:** round((2 * max)^U) for U uniform in [0, 1).
/// - **Settings:** random level and version range, a forced mask half the
///   time, boost a fifth of the time.
#[test]
fn nayuki_batch_trials() {
    let mut rng = Rng(8);
    let ascii: Vec<u8> = (0..128).collect();
    for _ in 0..1500 {
        let mode = rng.below(4);
        let max: f64 = [7089.0, 4296.0, 2953.0, 2953.0][mode];
        let len = (2.0 * max).powf(rng.unit()).round() as usize;
        let data = match mode {
            0 => rng.bytes(len, NUMERIC),
            1 => rng.bytes(len, ALPHANUMERIC),
            2 => rng.bytes(len, &ascii),
            _ => rng.any_bytes(len),
        };
        // As in the batch worker: ASCII goes through make_segments, anything else is bytes.
        let segs = if data.is_ascii() {
            let text = String::from_utf8(data).unwrap();
            if text.is_empty() {
                Vec::new()
            } else if theirs::QrSegment::is_numeric(&text) {
                vec![Seg::Numeric(text)]
            } else if theirs::QrSegment::is_alphanumeric(&text) {
                vec![Seg::Alphanumeric(text)]
            } else {
                vec![Seg::Bytes(text.into_bytes())]
            }
        } else {
            vec![Seg::Bytes(data)]
        };
        let ecl = rng.below(4);
        let (a, b) = (rng.below(40) as u8 + 1, rng.below(40) as u8 + 1);
        let mask = (rng.unit() < 0.5).then(|| rng.below(8) as u8);
        let boost = rng.unit() < 0.2;
        Case::new(segs, ecl, a.min(b), a.max(b), mask, boost).check();
    }
}

// ---- Bit buffer ------------------------------------------------------------

/// `append_bits` at every width, with the largest value each width allows.
#[test]
fn bit_buffer_widths() {
    let mut ours_bb = ours::BitBuffer(Vec::new());
    let mut theirs_bb = theirs::BitBuffer(Vec::new());
    for len in 0..=31u8 {
        let max = (1u64 << len) as u32 - 1;
        for val in [0, max, max / 3] {
            ours_bb.append_bits(val, len);
            theirs_bb.append_bits(val, len);
        }
    }
    assert_eq!(ours_bb.0, theirs_bb.0);
}
