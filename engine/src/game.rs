//! [`Game`]: a chess game from start to finish, with its history and result.

mod result;

pub use result::{DrawReason, GameResult, MoveError, WinReason};

use crate::{
    board::Board,
    types::{Bitboard, Color, Move, MoveList, PieceKind, Square},
};
use std::cmp::Ordering;

/// A game in progress or finished: the current position, every earlier
/// position, the moves played and the result.
///
/// The result is checked after every move, so a game ends on its own by
/// checkmate or stalemate.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Game {
    board: Board,
    history: Vec<Board>,
    moves: Vec<Move>,
    san: Vec<String>,
    legal: MoveList,
    result: Option<GameResult>,
}

impl Game {
    /// Starts a game from the standard starting position.
    #[must_use]
    pub fn new() -> Self {
        Self::from_board(Board::start())
    }

    /// The current position.
    #[must_use]
    pub const fn board(&self) -> &Board {
        &self.board
    }

    /// The position after `ply` half-moves: 0 is the start, `moves().len()`
    /// is the current position, past that is `None`.
    #[must_use]
    pub fn board_at(&self, ply: usize) -> Option<&Board> {
        match ply.cmp(&self.history.len()) {
            Ordering::Less => self.history.get(ply),
            Ordering::Equal => Some(&self.board),
            Ordering::Greater => None,
        }
    }

    /// The moves played so far, oldest first.
    #[must_use]
    pub const fn moves(&self) -> &[Move] {
        self.moves.as_slice()
    }

    /// The moves played so far in standard algebraic notation, like `Nf3`,
    /// oldest first.
    #[must_use]
    pub const fn san_moves(&self) -> &[String] {
        self.san.as_slice()
    }

    /// The last move played, or `None` before the first move.
    #[must_use]
    pub fn last_move(&self) -> Option<Move> {
        self.moves.last().copied()
    }

    /// How the game ended, or `None` while it is still going.
    #[must_use]
    pub const fn result(&self) -> Option<GameResult> {
        self.result
    }

    /// The moves that can be played now, or an empty list once the game has
    /// ended. They are generated once per position, so this is cheap.
    #[must_use]
    pub const fn legal_moves(&self) -> &MoveList {
        &self.legal
    }

    /// The legal move from `from` to `to`, promoting to `promotion` if it is a
    /// promotion, or `None` if there is no such move.
    #[must_use]
    pub fn find_move(
        &self,
        from: Square,
        to: Square,
        promotion: Option<PieceKind>,
    ) -> Option<Move> {
        self.legal.find(from, to, promotion)
    }

    /// The squares the piece on `from` can legally move to. Empty if the
    /// square is empty, holds a piece of the side not to move, or the game
    /// has ended.
    #[must_use]
    pub fn targets(&self, from: Square) -> Bitboard {
        self.legal
            .iter()
            .filter(|mv| mv.from() == from)
            .fold(Bitboard::EMPTY, |targets, mv| targets.with(mv.to()))
    }

    /// Whether `sq` holds a piece the side to move may pick up, which is never
    /// the case once the game has ended.
    #[must_use]
    pub fn can_select(&self, sq: Square) -> bool {
        self.result.is_none()
            && self
                .board
                .piece_at(sq)
                .is_some_and(|piece| piece.color() == self.board.side_to_move())
    }

    /// Plays `mv` and updates the result.
    ///
    /// # Errors
    ///
    /// Returns [`MoveError::GameOver`] if the game has ended, or
    /// [`MoveError::IllegalMove`] if `mv` is not legal in the current position.
    pub fn play(&mut self, mv: Move) -> Result<(), MoveError> {
        if self.result.is_some() {
            return Err(MoveError::GameOver);
        }
        if !self.legal.contains(mv) {
            return Err(MoveError::IllegalMove);
        }

        let next = self.board.make_move(mv);
        self.san.push(self.board.san(mv));
        self.history.push(std::mem::replace(&mut self.board, next));
        self.moves.push(mv);
        self.refresh();

        Ok(())
    }

    /// Takes back the last move, restoring the position and result before it.
    /// Does nothing at the start of the game.
    pub fn undo(&mut self) {
        if let Some(previous) = self.history.pop() {
            self.board = previous;
            self.moves.pop();
            self.san.pop();
            self.refresh();
        }
    }

    /// Ends the game with `loser` resigning, so the other side wins.
    ///
    /// # Errors
    ///
    /// Returns [`MoveError::GameOver`] if the game has already ended.
    pub const fn resign(&mut self, loser: Color) -> Result<(), MoveError> {
        self.finish(GameResult::Win {
            winner: loser.opposite(),
            reason: WinReason::Resignation,
        })
    }

    /// Ends the game as a draw both players agreed to.
    ///
    /// # Errors
    ///
    /// Returns [`MoveError::GameOver`] if the game has already ended.
    pub const fn agree_draw(&mut self) -> Result<(), MoveError> {
        self.finish(GameResult::Draw(DrawReason::Agreement))
    }

    fn from_board(board: Board) -> Self {
        let mut game = Self {
            board,
            history: Vec::new(),
            moves: Vec::new(),
            san: Vec::new(),
            legal: MoveList::new(),
            result: None,
        };
        game.refresh();
        game
    }

    fn refresh(&mut self) {
        self.legal = self.board.legal_moves();
        self.result = self.detect_result();
        if self.result.is_some() {
            self.legal = MoveList::new();
        }
    }

    fn detect_result(&self) -> Option<GameResult> {
        if self.legal.is_empty() {
            return Some(if self.board.in_check() {
                GameResult::Win {
                    winner: self.board.side_to_move().opposite(),
                    reason: WinReason::Checkmate,
                }
            } else {
                GameResult::Draw(DrawReason::Stalemate)
            });
        }
        if self.board.halfmove_clock() >= 100 {
            return Some(GameResult::Draw(DrawReason::FiftyMoveRule));
        }
        if self.occurrences() >= 3 {
            return Some(GameResult::Draw(DrawReason::ThreefoldRepetition));
        }
        if self.board.insufficient_material() {
            return Some(GameResult::Draw(DrawReason::InsufficientMaterial));
        }
        None
    }

    fn occurrences(&self) -> usize {
        self.history
            .iter()
            .filter(|earlier| earlier.same_position(&self.board))
            .count()
            .saturating_add(1)
    }

    const fn finish(&mut self, result: GameResult) -> Result<(), MoveError> {
        if self.result.is_some() {
            return Err(MoveError::GameOver);
        }
        self.result = Some(result);
        self.legal = MoveList::new();
        Ok(())
    }
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}
