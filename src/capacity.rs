// Derived from Project Nayuki's qrcodegen (MIT License); see LICENSE-MIT.
//
// Symbol geometry and codeword budgets as functions of version and error
// correction level (ISO/IEC 18004, tables 1 and 9).

use crate::ecc::QrCodeEcc;
use crate::version::Version;

/// Error correction codewords per block, indexed by `[ecl][version]`.
/// Index 0 is padding and never read.
#[rustfmt::skip]
const ECC_CODEWORDS_PER_BLOCK: [[u8; 41]; 4] = [
    // 0,  1,  2,  3,  4,  5,  6,  7,  8,  9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40
    [0,  7, 10, 15, 20, 26, 18, 20, 24, 30, 18, 20, 24, 26, 30, 22, 24, 28, 30, 28, 28, 28, 28, 30, 30, 26, 28, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30], // Low
    [0, 10, 16, 26, 18, 24, 16, 18, 22, 22, 26, 30, 22, 22, 24, 24, 28, 28, 26, 26, 26, 26, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28], // Medium
    [0, 13, 22, 18, 26, 18, 24, 18, 22, 20, 24, 28, 26, 24, 20, 30, 24, 28, 28, 26, 30, 28, 30, 30, 30, 30, 28, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30], // Quartile
    [0, 17, 28, 22, 16, 22, 28, 26, 26, 24, 28, 24, 28, 22, 24, 24, 30, 28, 28, 26, 28, 30, 24, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30], // High
];

/// Error correction blocks, indexed by `[ecl][version]`.
/// Index 0 is padding and never read.
#[rustfmt::skip]
const NUM_ERROR_CORRECTION_BLOCKS: [[u8; 41]; 4] = [
    // 0, 1, 2, 3, 4, 5, 6, 7, 8, 9,10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40
    [0, 1, 1, 1, 1, 1, 2, 2, 2, 2, 4,  4,  4,  4,  4,  6,  6,  6,  6,  7,  8,  8,  9,  9, 10, 12, 12, 12, 13, 14, 15, 16, 17, 18, 19, 19, 20, 21, 22, 24, 25], // Low
    [0, 1, 1, 1, 2, 2, 4, 4, 4, 5, 5,  5,  8,  9,  9, 10, 10, 11, 13, 14, 16, 17, 17, 18, 20, 21, 23, 25, 26, 28, 29, 31, 33, 35, 37, 38, 40, 43, 45, 47, 49], // Medium
    [0, 1, 1, 2, 2, 4, 4, 6, 6, 8, 8,  8, 10, 12, 16, 12, 17, 16, 18, 21, 20, 23, 23, 25, 27, 29, 34, 34, 35, 38, 40, 43, 45, 48, 51, 53, 56, 59, 62, 65, 68], // Quartile
    [0, 1, 1, 2, 4, 4, 4, 5, 6, 8, 8, 11, 11, 16, 16, 18, 16, 19, 21, 25, 25, 25, 34, 30, 32, 35, 37, 40, 42, 45, 48, 51, 54, 57, 60, 63, 66, 70, 74, 77, 81], // High
];

/// How the codewords of one symbol split into Reed-Solomon blocks.
///
/// The first `num_short_blocks` blocks carry one data codeword fewer than the rest.
#[derive(Clone, Copy, Debug)]
pub(crate) struct BlockLayout {
    pub num_blocks: usize,
    pub num_short_blocks: usize,
    /// Data plus ECC codewords of a short block.
    pub short_block_len: usize,
    pub ecc_len: usize,
}

impl BlockLayout {
    pub fn new(ver: Version, ecl: QrCodeEcc) -> Self {
        let num_blocks = table(&NUM_ERROR_CORRECTION_BLOCKS, ver, ecl);
        let raw_codewords = num_raw_data_modules(ver) / 8;
        Self {
            num_blocks,
            num_short_blocks: num_blocks - raw_codewords % num_blocks,
            short_block_len: raw_codewords / num_blocks,
            ecc_len: table(&ECC_CODEWORDS_PER_BLOCK, ver, ecl),
        }
    }

    /// Data codewords in block `i`.
    pub fn data_len(&self, i: usize) -> usize {
        self.short_block_len - self.ecc_len + usize::from(i >= self.num_short_blocks)
    }
}

/// Data bits a symbol of this version holds once every function module is
/// excluded, remainder bits included (so not always a multiple of 8).
/// The result is in the range [208, 29648].
pub(crate) fn num_raw_data_modules(ver: Version) -> usize {
    let v = usize::from(ver.value());
    let mut result = (16 * v + 128) * v + 64;
    if v >= 2 {
        let num_align = v / 7 + 2;
        result -= (25 * num_align - 10) * num_align - 55;
        if v >= 7 {
            result -= 36;
        }
    }
    result
}

/// Data (not error correction) codewords in a symbol of this version and
/// level, remainder bits discarded.
pub(crate) fn num_data_codewords(ver: Version, ecl: QrCodeEcc) -> usize {
    num_raw_data_modules(ver) / 8
        - table(&ECC_CODEWORDS_PER_BLOCK, ver, ecl) * table(&NUM_ERROR_CORRECTION_BLOCKS, ver, ecl)
}

/// Ascending centre coordinates of the alignment patterns, shared by both
/// axes. Empty for version 1.
pub(crate) fn alignment_pattern_positions(ver: Version) -> Vec<usize> {
    let v = usize::from(ver.value());
    if v == 1 {
        return Vec::new();
    }
    let num_align = v / 7 + 2;
    let step = (v * 8 + num_align * 3 + 5) / (num_align * 4 - 4) * 2;
    let last = ver.size() - 7;
    std::iter::once(6)
        .chain((0..num_align - 1).rev().map(|i| last - i * step))
        .collect()
}

fn table(table: &[[u8; 41]; 4], ver: Version, ecl: QrCodeEcc) -> usize {
    usize::from(table[ecl.ordinal()][usize::from(ver.value())])
}
