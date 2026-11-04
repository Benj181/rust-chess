use crate::{bitboard::Bitboard, direction::Direction, piece::Color, square::Square};

#[must_use]
pub const fn knight_attacks(sq: Square) -> Bitboard {
    let base = Bitboard::from_square(sq);
    let tall = base.north().north().union(base.south().south());
    let wide = base.east().east().union(base.west().west());
    tall.east()
        .union(tall.west())
        .union(wide.north())
        .union(wide.south())
}

#[must_use]
pub const fn king_attacks(sq: Square) -> Bitboard {
    let base = Bitboard::from_square(sq);
    let sides = base.west().union(base.east());
    let row = base.union(sides);
    sides.union(row.north()).union(row.south())
}

#[must_use]
pub const fn pawn_attacks(color: Color, start: Bitboard) -> Bitboard {
    match color {
        Color::White => start.north_east().union(start.north_west()),
        Color::Black => start.south_east().union(start.south_west()),
    }
}

fn sliding_attacks(sq: Square, occupied: Bitboard, dirs: &[Direction]) -> Bitboard {
    let mut attacks = Bitboard::EMPTY;
    for &dir in dirs {
        let mut cur = Bitboard::from_square(sq);
        for _ in 0..dir.steps_to_edge(sq) {
            cur = dir.step(cur);
            attacks |= cur;
            if !(cur & occupied).is_empty() {
                break;
            }
        }
    }
    attacks
}

#[must_use]
pub fn rook_attacks(sq: Square, occupied: Bitboard) -> Bitboard {
    sliding_attacks(sq, occupied, &Direction::ROOK)
}

#[must_use]
pub fn bishop_attacks(sq: Square, occupied: Bitboard) -> Bitboard {
    sliding_attacks(sq, occupied, &Direction::BISHOP)
}

#[must_use]
pub fn queen_attacks(sq: Square, occupied: Bitboard) -> Bitboard {
    rook_attacks(sq, occupied) | bishop_attacks(sq, occupied)
}
