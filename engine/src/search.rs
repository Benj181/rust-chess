//! Search: looking ahead through the moves to find the best one.
//!
//! [`search`] is the entry point. Inside it is a negamax search with
//! alpha-beta pruning, move ordering, a quiescence search at the leaves,
//! iterative deepening and a transposition table.

mod eval;
mod ordering;
mod tt;

use crate::{
    board::Board,
    search::{
        eval::evaluate,
        ordering::{History, Killers, is_loud, ordered_captures, ordered_moves},
        tt::{Bound, Entry, Table},
    },
    types::{Move, PieceKind},
};

const MATE: i32 = 100_000;
const MATE_BOUND: i32 = 99_000;
const INFINITY: i32 = 1_000_000;
const MAX_PLY: usize = 128;
const NULL_MOVE_MIN_DEPTH: u32 = 3;
const NULL_MOVE_REDUCTION: u32 = 3;
const REDUCE_MIN_DEPTH: u32 = 3;
const REDUCE_AFTER: u32 = 3;
const REDUCE_MORE_AFTER: u32 = 8;

/// When a [`search`] has to stop: whichever limit is reached first.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Limits {
    /// The deepest search to run, in plies (half-moves).
    pub depth: u32,
    /// The most positions to visit. This is the time limit: the same count
    /// takes about the same time on one machine, and it works in the browser,
    /// where there is no clock to read.
    pub nodes: u64,
}

/// What a [`search`] found.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Report {
    /// The best move, or `None` if the side to move has no legal move.
    pub best: Option<Move>,
    /// The score of the position in centipawns, from the point of view of
    /// the side to move. A forced mate is `100_000` minus the number of
    /// plies until the mate, negative when it is the side to move that gets
    /// mated.
    pub score: i32,
    /// The deepest search that finished.
    pub depth: u32,
    /// How many positions were visited.
    pub nodes: u64,
}

/// Searches `board` for the best move of the side to move.
///
/// It searches 1 ply deep, then 2, then 3 and so on (iterative deepening)
/// until a limit is reached or a forced mate is found, and reports the result
/// of the deepest search that finished.
///
/// ```
/// use engine::{search::{search, Limits}, Board};
///
/// let board = Board::from_fen("6k1/5ppp/8/8/8/8/8/3R2K1 w - - 0 1").unwrap();
/// let report = search(&board, Limits { depth: 4, nodes: 100_000 });
///
/// assert_eq!(report.best, board.parse_uci("d1d8"));
/// assert_eq!(report.score, 100_000 - 1);
/// ```
#[must_use]
pub fn search(board: &Board, limits: Limits) -> Report {
    let mut searcher = Searcher {
        table: Table::new(),
        nodes: 0,
        max_nodes: limits.nodes,
        stopped: false,
        root_move: None,
        killers: [[None; 2]; MAX_PLY],
        history: History::new(),
    };
    let mut report = Report {
        best: board.legal_moves().iter().next().copied(),
        score: 0,
        depth: 0,
        nodes: 0,
    };

    for depth in 1..=limits.depth {
        let score = searcher.negamax(board, depth, 0, -INFINITY, INFINITY, false);
        if searcher.stopped {
            break;
        }
        report.best = searcher.root_move;
        report.score = score;
        report.depth = depth;
        if score.abs() >= MATE_BOUND {
            break;
        }
    }

    report.nodes = searcher.nodes;
    report
}

struct Searcher {
    table: Table,
    nodes: u64,
    max_nodes: u64,
    stopped: bool,
    root_move: Option<Move>,
    killers: [Killers; MAX_PLY],
    history: History,
}

impl Searcher {
    const fn count_node(&mut self) -> bool {
        self.nodes = self.nodes.saturating_add(1);
        if self.nodes > self.max_nodes {
            self.stopped = true;
        }
        self.stopped
    }

    fn killers_at(&self, ply: i32) -> Killers {
        usize::try_from(ply)
            .ok()
            .and_then(|ply| self.killers.get(ply))
            .copied()
            .unwrap_or_default()
    }

    fn remember_quiet_cutoff(&mut self, board: &Board, mv: Move, depth: u32, ply: i32) {
        self.history.reward(board.side_to_move(), mv, depth);
        if let Some(killers) = usize::try_from(ply)
            .ok()
            .and_then(|ply| self.killers.get_mut(ply))
            && killers.first() != Some(&Some(mv))
        {
            *killers = [Some(mv), killers.first().copied().flatten()];
        }
    }

    fn passing_is_enough(&mut self, board: &Board, depth: u32, ply: i32, beta: i32) -> bool {
        if depth < NULL_MOVE_MIN_DEPTH || !has_pieces(board) || evaluate(board) < beta {
            return false;
        }
        let score = self
            .negamax(
                &board.pass(),
                depth.saturating_sub(NULL_MOVE_REDUCTION),
                ply.saturating_add(1),
                beta.saturating_neg(),
                beta.saturating_neg().saturating_add(1),
                false,
            )
            .saturating_neg();
        !self.stopped && score >= beta
    }

    fn reply(&mut self, child: &Board, depth: u32, ply: i32, alpha: i32, beta: i32) -> i32 {
        self.negamax(
            child,
            depth,
            ply,
            beta.saturating_neg(),
            alpha.saturating_neg(),
            true,
        )
        .saturating_neg()
    }

