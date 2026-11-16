// Derived from Project Nayuki's qrcodegen (MIT License); see LICENSE-MIT.

use crate::bits::BitBuffer;
use crate::canvas::Canvas;
use crate::capacity::num_data_codewords;
use crate::ecc::QrCodeEcc;
use crate::error::DataTooLong;
use crate::penalty::penalty_score;
use crate::reed_solomon::add_ecc_and_interleave;
use crate::segment::QrSegment;
use crate::version::{Mask, Version};

/// A QR Code symbol (ISO/IEC 18004, Model 2): an immutable square grid of
/// dark and light modules.
///
/// Ways to create one, from high to low level:
///
/// - [`QrCode::encode_text`] or [`QrCode::encode_binary`] from the payload.
/// - [`QrCode::encode_segments`] or [`QrCode::encode_segments_advanced`]
///   from a hand-made list of segments.
/// - [`QrCode::encode_codewords`] from the data codewords (segment headers and
///   padding included, error correction excluded) at a fixed version.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct QrCode {
    version: Version,
    size: i32,
    errorcorrectionlevel: QrCodeEcc,
    mask: Mask,
    /// Row-major, `true` = dark.
    modules: Vec<bool>,
}

impl QrCode {
    /// Encodes Unicode text at the given error correction level, in the
    /// smallest version that fits. The level may be raised if that does not
    /// need a larger version.
    ///
    /// Strings of up to 738 code points always fit at `QrCodeEcc::Low`.
    pub fn encode_text(text: &str, ecl: QrCodeEcc) -> Result<Self, DataTooLong> {
        QrCode::encode_segments(&QrSegment::make_segments(text), ecl)
    }

    /// Encodes binary data in byte mode at the given error correction level,
    /// in the smallest version that fits. The level may be raised if that
    /// does not need a larger version. At most 2953 bytes fit.
    pub fn encode_binary(data: &[u8], ecl: QrCodeEcc) -> Result<Self, DataTooLong> {
        QrCode::encode_segments(&[QrSegment::make_bytes(data)], ecl)
    }

    /// Encodes the given segments at the given error correction level, in the
    /// smallest version that fits. The level may be raised if that does not
    /// need a larger version.
    pub fn encode_segments(segs: &[QrSegment], ecl: QrCodeEcc) -> Result<Self, DataTooLong> {
        QrCode::encode_segments_advanced(segs, ecl, Version::MIN, Version::MAX, None, true)
    }

    /// Encodes the given segments in the smallest version in
    /// `minversion..=maxversion` that fits at level `ecl`.
    ///
    /// If `boostecl` is true the level is raised as far as the chosen version
    /// still holds the data. `mask` forces a mask pattern; `None` picks the one
    /// with the lowest penalty score.
    ///
    /// Panics if `minversion > maxversion`. Fails if the data fits no version
    /// in the range: [`DataTooLong::SegmentTooLong`] if a segment's character
    /// count overflows its count field at `maxversion`, otherwise
    /// [`DataTooLong::DataOverCapacity`] with the bits needed and the bits
    /// available at `maxversion`.
    pub fn encode_segments_advanced(
        segs: &[QrSegment],
        mut ecl: QrCodeEcc,
        minversion: Version,
        maxversion: Version,
        mask: Option<Mask>,
        boostecl: bool,
    ) -> Result<Self, DataTooLong> {
        assert!(minversion <= maxversion, "Invalid value");

        // The smallest version whose capacity holds the data.
        let mut version = minversion;
        let data_bits = loop {
            let capacity_bits = num_data_codewords(version, ecl) * 8;
            let used = QrSegment::total_bits(segs, version);
            match used {
                Some(n) if n <= capacity_bits => break n,
                _ if version >= maxversion => {
                    return Err(match used {
                        None => DataTooLong::SegmentTooLong,
                        Some(n) => DataTooLong::DataOverCapacity(n, capacity_bits),
                    })
                }
                _ => version = version.next().expect("version < maxversion <= MAX"),
            }
        };

        if boostecl {
            for higher in [QrCodeEcc::Medium, QrCodeEcc::Quartile, QrCodeEcc::High] {
                if data_bits <= num_data_codewords(version, higher) * 8 {
                    ecl = higher;
                }
            }
        }

        let datacodewords = data_codewords(segs, version, ecl, data_bits);
        Ok(QrCode::encode_codewords(version, ecl, &datacodewords, mask))
    }

