// Derived from Project Nayuki's qrcodegen (MIT License); see LICENSE-MIT.

/// The error correction level in a QR Code symbol.
///
/// Levels are ordered from the weakest to the strongest protection.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum QrCodeEcc {
    /// The QR Code can tolerate about 7% erroneous codewords.
    Low,
    /// The QR Code can tolerate about 15% erroneous codewords.
    Medium,
    /// The QR Code can tolerate about 25% erroneous codewords.
    Quartile,
    /// The QR Code can tolerate about 30% erroneous codewords.
    High,
}

impl QrCodeEcc {
    /// Row index into the per-level tables, in the range 0 to 3.
    pub(crate) fn ordinal(self) -> usize {
        self as usize
    }

    /// The 2-bit value this level carries in the format information.
    pub(crate) fn format_bits(self) -> u32 {
        match self {
            QrCodeEcc::Low => 1,
            QrCodeEcc::Medium => 0,
            QrCodeEcc::Quartile => 3,
            QrCodeEcc::High => 2,
        }
    }
}