    fn scout(
        &mut self,
        child: &Board,
        depth: u32,
        reduction: u32,
        ply: i32,
        alpha: i32,
        beta: i32,
    ) -> i32 {
        let just_above = alpha.saturating_add(1);
        let quick = self.reply(
            child,
            depth.saturating_sub(reduction),
            ply,
            alpha,
            just_above,
        );
        let trusted = reduction == 0 && beta == just_above;
        if quick <= alpha || trusted || self.stopped {
            quick
        } else {
            self.reply(child, depth, ply, alpha, beta)
        }
    }

    fn negamax(
        &mut self,
        board: &Board,
        depth: u32,
        ply: i32,
        mut alpha: i32,
        beta: i32,
        may_pass: bool,
    ) -> i32 {
        let in_check = board.in_check();
        let Some(next_depth) = depth.checked_sub(1).or_else(|| in_check.then_some(0)) else {
            return self.quiesce(board, alpha, beta);
        };
        if self.count_node() {
            return 0;
        }
        if ply > 0 && (board.halfmove_clock() >= 100 || board.insufficient_material()) {
            return 0;
        }

        let key = board.zobrist();
        let remembered = self.table.probe(key);
        if ply > 0
            && let Some(entry) = remembered
            && let Some(score) = settled_score(entry, depth, ply, alpha, beta)
        {
            return score;
        }

        let narrow = beta.saturating_sub(alpha) <= 1;
        if narrow
            && may_pass
            && !in_check
            && beta.abs() < MATE_BOUND
            && self.passing_is_enough(board, depth, ply, beta)
        {
            return beta;
        }
        if self.stopped {
            return 0;
        }

        let mut best: Option<(i32, Move)> = None;
        let mut bound = Bound::Upper;
        let mut searched: u32 = 0;
        let moves = ordered_moves(
            board,
            remembered.map(|entry| entry.best),
            self.killers_at(ply),
            &self.history,
        );

        for mv in moves {
            let Some(child) = board.try_move(mv) else {
                continue;
            };
            let quiet = !is_loud(mv.kind());
            let child_ply = ply.saturating_add(1);

            let reduction = if quiet && !in_check {
                reduction(depth, searched)
            } else {
                0
            };
            let reduction = if reduction > 0 && child.in_check() {
                0
            } else {
                reduction
            };
            let score = if searched == 0 {
                self.reply(&child, next_depth, child_ply, alpha, beta)
            } else {
                self.scout(&child, next_depth, reduction, child_ply, alpha, beta)
            };
            if self.stopped {
                return 0;
            }
            searched = searched.saturating_add(1);

            if best.is_none_or(|(best_score, _)| score > best_score) {
                best = Some((score, mv));
                if ply == 0 {
                    self.root_move = Some(mv);
                }
            }
            if score > alpha {
                alpha = score;
                bound = Bound::Exact;
            }
            if alpha >= beta {
                bound = Bound::Lower;
                if quiet {
                    self.remember_quiet_cutoff(board, mv, depth, ply);
                }
                break;
            }
        }

        let Some((score, best_move)) = best else {
            return if in_check {
                ply.saturating_sub(MATE)
            } else {
                0
            };
        };
        self.table.store(Entry {
            key,
            depth,
            score: to_table(score, ply),
            bound,
            best: best_move,
        });
        score
    }

    fn quiesce(&mut self, board: &Board, mut alpha: i32, beta: i32) -> i32 {
        if self.count_node() {
            return 0;
        }

        let mut best = evaluate(board);
        if best >= beta {
            return best;
        }
        alpha = alpha.max(best);

        for mv in ordered_captures(board) {
            let Some(child) = board.try_move(mv) else {
                continue;
            };
            let score = self
                .quiesce(&child, beta.saturating_neg(), alpha.saturating_neg())
                .saturating_neg();
            if self.stopped {
                return 0;
            }

            best = best.max(score);
            alpha = alpha.max(score);
            if alpha >= beta {
                break;
            }
        }

        best
    }
}

fn settled_score(entry: Entry, depth: u32, ply: i32, alpha: i32, beta: i32) -> Option<i32> {
    let score = from_table(entry.score, ply);
    let usable = entry.depth >= depth
        && match entry.bound {
            Bound::Exact => true,
            Bound::Lower => score >= beta,
            Bound::Upper => score <= alpha,
        };
    usable.then_some(score)
}

const fn reduction(depth: u32, searched: u32) -> u32 {
    if depth < REDUCE_MIN_DEPTH || searched < REDUCE_AFTER {
        0
    } else if searched < REDUCE_MORE_AFTER {
        1
    } else {
        2
    }
}

fn has_pieces(board: &Board) -> bool {
    let us = board.side_to_move();
    [
        PieceKind::Knight,
        PieceKind::Bishop,
        PieceKind::Rook,
        PieceKind::Queen,
    ]
    .into_iter()
    .any(|kind| !board.pieces_of(us, kind).is_empty())
}

const fn to_table(score: i32, ply: i32) -> i32 {
    if score >= MATE_BOUND {
        score.saturating_add(ply)
    } else if score <= -MATE_BOUND {
        score.saturating_sub(ply)
    } else {
        score
    }
}

const fn from_table(score: i32, ply: i32) -> i32 {
    if score >= MATE_BOUND {
        score.saturating_sub(ply)
    } else if score <= -MATE_BOUND {
        score.saturating_add(ply)
    } else {
        score
    }
}
