// Derived from Project Nayuki's qrcodegen (MIT License); see LICENSE-MIT.

//! Generates QR Codes (ISO/IEC 18004, Model 2) from text and binary data.
//!
//! The output is the raw module grid of the symbol. Every version from 1 to 40
//! and all four error correction levels are supported.
//!
//! - **Version.** The smallest version in a given range that holds the data is
//!   chosen.
//! - **Mask.** You can force one, or the library scores all eight and picks the best.
//! - **Error correction.** The level can be raised automatically when that does
//!   not need a larger version.
//! - **Segments.** Text is encoded as numeric, alphanumeric or byte segments.
//!   Mixed segment lists and ECI designators can be built by hand.
//!
//! The engine is derived from Project Nayuki's
//! [qrcodegen](https://www.nayuki.io/page/qr-code-generator-library), and its
//! output is checked against that library module for module. The test suite
//! covers every version, level, mode boundary and mask.
//!
//! # Examples
//!
//! ```
//! use molqrc::{QrCode, QrCodeEcc};
//!
//! let qr = QrCode::encode_text("Hello, world!", QrCodeEcc::Medium).unwrap();
//! for y in 0..qr.size() {
//!     let row: String = (0..qr.size())
//!         .map(|x| if qr.get_module(x, y) { '#' } else { ' ' })
//!         .collect();
//!     println!("{row}");
//! }
//! ```
//!
//! Manual parameters:
//!
//! ```
//! use molqrc::{Mask, QrCode, QrCodeEcc, QrSegment, Version};
//!
//! let segs = QrSegment::make_segments("3141592653589793238462643383");
//! let qr = QrCode::encode_segments_advanced(
//!     &segs,
//!     QrCodeEcc::High,
//!     Version::new(5),
//!     Version::new(5),
//!     Some(Mask::new(2)),
//!     false,
//! )
//! .unwrap();
//! assert_eq!((qr.version(), qr.mask()), (Version::new(5), Mask::new(2)));
//! ```

#![forbid(unsafe_code)]

mod bits;
mod canvas;
mod capacity;
mod ecc;
mod error;
mod penalty;
mod qrcode;
mod reed_solomon;
mod segment;
mod version;

pub use bits::BitBuffer;
pub use ecc::QrCodeEcc;
pub use error::DataTooLong;
pub use qrcode::QrCode;
pub use segment::{QrSegment, QrSegmentMode};
pub use version::{Mask, Version};

#[cfg(test)]
mod tests;
