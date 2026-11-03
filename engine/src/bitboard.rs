use crate::square::Square;
use std::fmt;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not};

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

    // Shifts. Square index = rank * 8 + file, so north is +8 and east is +1.
    // Bits shifted off either end of the u64 simply disappear, which handles the
    // top and bottom edges. East/west shifts would wrap onto the neighbouring rank,
    // so the file they would wrap onto is masked out afterwards.

    #[must_use]
    pub const fn north(self) -> Self {
        Self(self.0 << 8)
    }

    #[must_use]
    pub const fn south(self) -> Self {
        Self(self.0 >> 8)
    }

    #[must_use]
    pub const fn east(self) -> Self {
        Self(self.0 << 1 & !Self::FILE_A.0)
    }

    #[must_use]
    pub const fn west(self) -> Self {
        Self(self.0 >> 1 & !Self::FILE_H.0)
    }

    #[must_use]
    pub const fn north_east(self) -> Self {
        self.north().east()
    }

    #[must_use]
    pub const fn north_west(self) -> Self {
        self.north().west()
    }

    #[must_use]
    pub const fn south_east(self) -> Self {
        self.south().east()
    }

    #[must_use]
    pub const fn south_west(self) -> Self {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn bb(squares: &[&str]) -> Bitboard {
        let mut b = Bitboard::EMPTY;
        for s in squares {
            b.set(Square::from_algebraic(s).unwrap());
        }
        b
    }

    #[test]
    fn operators() {
        let a = bb(&["a1", "b2"]);
        let b = bb(&["b2", "c3"]);
        assert_eq!(a & b, bb(&["b2"]));
        assert_eq!(a | b, bb(&["a1", "b2", "c3"]));
        assert_eq!(a ^ b, bb(&["a1", "c3"]));
        assert_eq!(!Bitboard::EMPTY, Bitboard(u64::MAX));
        assert_eq!(!a & a, Bitboard::EMPTY);

        let mut c = a;
        c &= b;
        assert_eq!(c, bb(&["b2"]));
        c |= a;
        assert_eq!(c, a);
        c ^= a;
        assert_eq!(c, Bitboard::EMPTY);
    }

    #[test]
    fn iterates_lowest_bit_first() {
        let b = bb(&["h8", "a1", "c3"]);
        let squares: Vec<_> = b.into_iter().collect();
        let expected: Vec<_> = ["a1", "c3", "h8"]
            .iter()
            .map(|s| Square::from_algebraic(s).unwrap())
            .collect();
        assert_eq!(squares, expected);
        assert_eq!(b.pop_count(), 3);
        assert_eq!(Bitboard::EMPTY.into_iter().next(), None);
    }

    fn sq(name: &str) -> Bitboard {
        Bitboard::from_square(Square::from_algebraic(name).unwrap())
    }

    #[test]
    fn masks() {
        assert_eq!(Bitboard::FILE_A.pop_count(), 8);
        assert!(Bitboard::FILE_A.contains(Square::from_algebraic("a8").unwrap()));
        assert!(Bitboard::FILE_H.contains(Square::from_algebraic("h1").unwrap()));
        assert!(Bitboard::RANK_1.contains(Square::from_algebraic("h1").unwrap()));
        assert!(Bitboard::RANK_8.contains(Square::from_algebraic("a8").unwrap()));
        assert_eq!(Bitboard::RANK_2.pop_count(), 8);
        assert_eq!(Bitboard::FILE_B & Bitboard::RANK_3, sq("b3"));
        assert_eq!(Bitboard::FILE_G & Bitboard::RANK_6, sq("g6"));
        assert_eq!(Bitboard::FILE_A & Bitboard::RANK_7, sq("a7"));
    }

    #[test]
    fn shifts() {
        assert_eq!(sq("e4").north(), sq("e5"));
        assert_eq!(sq("e4").south(), sq("e3"));
        assert_eq!(sq("e4").east(), sq("f4"));
        assert_eq!(sq("e4").west(), sq("d4"));
        assert_eq!(sq("e4").north_east(), sq("f5"));
        assert_eq!(sq("e4").north_west(), sq("d5"));
        assert_eq!(sq("e4").south_east(), sq("f3"));
        assert_eq!(sq("e4").south_west(), sq("d3"));
    }

    #[test]
    fn shifts_fall_off_edges() {
        assert_eq!(sq("h4").east(), Bitboard::EMPTY);
        assert_eq!(sq("a4").west(), Bitboard::EMPTY);
        assert_eq!(sq("e8").north(), Bitboard::EMPTY);
        assert_eq!(sq("e1").south(), Bitboard::EMPTY);
        assert_eq!(sq("h8").north_east(), Bitboard::EMPTY);
        assert_eq!(sq("a1").south_west(), Bitboard::EMPTY);
        assert_eq!(sq("h1").north_east(), Bitboard::EMPTY);
        assert_eq!(sq("a8").south_west(), Bitboard::EMPTY);
        // Every h-file square shifted east must vanish, not wrap onto the a-file.
        assert_eq!(Bitboard::FILE_H.east(), Bitboard::EMPTY);
        assert_eq!(Bitboard::FILE_A.west(), Bitboard::EMPTY);
    }
}
