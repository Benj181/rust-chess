//! Search: looking ahead through the moves to find the best one.

use crate::{board::Board, eval::evaluate, movegen::generate_legal, types::MoveList};

const MATE: i32 = 100_000;

/// Scores `board` by searching `depth` plies ahead (negamax), in centipawns
/// from the point of view of the side to move. At depth 0 it is just the
/// evaluation: material plus piece-square tables.
///
/// Being checkmated scores `-100_000` and stalemate scores `0`. A mate is
/// only seen when the mated side still has depth left, so a mate in one
/// needs depth 2.
///
/// ```
/// use engine::{search::negamax, Board};
///
/// assert_eq!(negamax(&Board::start(), 0), 0);
///
/// let white = Board::from_fen("4k3/8/8/8/3N4/8/8/4K3 w - - 0 1").unwrap();
/// let black = Board::from_fen("4k3/8/8/8/3N4/8/8/4K3 b - - 0 1").unwrap();
/// assert!(negamax(&white, 0) > 300);
/// assert_eq!(negamax(&black, 0), -negamax(&white, 0));
/// ```
#[must_use]
pub fn negamax(board: &Board, depth: u32) -> i32 {
    let Some(next) = depth.checked_sub(1) else {
        return evaluate(board);
    };

    let mut moves = MoveList::new();
    generate_legal(board, &mut moves);
    moves
        .into_iter()
        .map(|mv| negamax(&board.make_move(*mv), next).saturating_neg())
        .max()
        .unwrap_or_else(|| if board.in_check() { -MATE } else { 0 })
}
