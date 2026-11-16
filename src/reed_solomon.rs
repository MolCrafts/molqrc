// Derived from Project Nayuki's qrcodegen (MIT License); see LICENSE-MIT.
//
// Reed-Solomon error correction over GF(2^8) modulo x^8 + x^4 + x^3 + x^2 + 1
// (0x11D), and the block interleaving that spreads codewords over the symbol.

use crate::capacity::{num_data_codewords, num_raw_data_modules, BlockLayout};
use crate::ecc::QrCodeEcc;
use crate::version::Version;

/// Appends the error correction codewords to `data` and interleaves every
/// block into the final codeword sequence.
///
/// Panics if `data` is not exactly the data capacity of `ver` at `ecl`.
pub(crate) fn add_ecc_and_interleave(data: &[u8], ver: Version, ecl: QrCodeEcc) -> Vec<u8> {
    assert_eq!(data.len(), num_data_codewords(ver, ecl), "Illegal argument");
    let layout = BlockLayout::new(ver, ecl);
    let divisor = compute_divisor(layout.ecc_len);

    let mut blocks: Vec<(&[u8], Vec<u8>)> = Vec::with_capacity(layout.num_blocks);
    let mut rest = data;
    for i in 0..layout.num_blocks {
        let (block, tail) = rest.split_at(layout.data_len(i));
        rest = tail;
        blocks.push((block, compute_remainder(block, &divisor)));
    }

    // Column-major read over the blocks: all data codewords first (short
    // blocks simply have one column fewer), then all ECC codewords.
    let max_data_len = layout.data_len(layout.num_blocks - 1);
    let mut result = Vec::with_capacity(num_raw_data_modules(ver) / 8);
    for i in 0..max_data_len {
        result.extend(blocks.iter().filter_map(|(dat, _)| dat.get(i)));
    }
    for i in 0..layout.ecc_len {
        result.extend(blocks.iter().map(|(_, ecc)| ecc[i]));
    }
    result
}

/// Generator polynomial (x - r^0)(x - r^1)...(x - r^{degree-1}) with r = 0x02,
/// coefficients from the highest power down, the leading 1 dropped. For
/// example x^3 + 255x^2 + 8x + 93 is `[255, 8, 93]`.
///
/// Panics unless 1 <= degree <= 255.
pub(crate) fn compute_divisor(degree: usize) -> Vec<u8> {
    assert!((1..=255).contains(&degree), "Degree out of range");
    let mut result = vec![0u8; degree];
    result[degree - 1] = 1; // The monomial x^0
    let mut root: u8 = 1;
    for _ in 0..degree {
        // Multiply the current product by (x - root).
        for j in 0..degree {
            result[j] = multiply(result[j], root);
            if j + 1 < degree {
                result[j] ^= result[j + 1];
            }
        }
        root = multiply(root, 0x02);
    }
    result
}

/// Remainder of `data` times x^degree divided by the generator `divisor`:
/// the error correction codewords of one block.
pub(crate) fn compute_remainder(data: &[u8], divisor: &[u8]) -> Vec<u8> {
    let mut result = vec![0u8; divisor.len()];
    for &b in data {
        let factor = b ^ result[0];
        result.rotate_left(1);
        let last = result.len() - 1;
        result[last] = 0;
        for (r, &d) in result.iter_mut().zip(divisor) {
            *r ^= multiply(d, factor);
        }
    }
    result
}

/// Product of two field elements in GF(2^8/0x11D).
pub(crate) fn multiply(x: u8, y: u8) -> u8 {
    // Russian peasant multiplication.
    let mut z: u8 = 0;
    for i in (0..8).rev() {
        z = (z << 1) ^ ((z >> 7) * 0x1D);
        z ^= ((y >> i) & 1) * x;
    }
    z
}
