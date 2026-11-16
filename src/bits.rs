// Derived from Project Nayuki's qrcodegen (MIT License); see LICENSE-MIT.

/// An appendable sequence of bits (0s and 1s), most significant bit first.
///
/// Mainly used to build the data of a [`QrSegment`](crate::QrSegment).
#[derive(Clone, PartialEq, Eq, Default, Debug)]
pub struct BitBuffer(pub Vec<bool>);

impl BitBuffer {
    /// Appends the given number of low-order bits of the given value to this buffer.
    ///
    /// Panics unless `len` &#x2264; 31 and `val` &lt; 2<sup>`len`</sup>.
    pub fn append_bits(&mut self, val: u32, len: u8) {
        assert!(len <= 31 && val >> len == 0, "Value out of range");
        self.0.extend((0..len).rev().map(|i| get_bit(val, i)));
    }
}

/// Whether bit `i` of `x` is set.
pub(crate) fn get_bit(x: u32, i: u8) -> bool {
    (x >> i) & 1 != 0
}
