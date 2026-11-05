//! [`Game`]: a chess game from start to finish, with its history and result.

use crate::{
    board::Board,
    movegen::generate_legal,
    moves::{Move, MoveList},
    piece::Color,
};

/// How a game ended.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GameResult {
    /// One side won.
    Win {
        /// The side that won.
        winner: Color,
        /// Why the game was won.
        reason: WinReason,
    },
    /// Neither side won.
    Draw(DrawReason),
}

/// Why a game was won.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum WinReason {
    Checkmate,
    Resignation,
    Timeout,
}

/// Why a game was drawn.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DrawReason {
    Stalemate,
    ThreefoldRepetition,
    FiftyMoveRule,
    InsufficientMaterial,
    Agreement,
}

/// Why [`Game::play`] rejected a move.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MoveError {
    /// The game has already ended.
    GameOver,
    /// The move is not legal in the current position.
    IllegalMove,
}

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
    result: Option<GameResult>,
}

impl Game {
    /// Starts a game from the position in `fen`, or returns `None` if the FEN
    /// is invalid.
    ///
    /// If that position is already checkmate or stalemate, the game starts
    /// finished.
    #[must_use]
    pub fn from_fen(fen: &str) -> Option<Self> {
        let board = Board::from_fen(fen)?;
        Some(Self {
            result: Self::detect_result(&board),
            board,
            history: Vec::new(),
            moves: Vec::new(),
        })
    }

    /// The current position.
    #[must_use]
    pub const fn board(&self) -> &Board {
        &self.board
    }

    /// How the game ended, or `None` while it is still going.
    #[must_use]
    pub const fn result(&self) -> Option<GameResult> {
        self.result
    }

    /// The moves played so far, oldest first.
    #[must_use]
    pub const fn moves(&self) -> &[Move] {
        self.moves.as_slice()
    }

    #[must_use]
    fn detect_result(board: &Board) -> Option<GameResult> {
        let mut list = MoveList::new();
        generate_legal(board, &mut list);
        if list.is_empty() {
            if board.in_check() {
                Some(GameResult::Win {
                    winner: board.side_to_move().opposite(),
                    reason: WinReason::Checkmate,
                })
            } else {
                Some(GameResult::Draw(DrawReason::Stalemate))
            }
        } else {
            None
        }
    }

    /// The moves that can be played now, or an empty list once the game has
    /// ended.
    #[must_use]
    pub fn legal_moves(&self) -> MoveList {
        let mut list = MoveList::new();
        if self.result.is_none() {
            generate_legal(&self.board, &mut list);
        }
        list
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
        if !self.legal_moves().contains(mv) {
            return Err(MoveError::IllegalMove);
        }

        let next = self.board.make_move(mv);
        self.history.push(std::mem::replace(&mut self.board, next));
        self.moves.push(mv);
        self.result = Self::detect_result(&self.board);

        Ok(())
    }

    /// Takes back the last move, restoring the position and result before it.
    /// Does nothing at the start of the game.
    pub fn undo(&mut self) {
        if let Some(previous) = self.history.pop() {
            self.board = previous;
            self.moves.pop();
            self.result = Self::detect_result(&self.board);
        }
    }
}
