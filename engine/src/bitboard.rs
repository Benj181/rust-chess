use crate::square::Square;
use std::{
    fmt,
    ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not},
};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Bitboard(u64);

impl Bitboard {
    pub const EMPTY: Self = Self(0);

    pub const FILE_A: Self = Self(0x0101_0101_0101_0101);
    pub const FILE_B: Self = Self(Self::FILE_A.0 << 1);
    pub const FILE_G: Self = Self(Self::FILE_A.0 << 6);
    pub const FILE_H: Self = Self(Self::FILE_A.0 << 7);

    pub const RANK_1: Self = Self(0xFF);
    pub const RANK_2: Self = Self(Self::RANK_1.0 << 8);
    pub const RANK_3: Self = Self(Self::RANK_1.0 << 16);
    pub const RANK_6: Self = Self(Self::RANK_1.0 << 40);
    pub const RANK_7: Self = Self(Self::RANK_1.0 << 48);
    pub const RANK_8: Self = Self(Self::RANK_1.0 << 56);

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

    #[must_use]
    pub(crate) const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    #[must_use]
    pub(crate) const fn north(self) -> Self {
        Self(self.0 << 8)
    }

    #[must_use]
    pub(crate) const fn south(self) -> Self {
        Self(self.0 >> 8)
    }

    #[must_use]
    pub(crate) const fn east(self) -> Self {
        Self(self.0 << 1 & !Self::FILE_A.0)
    }

    #[must_use]
    pub(crate) const fn west(self) -> Self {
        Self(self.0 >> 1 & !Self::FILE_H.0)
    }

    #[must_use]
    pub(crate) const fn north_east(self) -> Self {
        self.north().east()
    }

    #[must_use]
    pub(crate) const fn north_west(self) -> Self {
        self.north().west()
    }

    #[must_use]
    pub(crate) const fn south_east(self) -> Self {
        self.south().east()
    }

    #[must_use]
    pub(crate) const fn south_west(self) -> Self {
        self.south().west()
    }

    #[must_use]
    pub const fn pop_count(self) -> u32 {
        self.0.count_ones()
    }
}

macro_rules! impl_bit_op {
    ($trait:ident, $method:ident, $assign_trait:ident, $assign_method:ident, $op:tt) => {
        impl $trait for Bitboard {
            type Output = Self;

            fn $method(self, rhs: Self) -> Self::Output {
                Self(self.0 $op rhs.0)
            }
        }

        impl $assign_trait for Bitboard {
            fn $assign_method(&mut self, rhs: Self) {
                self.0 = self.0 $op rhs.0;
            }
        }
    };
}

impl_bit_op!(BitAnd, bitand, BitAndAssign, bitand_assign, &);
impl_bit_op!(BitOr, bitor, BitOrAssign, bitor_assign, |);
impl_bit_op!(BitXor, bitxor, BitXorAssign, bitxor_assign, ^);

impl Not for Bitboard {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}

#[derive(Clone, Debug)]
pub struct Squares(Bitboard);

impl Iterator for Squares {
    type Item = Square;

    fn next(&mut self) -> Option<Self::Item> {
        let bits = &mut self.0.0;
        if *bits == 0 {
            return None;
        }
        let idx = bits.trailing_zeros();
        *bits &= bits.wrapping_sub(1);
        Square::new(u8::try_from(idx).ok()?)
    }
}

impl IntoIterator for Bitboard {
    type Item = Square;
    type IntoIter = Squares;

    fn into_iter(self) -> Self::IntoIter {
        Squares(self)
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
