//! How a game ends, and why a move can be rejected.

use crate::types::Color;
use std::fmt;

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

impl fmt::Display for WinReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Checkmate => "checkmate",
            Self::Resignation => "resignation",
            Self::Timeout => "timeout",
        })
    }
}

impl fmt::Display for DrawReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Stalemate => "stalemate",
            Self::ThreefoldRepetition => "threefold repetition",
            Self::FiftyMoveRule => "the fifty-move rule",
            Self::InsufficientMaterial => "insufficient material",
            Self::Agreement => "agreement",
        })
    }
}

/// Why [`Game::play`](crate::game::Game::play) rejected a move.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MoveError {
    /// The game has already ended.
    GameOver,
    /// The move is not legal in the current position.
    IllegalMove,
}
