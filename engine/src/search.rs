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
        ordering::{ordered_captures, ordered_moves},
        tt::{Bound, Entry, Table},
    },
    types::Move,
};

const MATE: i32 = 100_000;
const MATE_BOUND: i32 = 99_000;
const INFINITY: i32 = 1_000_000;

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
    };
    let mut report = Report {
        best: board.legal_moves().iter().next().copied(),
        score: 0,
        depth: 0,
        nodes: 0,
    };

    for depth in 1..=limits.depth {
        let score = searcher.negamax(board, depth, 0, -INFINITY, INFINITY);
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
}

impl Searcher {
    const fn count_node(&mut self) -> bool {
        self.nodes = self.nodes.saturating_add(1);
        if self.nodes > self.max_nodes {
            self.stopped = true;
        }
        self.stopped
    }

    fn negamax(&mut self, board: &Board, depth: u32, ply: i32, mut alpha: i32, beta: i32) -> i32 {
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
            && entry.depth >= depth
        {
            let score = from_table(entry.score, ply);
            let usable = match entry.bound {
                Bound::Exact => true,
                Bound::Lower => score >= beta,
                Bound::Upper => score <= alpha,
            };
            if usable {
                return score;
            }
        }

        let mut best: Option<(i32, Move)> = None;
        let mut bound = Bound::Upper;

        for &mv in &ordered_moves(board, remembered.map(|entry| entry.best)) {
            let Some(child) = board.try_move(mv) else {
                continue;
            };
            let score = self
                .negamax(
                    &child,
                    next_depth,
                    ply.saturating_add(1),
                    beta.saturating_neg(),
                    alpha.saturating_neg(),
                )
                .saturating_neg();
            if self.stopped {
                return 0;
            }

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

        for &mv in &ordered_captures(board) {
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
