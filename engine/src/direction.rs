use crate::{bitboard::Bitboard, square::Square};

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
    pub const ROOK: [Self; 4] = [Self::North, Self::South, Self::East, Self::West];
    pub const BISHOP: [Self; 4] = [
        Self::NorthEast,
        Self::NorthWest,
        Self::SouthEast,
        Self::SouthWest,
    ];

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
