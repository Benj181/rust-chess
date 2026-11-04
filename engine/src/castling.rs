//! Castling rights, and the squares involved in each of the four castling
//! moves, shared by move generation and
//! [`Board::make_move`](crate::board::Board::make_move).

use crate::{bitboard::Bitboard, moves::MoveKind, piece::Color, square::Square};

/// Which of the four castlings are still allowed, as a set of flags.
///
/// Build sets with [`insert`](Self::insert) and query them with
/// [`contains`](Self::contains).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct CastlingRights(u8);

impl CastlingRights {
    /// No castling rights.
    pub const NONE: Self = Self(0);
    /// White may castle king side.
    pub const WHITE_KING: Self = Self(0b0001);
    /// White may castle queen side.
    pub const WHITE_QUEEN: Self = Self(0b0010);
    /// Black may castle king side.
    pub const BLACK_KING: Self = Self(0b0100);
    /// Black may castle queen side.
    pub const BLACK_QUEEN: Self = Self(0b1000);

    /// Parses the castling field of a FEN string: `-`, or any combination of
    /// `K`, `Q`, `k` and `q`. Returns `None` for any other character.
    #[must_use]
    pub fn from_fen(s: &str) -> Option<Self> {
        let mut rights = Self::NONE;
        if s == "-" {
            return Some(rights);
        }
        for c in s.chars() {
            let right = match c {
                'K' => Self::WHITE_KING,
                'Q' => Self::WHITE_QUEEN,
                'k' => Self::BLACK_KING,
                'q' => Self::BLACK_QUEEN,
                _ => return None,
            };
            rights.insert(right);
        }
        Some(rights)
    }

    /// Whether every right in `right` is present.
    #[must_use]
    pub const fn contains(self, right: Self) -> bool {
        self.0 & right.0 == right.0
    }

    /// Adds the rights in `right`.
    pub const fn insert(&mut self, right: Self) {
        self.0 |= right.0;
    }

    /// Removes the rights in `right`.
    pub const fn remove(&mut self, right: Self) {
        self.0 &= !right.0;
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Castle {
    pub(crate) right: CastlingRights,
    pub(crate) kind: MoveKind,
    pub(crate) king_from: Square,
    pub(crate) king_to: Square,
    pub(crate) rook_from: Square,
    pub(crate) rook_to: Square,
    pub(crate) must_be_empty: Bitboard,
    pub(crate) must_be_safe: Bitboard,
}

impl Castle {
    const WHITE_KING_SIDE: Self = Self {
        right: CastlingRights::WHITE_KING,
        kind: MoveKind::KingCastle,
        king_from: Square::E1,
        king_to: Square::G1,
        rook_from: Square::H1,
        rook_to: Square::F1,
        must_be_empty: Bitboard::EMPTY.with(Square::F1).with(Square::G1),
        must_be_safe: Bitboard::EMPTY
            .with(Square::E1)
            .with(Square::F1)
            .with(Square::G1),
    };

    const WHITE_QUEEN_SIDE: Self = Self {
        right: CastlingRights::WHITE_QUEEN,
        kind: MoveKind::QueenCastle,
        king_from: Square::E1,
        king_to: Square::C1,
        rook_from: Square::A1,
        rook_to: Square::D1,
        must_be_empty: Bitboard::EMPTY
            .with(Square::B1)
            .with(Square::C1)
            .with(Square::D1),
        must_be_safe: Bitboard::EMPTY
            .with(Square::E1)
            .with(Square::D1)
            .with(Square::C1),
    };

    const BLACK_KING_SIDE: Self = Self {
        right: CastlingRights::BLACK_KING,
        kind: MoveKind::KingCastle,
        king_from: Square::E8,
        king_to: Square::G8,
        rook_from: Square::H8,
        rook_to: Square::F8,
        must_be_empty: Bitboard::EMPTY.with(Square::F8).with(Square::G8),
        must_be_safe: Bitboard::EMPTY
            .with(Square::E8)
            .with(Square::F8)
            .with(Square::G8),
    };

    const BLACK_QUEEN_SIDE: Self = Self {
        right: CastlingRights::BLACK_QUEEN,
        kind: MoveKind::QueenCastle,
        king_from: Square::E8,
        king_to: Square::C8,
        rook_from: Square::A8,
        rook_to: Square::D8,
        must_be_empty: Bitboard::EMPTY
            .with(Square::B8)
            .with(Square::C8)
            .with(Square::D8),
        must_be_safe: Bitboard::EMPTY
            .with(Square::E8)
            .with(Square::D8)
            .with(Square::C8),
    };

    pub(crate) const ALL: [Self; 4] = [
        Self::WHITE_KING_SIDE,
        Self::WHITE_QUEEN_SIDE,
        Self::BLACK_KING_SIDE,
        Self::BLACK_QUEEN_SIDE,
    ];

    #[must_use]
    pub(crate) const fn for_color(color: Color) -> [Self; 2] {
        match color {
            Color::White => [Self::WHITE_KING_SIDE, Self::WHITE_QUEEN_SIDE],
            Color::Black => [Self::BLACK_KING_SIDE, Self::BLACK_QUEEN_SIDE],
        }
    }
}