    /// Builds the symbol from its data codewords at a fixed version and level.
    /// `mask` forces a mask pattern; `None` picks the one with the lowest
    /// penalty score.
    ///
    /// Panics unless `datacodewords` is exactly the data capacity of `ver` at `ecl`.
    pub fn encode_codewords(
        ver: Version,
        ecl: QrCodeEcc,
        datacodewords: &[u8],
        mask: Option<Mask>,
    ) -> Self {
        let mut canvas = Canvas::with_function_patterns(ver, ecl);
        canvas.draw_codewords(ver, &add_ecc_and_interleave(datacodewords, ver, ecl));

        let mask = mask.unwrap_or_else(|| best_mask(&mut canvas, ecl));
        canvas.apply_mask(mask);
        canvas.draw_format_bits(ecl, mask);

        QrCode {
            version: ver,
            size: ver.size() as i32,
            errorcorrectionlevel: ecl,
            mask,
            modules: canvas.modules,
        }
    }

    /// Returns this QR Code's version, in the range [1, 40].
    pub fn version(&self) -> Version {
        self.version
    }

    /// Returns this QR Code's size in modules, in the range [21, 177].
    pub fn size(&self) -> i32 {
        self.size
    }

    /// Returns this QR Code's error correction level.
    pub fn error_correction_level(&self) -> QrCodeEcc {
        self.errorcorrectionlevel
    }

    /// Returns this QR Code's mask, in the range [0, 7].
    pub fn mask(&self) -> Mask {
        self.mask
    }

    /// Returns the colour of the module at (x, y): `true` for dark, `false`
    /// for light. (0, 0) is the top left corner; coordinates outside the
    /// symbol are light.
    pub fn get_module(&self, x: i32, y: i32) -> bool {
        (0..self.size).contains(&x)
            && (0..self.size).contains(&y)
            && self.modules[(y * self.size + x) as usize]
    }
}

/// The mask with the lowest penalty score, the lowest-numbered on a tie. Each
/// candidate is scored with its own format bits drawn. Leaves the canvas
/// unmasked.
fn best_mask(canvas: &mut Canvas, ecl: QrCodeEcc) -> Mask {
    let mut best = (u32::MAX, Mask::new(0));
    for mask in Mask::ALL {
        canvas.apply_mask(mask);
        canvas.draw_format_bits(ecl, mask);
        let penalty = penalty_score(canvas);
        if penalty < best.0 {
            best = (penalty, mask);
        }
        canvas.apply_mask(mask); // XOR again undoes it
    }
    best.1
}

/// The data codewords: every segment's header and data, a terminator of up to
/// four zero bits, zero bits up to a byte boundary, then alternating pad bytes
/// 0xEC 0x11 up to the capacity of `version` at `ecl`.
fn data_codewords(
    segs: &[QrSegment],
    version: Version,
    ecl: QrCodeEcc,
    data_bits: usize,
) -> Vec<u8> {
    let capacity_bits = num_data_codewords(version, ecl) * 8;
    let mut bb = BitBuffer(Vec::with_capacity(capacity_bits));
    for seg in segs {
        let ccbits = seg.mode().num_char_count_bits(version);
        let numchars = u32::try_from(seg.num_chars()).expect("total_bits bounds the count");
        bb.append_bits(seg.mode().mode_bits(), 4);
        bb.append_bits(numchars, ccbits);
        bb.0.extend_from_slice(seg.data());
    }
    debug_assert_eq!(bb.0.len(), data_bits);

    let terminator = (capacity_bits - bb.0.len()).min(4);
    bb.0.resize(bb.0.len() + terminator, false);
    bb.0.resize(bb.0.len().next_multiple_of(8), false);
    for pad in [0xEC, 0x11].into_iter().cycle() {
        if bb.0.len() >= capacity_bits {
            break;
        }
        bb.append_bits(pad, 8);
    }

    bb.0.chunks(8)
        .map(|byte| byte.iter().fold(0u8, |acc, &bit| acc << 1 | u8::from(bit)))
        .collect()
}
