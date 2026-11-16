// Derived from Project Nayuki's qrcodegen (MIT License); see LICENSE-MIT.

use crate::bits::BitBuffer;
use crate::version::Version;

/// The set of all legal characters in alphanumeric mode, where each
/// character's value is its index in this string.
const ALPHANUMERIC_CHARSET: &str = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ $%*+-./:";

/// Describes how a segment's data bits are interpreted.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum QrSegmentMode {
    Numeric,
    Alphanumeric,
    Byte,
    Kanji,
    Eci,
}

impl QrSegmentMode {
    /// The 4-bit mode indicator written before each segment.
    pub(crate) fn mode_bits(self) -> u32 {
        match self {
            QrSegmentMode::Numeric => 0x1,
            QrSegmentMode::Alphanumeric => 0x2,
            QrSegmentMode::Byte => 0x4,
            QrSegmentMode::Kanji => 0x8,
            QrSegmentMode::Eci => 0x7,
        }
    }

    /// Bit width of the character count field at the given version, 0 to 16.
    ///
    /// The width steps up between versions 9/10 and 26/27.
    pub(crate) fn num_char_count_bits(self, ver: Version) -> u8 {
        let widths = match self {
            QrSegmentMode::Numeric => [10, 12, 14],
            QrSegmentMode::Alphanumeric => [9, 11, 13],
            QrSegmentMode::Byte => [8, 16, 16],
            QrSegmentMode::Kanji => [8, 10, 12],
            QrSegmentMode::Eci => [0, 0, 0],
        };
        widths[usize::from((ver.value() + 7) / 17)]
    }
}

/// A segment of character, binary or control data in a QR Code symbol.
///
/// The mid-level way to create a segment is a factory function such as
/// [`QrSegment::make_numeric`]; the low-level way is [`QrSegment::new`].
///
/// A segment imposes no length limit of its own. Even in the most favourable
/// conditions a QR Code holds at most 7089 characters, so longer segments
/// always fail to encode.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct QrSegment {
    mode: QrSegmentMode,
    /// Characters for numeric/alphanumeric/kanji mode, bytes for byte mode,
    /// 0 for ECI. Not the bit length of `data`.
    numchars: usize,
    data: Vec<bool>,
}

impl QrSegment {
    /// Returns a segment representing the given binary data encoded in byte mode.
    ///
    /// Any text can be encoded this way as its UTF-8 bytes.
    pub fn make_bytes(data: &[u8]) -> Self {
        let mut bb = BitBuffer(Vec::with_capacity(data.len() * 8));
        for &b in data {
            bb.append_bits(u32::from(b), 8);
        }
        QrSegment::new(QrSegmentMode::Byte, data.len(), bb.0)
    }

    /// Returns a segment representing the given string of decimal digits encoded in numeric mode.
    ///
    /// Panics if the string contains non-digit characters.
    pub fn make_numeric(text: &str) -> Self {
        assert!(
            QrSegment::is_numeric(text),
            "String contains non-numeric characters"
        );
        let mut bb = BitBuffer(Vec::with_capacity(text.len() * 10 / 3 + 4));
        // Each group of up to 3 digits takes 10, 7 or 4 bits.
        for chunk in text.as_bytes().chunks(3) {
            let value = chunk
                .iter()
                .fold(0, |acc, &b| acc * 10 + u32::from(b - b'0'));
            bb.append_bits(value, chunk.len() as u8 * 3 + 1);
        }
        QrSegment::new(QrSegmentMode::Numeric, text.len(), bb.0)
    }

    /// Returns a segment representing the given text encoded in alphanumeric mode.
    ///
    /// The characters allowed are: 0 to 9, A to Z (uppercase only), space,
    /// dollar, percent, asterisk, plus, hyphen, period, slash, colon.
    ///
    /// Panics if the string contains non-encodable characters.
    pub fn make_alphanumeric(text: &str) -> Self {
        let mut bb = BitBuffer(Vec::with_capacity(text.len() * 11 / 2 + 6));
        // Each pair of characters takes 11 bits, a final single one 6.
        for chunk in text.as_bytes().chunks(2) {
            let value = chunk
                .iter()
                .fold(0, |acc, &b| acc * 45 + alphanumeric_value(b));
            bb.append_bits(value, chunk.len() as u8 * 5 + 1);
        }
        QrSegment::new(QrSegmentMode::Alphanumeric, text.len(), bb.0)
    }

