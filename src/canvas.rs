// Derived from Project Nayuki's qrcodegen (MIT License); see LICENSE-MIT.
//
// The module grid while a symbol is being drawn. It tracks which modules are
// function patterns, which masking must leave alone.

use crate::bits::get_bit;
use crate::capacity::{alignment_pattern_positions, num_raw_data_modules};
use crate::ecc::QrCodeEcc;
use crate::version::{Mask, Version};

pub(crate) struct Canvas {
    pub size: usize,
    /// Row-major colours, `true` = dark.
    pub modules: Vec<bool>,
    is_function: Vec<bool>,
}

impl Canvas {
    /// A grid with every function pattern of `ver` drawn and the format
    /// information drawn for mask 0.
    pub fn with_function_patterns(ver: Version, ecl: QrCodeEcc) -> Self {
        let size = ver.size();
        let mut canvas = Self {
            size,
            modules: vec![false; size * size],
            is_function: vec![false; size * size],
        };

        // Timing patterns.
        for i in 0..size {
            canvas.set_function(6, i, i % 2 == 0);
            canvas.set_function(i, 6, i % 2 == 0);
        }

        // Finder patterns in three corners, overwriting some timing modules.
        canvas.draw_finder_pattern(3, 3);
        canvas.draw_finder_pattern(size - 4, 3);
        canvas.draw_finder_pattern(3, size - 4);

        // Alignment patterns everywhere except the three finder corners.
        let positions = alignment_pattern_positions(ver);
        let last = positions.len().saturating_sub(1);
        for (i, &x) in positions.iter().enumerate() {
            for (j, &y) in positions.iter().enumerate() {
                let finder_corner =
                    (i == 0 && j == 0) || (i == 0 && j == last) || (i == last && j == 0);
                if !finder_corner {
                    canvas.draw_alignment_pattern(x, y);
                }
            }
        }

        // Reserve the format area (rewritten once a mask is chosen) and draw
        // the version information.
        canvas.draw_format_bits(ecl, Mask::new(0));
        canvas.draw_version(ver);
        canvas
    }

    pub fn get(&self, x: usize, y: usize) -> bool {
        self.modules[y * self.size + x]
    }

    /// Draws both copies of the 15-bit format information (level, mask and
    /// a BCH(15,5) code, XOR 0x5412).
    pub fn draw_format_bits(&mut self, ecl: QrCodeEcc, mask: Mask) {
        let data = ecl.format_bits() << 3 | u32::from(mask.value());
        let bits = (data << 10 | bch_remainder(data, 10, 0x537)) ^ 0x5412;

        // First copy, around the top-left finder.
        for i in 0..6 {
            self.set_function(8, usize::from(i), get_bit(bits, i));
        }
        self.set_function(8, 7, get_bit(bits, 6));
        self.set_function(8, 8, get_bit(bits, 7));
        self.set_function(7, 8, get_bit(bits, 8));
        for i in 9..15 {
            self.set_function(14 - usize::from(i), 8, get_bit(bits, i));
        }

        // Second copy, split between the other two finders.
        let size = self.size;
        for i in 0..8 {
            self.set_function(size - 1 - usize::from(i), 8, get_bit(bits, i));
        }
        for i in 8..15 {
            self.set_function(8, size - 15 + usize::from(i), get_bit(bits, i));
        }
        self.set_function(8, size - 8, true); // The dark module
    }

    /// Draws the codeword bits in the zigzag order over every non-function
    /// module. Remainder bits (0 to 7 of them) stay light.
    ///
    /// Panics unless `data` holds exactly the raw codewords of `ver`.
    pub fn draw_codewords(&mut self, ver: Version, data: &[u8]) {
        assert_eq!(
            data.len(),
            num_raw_data_modules(ver) / 8,
            "Illegal argument"
        );
        let size = self.size;
        let total_bits = data.len() * 8;
        let mut i = 0;
        // Column pairs from the right edge; the vertical timing column 6 is skipped.
        let mut right = size - 1;
        loop {
            let upward = (right + 1) & 2 == 0;
            for vert in 0..size {
                let y = if upward { size - 1 - vert } else { vert };
                for x in [right, right - 1] {
                    if !self.is_function[y * size + x] && i < total_bits {
                        self.modules[y * size + x] = (data[i >> 3] >> (7 - (i & 7))) & 1 != 0;
                        i += 1;
                    }
                }
            }
            match right {
                1 => break,
                8 => right = 5,
                _ => right -= 2,
            }
        }
        debug_assert_eq!(i, total_bits);
    }

    /// XORs the pattern of `mask` into every non-function module. Applying
    /// the same mask twice restores the grid.
    pub fn apply_mask(&mut self, mask: Mask) {
        let size = self.size;
        for y in 0..size {
            for x in 0..size {
                let idx = y * size + x;
                self.modules[idx] ^= !self.is_function[idx] && mask.inverts(x, y);
            }
        }
    }

    /// Draws both copies of the 18-bit version information (version and a
    /// BCH(18,6) code), for versions 7 and up only.
    fn draw_version(&mut self, ver: Version) {
        if ver.value() < 7 {
            return;
        }
        let data = u32::from(ver.value());
        let bits = data << 12 | bch_remainder(data, 12, 0x1F25);
        for i in 0..18u8 {
            let bit = get_bit(bits, i);
            let a = self.size - 11 + usize::from(i % 3);
            let b = usize::from(i / 3);
            self.set_function(a, b, bit);
            self.set_function(b, a, bit);
        }
    }

    /// A 9x9 finder pattern with its separator, centred on (x, y); parts
    /// outside the symbol are clipped.
    fn draw_finder_pattern(&mut self, x: usize, y: usize) {
        for dy in -4isize..=4 {
            for dx in -4isize..=4 {
                let (Some(xx), Some(yy)) = (x.checked_add_signed(dx), y.checked_add_signed(dy))
                else {
                    continue;
                };
                if xx < self.size && yy < self.size {
                    let dist = dx.abs().max(dy.abs()); // Chebyshev distance
                    self.set_function(xx, yy, dist != 2 && dist != 4);
                }
            }
        }
    }

    /// A 5x5 alignment pattern centred on (x, y), entirely inside the symbol.
    fn draw_alignment_pattern(&mut self, x: usize, y: usize) {
        for dy in 0..5usize {
            for dx in 0..5usize {
                let dist = dx.abs_diff(2).max(dy.abs_diff(2));
                self.set_function(x + dx - 2, y + dy - 2, dist != 1);
            }
        }
    }

    fn set_function(&mut self, x: usize, y: usize, dark: bool) {
        let idx = y * self.size + x;
        self.modules[idx] = dark;
        self.is_function[idx] = true;
    }
}

/// Remainder of `data * x^degree` divided by the generator `poly` (whose
/// leading term is x^degree) over GF(2).
fn bch_remainder(data: u32, degree: u32, poly: u32) -> u32 {
    let mut rem = data;
    for _ in 0..degree {
        rem = (rem << 1) ^ ((rem >> (degree - 1)) * poly);
    }
    rem
}
