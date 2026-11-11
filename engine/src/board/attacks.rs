//! Attack sets: the squares each kind of piece attacks from a square.
//!
//! An attack set includes squares holding the attacker's own pieces. Move
//! generation removes those.
//!
//! Sliding pieces use a table of rays, built on first use: the squares in
//! one direction from a square on an empty board. A ray is cut off behind
//! the nearest blocker by removing that blocker's own ray.

use crate::{
    board::direction::Direction,
    types::{Bitboard, Color, Square},
};
use std::{array, sync::LazyLock};

/// The squares a knight on `sq` attacks.
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

/// The squares a king on `sq` attacks: the up to eight squares around it.
#[must_use]
pub const fn king_attacks(sq: Square) -> Bitboard {
    let base = Bitboard::from_square(sq);
    let sides = base.west().union(base.east());
    let row = base.union(sides);
    sides.union(row.north()).union(row.south())
}

/// The squares attacked by any of `pawns`, all of them belonging to `color`:
/// one step diagonally forward. Forward pushes are not attacks.
#[must_use]
pub const fn pawn_attacks(color: Color, pawns: Bitboard) -> Bitboard {
    match color {
        Color::White => pawns.north_east().union(pawns.north_west()),
        Color::Black => pawns.south_east().union(pawns.south_west()),
    }
}

static RAYS: LazyLock<[[Bitboard; 64]; 8]> =
    LazyLock::new(|| Direction::ALL.map(|dir| array::from_fn(|index| ray_from(dir, index))));

fn ray_from(dir: Direction, index: usize) -> Bitboard {
    let Some(sq) = u8::try_from(index).ok().and_then(Square::new) else {
        return Bitboard::EMPTY;
    };
    let mut ray = Bitboard::EMPTY;
    let mut reached = Bitboard::from_square(sq);
    for _ in 0..dir.steps_to_edge(sq) {
        reached = dir.step(reached);
        ray |= reached;
    }
    ray
}

fn ray(dir: Direction, sq: Square) -> Bitboard {
    RAYS.get(dir.index())
        .and_then(|rays| rays.get(usize::from(sq.index())))
        .copied()
        .unwrap_or(Bitboard::EMPTY)
}

fn sliding_attacks(sq: Square, occupied: Bitboard, dirs: &[Direction]) -> Bitboard {
    let mut attacks = Bitboard::EMPTY;
    for &dir in dirs {
        let open = ray(dir, sq);
        let blockers = open & occupied;
        let nearest = if dir.is_ascending() {
            blockers.first()
        } else {
            blockers.last()
        };
        let hidden = nearest.map_or(Bitboard::EMPTY, |blocker| ray(dir, blocker));
        attacks |= open ^ hidden;
    }
    attacks
}

/// The squares a rook on `sq` attacks along its rank and file. A ray stops
/// at the first square in `occupied`, which is itself attacked.
#[must_use]
pub fn rook_attacks(sq: Square, occupied: Bitboard) -> Bitboard {
    sliding_attacks(sq, occupied, &Direction::ROOK)
}

/// The squares a bishop on `sq` attacks along its diagonals. A ray stops at
/// the first square in `occupied`, which is itself attacked.
#[must_use]
pub fn bishop_attacks(sq: Square, occupied: Bitboard) -> Bitboard {
    sliding_attacks(sq, occupied, &Direction::BISHOP)
}

/// The squares a queen on `sq` attacks: those of a rook and a bishop
/// together.
#[must_use]
pub fn queen_attacks(sq: Square, occupied: Bitboard) -> Bitboard {
    rook_attacks(sq, occupied) | bishop_attacks(sq, occupied)
}
