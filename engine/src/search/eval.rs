//! Position evaluation: how good a position is, as a number.

use crate::{
    board::Board,
    types::{Color, PieceKind, Square},
};

/// Scores `board` in centipawns (100 = one pawn) from the point of view of
/// the side to move: positive is good for them, negative for the opponent.
///
/// It counts material plus a bonus for where each piece stands
/// (piece-square tables).
#[must_use]
pub fn evaluate(board: &Board) -> i32 {
    let us = board.side_to_move();

    let ours = side_score(board, us);
    let theirs = side_score(board, us.opposite());

    ours.saturating_sub(theirs)
}

fn side_score(board: &Board, color: Color) -> i32 {
    PieceKind::ALL
        .into_iter()
        .flat_map(|kind| {
            board
                .pieces_of(color, kind)
                .into_iter()
                .map(move |sq| material_value(kind).saturating_add(square_bonus(kind, color, sq)))
        })
        .sum()
}

const fn material_value(kind: PieceKind) -> i32 {
    match kind {
        PieceKind::Pawn => 100,
        PieceKind::Knight | PieceKind::Bishop => 300,
        PieceKind::Rook => 500,
        PieceKind::Queen => 900,
        PieceKind::King => 0,
    }
}

#[rustfmt::skip]
const KNIGHT_BONUS: [i32; 64] = [
    -50, -40, -30, -30, -30, -30, -40, -50,
    -40, -20,   0,   0,   0,   0, -20, -40,
    -30,   0,  10,  15,  15,  10,   0, -30,
    -30,   5,  15,  20,  20,  15,   5, -30,
    -30,   0,  15,  20,  20,  15,   0, -30,
    -30,   5,  10,  15,  15,  10,   5, -30,
    -40, -20,   0,   5,   5,   0, -20, -40,
    -50, -40, -30, -30, -30, -30, -40, -50,
];

#[rustfmt::skip]
const PAWN_BONUS: [i32; 64] = [
      0,   0,   0,   0,   0,   0,   0,   0,
     50,  50,  50,  50,  50,  50,  50,  50,
     10,  10,  20,  30,  30,  20,  10,  10,
      5,   5,  10,  25,  25,  10,   5,   5,
      0,   0,   0,  20,  20,   0,   0,   0,
      5,  -5, -10,   0,   0, -10,  -5,   5,
      5,  10,  10, -20, -20,  10,  10,   5,
      0,   0,   0,   0,   0,   0,   0,   0,
];

#[rustfmt::skip]
const BISHOP_BONUS: [i32; 64] = [
    -20, -10, -10, -10, -10, -10, -10, -20,
    -10,   0,   0,   0,   0,   0,   0, -10,
    -10,   0,   5,  10,  10,   5,   0, -10,
    -10,   5,   5,  10,  10,   5,   5, -10,
    -10,   0,  10,  10,  10,  10,   0, -10,
    -10,  10,  10,  10,  10,  10,  10, -10,
    -10,   5,   0,   0,   0,   0,   5, -10,
    -20, -10, -10, -10, -10, -10, -10, -20,
];

#[rustfmt::skip]
const ROOK_BONUS: [i32; 64] = [
      0,   0,   0,   0,   0,   0,   0,   0,
      5,  10,  10,  10,  10,  10,  10,   5,
     -5,   0,   0,   0,   0,   0,   0,  -5,
     -5,   0,   0,   0,   0,   0,   0,  -5,
     -5,   0,   0,   0,   0,   0,   0,  -5,
     -5,   0,   0,   0,   0,   0,   0,  -5,
     -5,   0,   0,   0,   0,   0,   0,  -5,
      0,   0,   0,   5,   5,   0,   0,   0,
];

#[rustfmt::skip]
const QUEEN_BONUS: [i32; 64] = [
    -20, -10, -10,  -5,  -5, -10, -10, -20,
    -10,   0,   0,   0,   0,   0,   0, -10,
    -10,   0,   5,   5,   5,   5,   0, -10,
     -5,   0,   5,   5,   5,   5,   0,  -5,
      0,   0,   5,   5,   5,   5,   0,  -5,
    -10,   5,   5,   5,   5,   5,   0, -10,
    -10,   0,   5,   0,   0,   0,   0, -10,
    -20, -10, -10,  -5,  -5, -10, -10, -20,
];

#[rustfmt::skip]
const KING_BONUS: [i32; 64] = [
    -30, -40, -40, -50, -50, -40, -40, -30,
    -30, -40, -40, -50, -50, -40, -40, -30,
    -30, -40, -40, -50, -50, -40, -40, -30,
    -30, -40, -40, -50, -50, -40, -40, -30,
    -20, -30, -30, -40, -40, -30, -30, -20,
    -10, -20, -20, -20, -20, -20, -20, -10,
     20,  20,   0,   0,   0,   0,  20,  20,
     20,  30,  10,   0,   0,  10,  30,  20,
];

fn square_bonus(kind: PieceKind, color: Color, sq: Square) -> i32 {
    let table = match kind {
        PieceKind::Pawn => &PAWN_BONUS,
        PieceKind::Knight => &KNIGHT_BONUS,
        PieceKind::Bishop => &BISHOP_BONUS,
        PieceKind::Rook => &ROOK_BONUS,
        PieceKind::Queen => &QUEEN_BONUS,
        PieceKind::King => &KING_BONUS,
    };

    let index = match color {
        Color::Black => usize::from(sq.index()),
        Color::White => usize::from(sq.index() ^ 0b111_000),
    };
    table.get(index).copied().unwrap_or(0)
}
