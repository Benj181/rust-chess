//! [`Game`]: a chess game from start to finish, with its history and result.

use crate::{
    board::Board,
    movegen::generate_legal,
    moves::{Move, MoveList},
    piece::{Color, PieceKind},
    square::Square,
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
    /// The UCI parsing failed due to invalid notation.
    InvalidNotation,
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
    fn from_board(board: Board) -> Self {
        let mut g = Self {
            board,
            history: Vec::new(),
            moves: Vec::new(),
            result: None,
        };
        g.result = g.detect_result();
        g
    }

    /// Starts a game from the position in `fen`, or returns `None` if the FEN
    /// is invalid.
    ///
    /// If that position is already checkmate or stalemate, the game starts
    /// finished.
    #[must_use]
    pub fn from_fen(fen: &str) -> Option<Self> {
        let board = Board::from_fen(fen)?;
        Some(Self::from_board(board))
    }

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
    fn detect_result(&self) -> Option<GameResult> {
        let mut list = MoveList::new();
        generate_legal(&self.board, &mut list);
        if list.is_empty() {
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
        if self.repetitions() >= 2 {
            return Some(GameResult::Draw(DrawReason::ThreefoldRepetition));
        }
        if self.board.insufficient_material() {
            return Some(GameResult::Draw(DrawReason::InsufficientMaterial));
        }
        None
    }

    fn repetitions(&self) -> usize {
        self.history
            .iter()
            .filter(|earlier| earlier.same_position(&self.board))
            .count()
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
        self.result = self.detect_result();

        Ok(())
    }

    fn parse_uci(s: &str) -> Option<(Square, Square, Option<PieceKind>)> {
        let from = Square::from_algebraic(s.get(..2)?)?;
        let to = Square::from_algebraic(s.get(2..4)?)?;
        let promo = match s.get(4..)? {
            "" => None,
            "n" => Some(PieceKind::Knight),
            "b" => Some(PieceKind::Bishop),
            "r" => Some(PieceKind::Rook),
            "q" => Some(PieceKind::Queen),
            _ => return None,
        };
        Some((from, to, promo))
    }

    /// Plays a move given in UCI notation, such as `e2e4`, or `e7e8q` for a
    /// promotion.
    ///
    /// # Errors
    ///
    /// Returns [`MoveError::GameOver`] if the game has ended,
    /// [`MoveError::InvalidNotation`] if `s` is not valid UCI notation, or
    /// [`MoveError::IllegalMove`] if the move is not legal in the current
    /// position.
    pub fn play_uci(&mut self, s: &str) -> Result<(), MoveError> {
        if self.result.is_some() {
            return Err(MoveError::GameOver);
        }
        let (from, to, promo) = Self::parse_uci(s).ok_or(MoveError::InvalidNotation)?;
        let mv = self
            .legal_moves()
            .iter()
            .find(|mv| mv.from() == from && mv.to() == to && mv.kind().promotion() == promo)
            .copied()
            .ok_or(MoveError::IllegalMove)?;

        self.play(mv)
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

    const fn finish(&mut self, result: GameResult) -> Result<(), MoveError> {
        if self.result.is_some() {
            return Err(MoveError::GameOver);
        }
        self.result = Some(result);
        Ok(())
    }

    /// Takes back the last move, restoring the position and result before it.
    /// Does nothing at the start of the game.
    pub fn undo(&mut self) {
        if let Some(previous) = self.history.pop() {
            self.board = previous;
            self.moves.pop();
            self.result = self.detect_result();
        }
    }
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}
