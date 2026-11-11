//! Attack sets: the squares each kind of piece attacks from a square.
//!
//! An attack set includes squares holding the attacker's own pieces. Move
//! generation removes those.
//!
//! Sliding pieces look their attacks up in a table, see the `magic` module.

use crate::{
    board::magic,
    types::{Bitboard, Color, Square},
};

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

/// The squares a rook on `sq` attacks along its rank and file. A ray stops
/// at the first square in `occupied`, which is itself attacked.
#[must_use]
pub fn rook_attacks(sq: Square, occupied: Bitboard) -> Bitboard {
    magic::rook(sq, occupied)
}

/// The squares a bishop on `sq` attacks along its diagonals. A ray stops at
/// the first square in `occupied`, which is itself attacked.
#[must_use]
pub fn bishop_attacks(sq: Square, occupied: Bitboard) -> Bitboard {
    magic::bishop(sq, occupied)
}

/// The squares a queen on `sq` attacks: those of a rook and a bishop
/// together.
#[must_use]
pub fn queen_attacks(sq: Square, occupied: Bitboard) -> Bitboard {
    rook_attacks(sq, occupied) | bishop_attacks(sq, occupied)
}
