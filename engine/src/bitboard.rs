use crate::square::Square;
use std::fmt;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Bitboard(u64);

impl Bitboard {
    pub const EMPTY: Self = Self(0);

    #[must_use]
    pub const fn from_square(sq: Square) -> Self {
        Self(1 << sq.index())
    }

    #[must_use]
    pub const fn contains(self, sq: Square) -> bool {
        let sq_bb = Self::from_square(sq).0;
        self.0 & sq_bb == sq_bb
    }

    pub const fn set(&mut self, sq: Square) {
        self.0 |= Self::from_square(sq).0;
    }

    pub const fn clear(&mut self, sq: Square) {
        self.0 &= !Self::from_square(sq).0;
    }

    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0 == Self::EMPTY.0
    }
}

impl fmt::Display for Bitboard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (rank, label) in (0..8).rev().zip("87654321".chars()) {
            write!(f, "{label}")?;
            for file in 0..8 {
                let occupied =
                    Square::from_file_rank(file, rank).is_some_and(|sq| self.contains(sq));
                write!(f, " {}", if occupied { '1' } else { '.' })?;
            }
            writeln!(f)?;
        }
        write!(f, "  a b c d e f g h")
    }
}