    /// Returns zero or one segment representing the given Unicode text: numeric
    /// mode if every character is a digit, else alphanumeric mode if every
    /// character is in that set, else byte mode over the UTF-8 encoding.
    /// Empty text gives no segments.
    pub fn make_segments(text: &str) -> Vec<Self> {
        if text.is_empty() {
            Vec::new()
        } else if QrSegment::is_numeric(text) {
            vec![QrSegment::make_numeric(text)]
        } else if QrSegment::is_alphanumeric(text) {
            vec![QrSegment::make_alphanumeric(text)]
        } else {
            vec![QrSegment::make_bytes(text.as_bytes())]
        }
    }

    /// Returns a segment representing an Extended Channel Interpretation
    /// (ECI) designator with the given assignment value.
    ///
    /// Panics unless the value is below 1 000 000.
    pub fn make_eci(assignval: u32) -> Self {
        let mut bb = BitBuffer(Vec::with_capacity(24));
        if assignval < 1 << 7 {
            bb.append_bits(assignval, 8);
        } else if assignval < 1 << 14 {
            bb.append_bits(0b10, 2);
            bb.append_bits(assignval, 14);
        } else if assignval < 1_000_000 {
            bb.append_bits(0b110, 3);
            bb.append_bits(assignval, 21);
        } else {
            panic!("ECI assignment value out of range");
        }
        QrSegment::new(QrSegmentMode::Eci, 0, bb.0)
    }

    /// Creates a segment with the given attributes and data.
    ///
    /// The character count must agree with the mode and the bit length of
    /// `data`; this is not checked.
    pub fn new(mode: QrSegmentMode, numchars: usize, data: Vec<bool>) -> Self {
        Self {
            mode,
            numchars,
            data,
        }
    }

    /// Returns the mode indicator of this segment.
    pub fn mode(&self) -> QrSegmentMode {
        self.mode
    }

    /// Returns the character count field of this segment.
    pub fn num_chars(&self) -> usize {
        self.numchars
    }

    /// Returns the data bits of this segment.
    pub fn data(&self) -> &[bool] {
        &self.data
    }

    /// Tests whether the given string can be encoded as a segment in numeric mode,
    /// that is, whether every character is in the range 0 to 9.
    pub fn is_numeric(text: &str) -> bool {
        text.chars().all(|c| c.is_ascii_digit())
    }

    /// Tests whether the given string can be encoded as a segment in alphanumeric mode,
    /// that is, whether every character is in the set: 0 to 9, A to Z (uppercase
    /// only), space, dollar, percent, asterisk, plus, hyphen, period, slash, colon.
    pub fn is_alphanumeric(text: &str) -> bool {
        text.chars().all(|c| ALPHANUMERIC_CHARSET.contains(c))
    }

    /// Number of bits the segments take at the given version, headers included.
    ///
    /// `None` if a segment has more characters than its count field can hold
    /// at this version, or if the total overflows `usize`.
    pub(crate) fn total_bits(segs: &[Self], version: Version) -> Option<usize> {
        segs.iter().try_fold(0usize, |total, seg| {
            let ccbits = seg.mode.num_char_count_bits(version);
            // ccbits reaches 16, while usize may be only 16 bits wide.
            if let Some(limit) = 1usize.checked_shl(ccbits.into()) {
                if seg.numchars >= limit {
                    return None;
                }
            }
            total
                .checked_add(4 + usize::from(ccbits))?
                .checked_add(seg.data.len())
        })
    }
}

/// Index of a byte in the alphanumeric character set.
///
/// Panics if the byte is not in the set; a non-ASCII character never is, as
/// none of its UTF-8 bytes are.
fn alphanumeric_value(b: u8) -> u32 {
    let index = ALPHANUMERIC_CHARSET
        .bytes()
        .position(|c| c == b)
        .expect("String contains unencodable characters in alphanumeric mode");
    index as u32
}
