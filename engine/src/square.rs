//! [`Square`]: one of the 64 squares of the board.

use std::fmt;

/// One of the 64 squares, always valid.
///
/// Squares are numbered a1 = 0, b1 = 1, …, h1 = 7, a2 = 8, …, h8 = 63, so
/// the index is `rank * 8 + file` with both counted from 0. A square displays
/// in algebraic notation, like `e4`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Square(u8);

impl Square {
    pub const A1: Self = Self(0);
    pub const B1: Self = Self(1);
    pub const C1: Self = Self(2);
    pub const D1: Self = Self(3);
    pub const E1: Self = Self(4);
    pub const F1: Self = Self(5);
    pub const G1: Self = Self(6);
    pub const H1: Self = Self(7);
    pub const A8: Self = Self(56);
    pub const B8: Self = Self(57);
    pub const C8: Self = Self(58);
    pub const D8: Self = Self(59);
    pub const E8: Self = Self(60);
    pub const F8: Self = Self(61);
    pub const G8: Self = Self(62);
    pub const H8: Self = Self(63);

    /// The square with `index`, or `None` if `index` is 64 or more.
    #[must_use]
    pub const fn new(index: u8) -> Option<Self> {
        if index >= 64 {
            return None;
        }

        Some(Self(index))
    }

    /// The square on `file` and `rank`, both counted from 0 (the a-file and
    /// rank 1 are 0). Returns `None` if either is 8 or more.
    #[must_use]
    pub fn from_file_rank(file: u8, rank: u8) -> Option<Self> {
        if file >= 8 {
            return None;
        }
        Self::new(rank.checked_mul(8)?.checked_add(file)?)
    }

    /// Parses algebraic notation such as `e4`. Returns `None` for anything else.
    #[must_use]
    pub fn from_algebraic(s: &str) -> Option<Self> {
        let mut chars = s.chars();
        let file = "abcdefgh".find(chars.next()?)?;
        let rank = "12345678".find(chars.next()?)?;
        if chars.next().is_some() {
            return None;
        }
        Self::from_file_rank(u8::try_from(file).ok()?, u8::try_from(rank).ok()?)
    }

    #[must_use]
    pub(crate) const fn from_index_masked(index: u8) -> Self {
        Self(index & 63)
    }

    /// The index of the square, from 0 (a1) to 63 (h8).
    #[must_use]
    pub const fn index(self) -> u8 {
        self.0
    }

    /// The file, from 0 (the a-file) to 7 (the h-file).
    #[must_use]
    pub const fn file(self) -> u8 {
        self.0 % 8
    }

    /// The rank, from 0 (rank 1) to 7 (rank 8).
    #[must_use]
    pub const fn rank(self) -> u8 {
        self.0 / 8
    }
}

impl fmt::Display for Square {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let file = char::from(b'a'.saturating_add(self.file()));
        let rank = self.rank().saturating_add(1);
        write!(f, "{file}{rank}")
    }
}
