//! Move ordering: trying the most promising moves first, so alpha-beta can
//! cut off the rest sooner.

use crate::{
    board::Board,
    movegen::generate_pseudo_legal,
    search::eval::material_value,
    types::{Move, MoveKind, MoveList, PieceKind},
};
use std::cmp::Reverse;

const HASH_MOVE: i32 = 1_000_000;
const TACTICAL: i32 = 10_000;

/// Every pseudo-legal move, best guess first: the move remembered from an
/// earlier search of this position, then captures and queen promotions
/// (biggest win first), then the quiet moves.
pub fn ordered_moves(board: &Board, hash_move: Option<Move>) -> MoveList {
    let mut moves = MoveList::new();
    generate_pseudo_legal(board, &mut moves);
    moves.sort_by_key(|&mv| {
        Reverse(if Some(mv) == hash_move {
            HASH_MOVE
        } else {
            guess(board, mv)
        })
    });
    moves
}

/// Only the pseudo-legal captures and queen promotions, biggest win first.
pub fn ordered_captures(board: &Board) -> MoveList {
    let mut all = MoveList::new();
    generate_pseudo_legal(board, &mut all);

    let mut loud = MoveList::new();
    for &mv in all.iter().filter(|mv| is_loud(mv.kind())) {
        loud.push(mv);
    }
    loud.sort_by_key(|&mv| Reverse(guess(board, mv)));
    loud
}

fn is_loud(kind: MoveKind) -> bool {
    kind.is_capture() || kind.promotion() == Some(PieceKind::Queen)
}

fn guess(board: &Board, mv: Move) -> i32 {
    let kind = mv.kind();
    if !is_loud(kind) {
        return 0;
    }
    let value_at = |sq| {
        board
            .piece_at(sq)
            .map_or(0, |piece| material_value(piece.kind()))
    };

    let victim = match kind {
        MoveKind::EnPassant => material_value(PieceKind::Pawn),
        _ if kind.is_capture() => value_at(mv.to()),
        _ => 0,
    };
    let promotion = kind.promotion().map_or(0, material_value);
    let attacker = value_at(mv.from()) / 10;

    TACTICAL
        .saturating_add(victim)
        .saturating_add(promotion)
        .saturating_sub(attacker)
}
