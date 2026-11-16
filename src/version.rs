// Derived from Project Nayuki's qrcodegen (MIT License); see LICENSE-MIT.

/// A QR Code version number, between 1 and 40 (inclusive).
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Version(u8);

impl Version {
    /// The minimum version number supported in the QR Code Model 2 standard.
    pub const MIN: Version = Version(1);

    /// The maximum version number supported in the QR Code Model 2 standard.
    pub const MAX: Version = Version(40);

    /// Creates a version object from the given number.
    ///
    /// Panics if the number is outside the range [1, 40].
    pub const fn new(ver: u8) -> Self {
        assert!(
            Version::MIN.0 <= ver && ver <= Version::MAX.0,
            "Version number out of range"
        );
        Self(ver)
    }

    /// Returns the value, which is in the range [1, 40].
    pub const fn value(self) -> u8 {
        self.0
    }

    /// Side length of a symbol of this version, in modules: 21 to 177.
    pub(crate) fn size(self) -> usize {
        usize::from(self.0) * 4 + 17
    }

    /// The next version up, or `None` at version 40.
    pub(crate) fn next(self) -> Option<Version> {
        (self < Version::MAX).then(|| Version(self.0 + 1))
    }
}

/// A mask pattern number, between 0 and 7 (inclusive).
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Mask(u8);

impl Mask {
    /// All eight mask patterns, in ascending order.
    pub(crate) const ALL: [Mask; 8] = [
        Mask(0),
        Mask(1),
        Mask(2),
        Mask(3),
        Mask(4),
        Mask(5),
        Mask(6),
        Mask(7),
    ];

    /// Creates a mask object from the given number.
    ///
    /// Panics if the number is outside the range [0, 7].
    pub const fn new(mask: u8) -> Self {
        assert!(mask <= 7, "Mask value out of range");
        Self(mask)
    }

    /// Returns the value, which is in the range [0, 7].
    pub const fn value(self) -> u8 {
        self.0
    }

    /// Whether this pattern inverts the module at column `x`, row `y`.
    pub(crate) fn inverts(self, x: usize, y: usize) -> bool {
        match self.0 {
            0 => (x + y).is_multiple_of(2),
            1 => y.is_multiple_of(2),
            2 => x.is_multiple_of(3),
            3 => (x + y).is_multiple_of(3),
            4 => (x / 3 + y / 2).is_multiple_of(2),
            5 => x * y % 2 + x * y % 3 == 0,
            6 => (x * y % 2 + x * y % 3).is_multiple_of(2),
            7 => ((x + y) % 2 + x * y % 3).is_multiple_of(2),
            _ => unreachable!("Mask::new keeps the value in 0..=7"),
        }
    }
}
