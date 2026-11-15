//! Position evaluation: how good a position is, as a number.

use crate::{board::Board, types::Color};

/// Scores `board` in centipawns (100 = one pawn) from the point of view of
/// the side to move: positive is good for them, negative for the opponent.
///
/// It counts material plus a bonus for where each piece stands
/// (piece-square tables). The board keeps that sum up to date as pieces
/// move, so this only has to pick the point of view.
#[must_use]
pub const fn evaluate(board: &Board) -> i32 {
    match board.side_to_move() {
        Color::White => board.score(),
        Color::Black => board.score().saturating_neg(),
    }
}
