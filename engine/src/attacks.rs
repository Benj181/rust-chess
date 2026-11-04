use crate::bitboard::Bitboard;
use crate::direction::Direction;
use crate::piece::Color;
use crate::square::Square;

#[must_use]
const fn knight_attacks(sq: Square) -> Bitboard {
    let base = Bitboard::from_square(sq);
    let tall = base.north().north().union(base.south().south());
    let wide = base.east().east().union(base.west().west());
    tall.east()
        .union(tall.west())
        .union(wide.north())
        .union(wide.south())
}

#[must_use]
const fn king_attacks(sq: Square) -> Bitboard {
    let base = Bitboard::from_square(sq);
    let sides = base.west().union(base.east());
    let row = base.union(sides);
    sides.union(row.north()).union(row.south())
}

macro_rules! square_table {
    ($f:path) => {{
        let mut table = [Bitboard::EMPTY; 64];
        let mut i = 0;
        while i < 64 {
            if let Some(sq) = Square::new(i) {
                #[allow(
                    clippy::indexing_slicing,
                    reason = "const context: Square::as_usize() is always < 64, and an \
                              out-of-bounds index would be a compile error, not a panic"
                )]
                {
                    table[sq.as_usize()] = $f(sq);
                }
            }
            i = i.wrapping_add(1);
        }
        table
    }};
}

pub const KNIGHT_ATTACKS: [Bitboard; 64] = square_table!(knight_attacks);
pub const KING_ATTACKS: [Bitboard; 64] = square_table!(king_attacks);

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
    sliding_attacks(sq, occupied, &Direction::ALL)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sq(name: &str) -> Square {
        Square::from_algebraic(name).unwrap()
    }

    fn bb(squares: &[&str]) -> Bitboard {
        let mut b = Bitboard::EMPTY;
        for s in squares {
            b.set(sq(s));
        }
        b
    }

    #[test]
    fn rook() {
        let pos = "e6";
        let occupied = bb(&["b6", "e4", "e7", "g6"]);
        println!("occupied:\n{occupied}");
        println!("{}", bb(&[pos]));
        println!("{}", rook_attacks(sq(pos), occupied));
    }
}
