//! Move ordering: trying the most promising moves first, so alpha-beta can
//! cut off the rest sooner.

use crate::{
    board::Board,
    movegen::{generate_pseudo_legal, generate_pseudo_legal_captures},
    search::eval::material_value,
    types::{Color, Move, MoveKind, MoveList, PieceKind},
};

const HASH_MOVE: i32 = 1_000_000;
const TACTICAL: i32 = 100_000;
const KILLER: i32 = 90_000;
const HISTORY_MAX: i32 = 80_000;
const MAX_MOVES: usize = 256;

/// The two quiet moves that most recently caused a cutoff at one distance
/// from the root ("killer moves"). A move that refuted one line often
/// refutes its sibling lines too.
pub type Killers = [Option<Move>; 2];

/// How often each quiet move, by side and from/to squares, has caused a
/// cutoff anywhere in the tree. Quiet moves are tried in this order.
pub struct History {
    scores: Vec<[[i32; 64]; 64]>,
}

impl History {
    pub fn new() -> Self {
        Self {
            scores: vec![[[0; 64]; 64]; 2],
        }
    }

    fn slot(&mut self, side: Color, mv: Move) -> Option<&mut i32> {
        let side = match side {
            Color::White => 0,
            Color::Black => 1,
        };
        self.scores
            .get_mut(side)?
            .get_mut(usize::from(mv.from().index()))?
            .get_mut(usize::from(mv.to().index()))
    }

    fn score(&self, side: Color, mv: Move) -> i32 {
        let side = match side {
            Color::White => 0,
            Color::Black => 1,
        };
        self.scores
            .get(side)
            .and_then(|from| from.get(usize::from(mv.from().index())))
            .and_then(|to| to.get(usize::from(mv.to().index())))
            .copied()
            .unwrap_or(0)
    }

    /// Records that the quiet move `mv` caused a cutoff `depth` plies above
    /// the leaves. Cutoffs high in the tree count for more.
    pub fn reward(&mut self, side: Color, mv: Move, depth: u32) {
        let bonus = i32::try_from(depth.saturating_mul(depth)).unwrap_or(HISTORY_MAX);
        if let Some(slot) = self.slot(side, mv) {
            *slot = slot.saturating_add(bonus).min(HISTORY_MAX);
        }
    }
}

/// Hands out moves best guess first.
///
/// It finds the best remaining move only when asked for the next one, so a
/// position that is cut off after its first move never pays for sorting the
/// rest.
pub struct Picker {
    moves: MoveList,
    scores: [i32; MAX_MOVES],
    next: usize,
}

impl Picker {
    fn new(moves: MoveList, score: impl Fn(Move) -> i32) -> Self {
        let mut scores = [0; MAX_MOVES];
        for (slot, &mv) in scores.iter_mut().zip(&moves) {
            *slot = score(mv);
        }
        Self {
            moves,
            scores,
            next: 0,
        }
    }
}

impl Iterator for Picker {
    type Item = Move;

    fn next(&mut self) -> Option<Move> {
        let remaining = self.scores.get(self.next..self.moves.len())?;
        let (offset, _) = remaining
            .iter()
            .enumerate()
            .rev()
            .max_by_key(|&(_, score)| score)?;
        let best = self.next.saturating_add(offset);

        self.scores.swap(self.next, best);
        self.moves.swap(self.next, best);
        let mv = self.moves.as_slice().get(self.next).copied();
        self.next = self.next.saturating_add(1);
        mv
    }
}

/// Every pseudo-legal move, best guess first: the move remembered from an
/// earlier search of this position, then captures and queen promotions
/// (biggest win first), then the killer moves, then the other quiet moves
/// by their history.
pub fn ordered_moves(
    board: &Board,
    hash_move: Option<Move>,
    killers: Killers,
    history: &History,
) -> Picker {
    let mut moves = MoveList::new();
    generate_pseudo_legal(board, &mut moves);

    let side = board.side_to_move();
    Picker::new(moves, |mv| {
        if Some(mv) == hash_move {
            HASH_MOVE
        } else if is_loud(mv.kind()) {
            tactical_guess(board, mv)
        } else if killers.contains(&Some(mv)) {
            KILLER
        } else {
            history.score(side, mv)
        }
    })
}

/// Only the pseudo-legal captures and queen promotions, biggest win first.
pub fn ordered_captures(board: &Board) -> Picker {
    let mut moves = MoveList::new();
    generate_pseudo_legal_captures(board, &mut moves);
    Picker::new(moves, |mv| tactical_guess(board, mv))
}

/// Whether a move changes the material: a capture or a queen promotion.
pub fn is_loud(kind: MoveKind) -> bool {
    kind.is_capture() || kind.promotion() == Some(PieceKind::Queen)
}

fn tactical_guess(board: &Board, mv: Move) -> i32 {
    let kind = mv.kind();
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
