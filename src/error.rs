// Derived from Project Nayuki's qrcodegen (MIT License); see LICENSE-MIT.

use std::fmt;

/// The error type when the supplied data does not fit any QR Code version.
///
/// Ways to handle this error include:
///
/// - Decrease the error correction level if it was greater than `QrCodeEcc::Low`.
/// - If `QrCode::encode_segments_advanced()` was called, increase the maxversion
///   argument if it was less than `Version::MAX`. (The other factory functions
///   already search every version up to `Version::MAX`.)
/// - Split the text into better segments to reduce the number of bits required.
/// - Make the text or binary data shorter.
/// - Change the text to fit the character set of a particular segment mode (e.g. alphanumeric).
/// - Propagate the error upward to the caller/user.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataTooLong {
    /// A segment holds more characters than its character count field can express.
    SegmentTooLong,
    /// The data needs this many bits (first value) but the largest version
    /// allowed holds only this many (second value).
    DataOverCapacity(usize, usize),
}

impl std::error::Error for DataTooLong {}

impl fmt::Display for DataTooLong {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::SegmentTooLong => write!(f, "Segment too long"),
            Self::DataOverCapacity(datalen, maxcapacity) => write!(
                f,
                "Data length = {datalen} bits, Max capacity = {maxcapacity} bits"
            ),
        }
    }
}
