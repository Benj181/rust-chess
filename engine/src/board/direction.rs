//! [`Direction`]: the eight directions a piece can slide in.

use crate::types::{Bitboard, Square};

/// One of the eight compass directions on the board, seen from White's
/// side: north is toward rank 8 and east is toward the h-file.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Direction {
    North,
    South,
    East,
    West,
    NorthEast,
    NorthWest,
    SouthEast,
    SouthWest,
}

impl Direction {
    /// The four directions a rook slides in.
    pub const ROOK: [Self; 4] = [Self::North, Self::South, Self::East, Self::West];
    /// The four directions a bishop slides in.
    pub const BISHOP: [Self; 4] = [
        Self::NorthEast,
        Self::NorthWest,
        Self::SouthEast,
        Self::SouthWest,
    ];

    /// Every square of `bb` moved one step in this direction. Squares that
    /// would leave the board are dropped.
    #[must_use]
    pub const fn step(self, bb: Bitboard) -> Bitboard {
        match self {
            Self::North => bb.north(),
            Self::South => bb.south(),
            Self::East => bb.east(),
            Self::West => bb.west(),
            Self::NorthEast => bb.north_east(),
            Self::NorthWest => bb.north_west(),
            Self::SouthEast => bb.south_east(),
            Self::SouthWest => bb.south_west(),
        }
    }

    /// How many steps `sq` is from the edge of the board in this direction.
    #[must_use]
    pub const fn steps_to_edge(self, sq: Square) -> u8 {
        let north = 7u8.saturating_sub(sq.rank());
        let south = sq.rank();
        let east = 7u8.saturating_sub(sq.file());
        let west = sq.file();
        match self {
            Self::North => north,
            Self::South => south,
            Self::East => east,
            Self::West => west,
            Self::NorthEast => min(north, east),
            Self::NorthWest => min(north, west),
            Self::SouthEast => min(south, east),
            Self::SouthWest => min(south, west),
        }
    }
}

const fn min(a: u8, b: u8) -> u8 {
    if a < b { a } else { b }
}
