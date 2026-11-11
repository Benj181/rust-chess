//! A chess engine built on bitboards.
//!
//! A position is a [` Board`], usually created with
//! [`Board::from_fen`]. [`movegen::generate_legal`] lists the moves
//! in a position, and [`board::Board::make_move`] plays one, returning the
//! new position.
//!
//! ```
//! use engine::{movegen::generate_legal, Board, MoveList};
//!
//! let board = Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
//! let mut moves = MoveList::new();
//! generate_legal(&board, &mut moves);
//! assert_eq!(moves.len(), 20);
//!
//! let first = *moves.iter().next().unwrap();
//! let after = board.make_move(first);
//! assert_ne!(after.side_to_move(), board.side_to_move());
//! ```

mod board;
mod eval;
mod game;
pub mod movegen;
mod san;
pub mod search;
mod types;

pub use board::Board;
pub use game::{DrawReason, Game, GameResult, MoveError, WinReason};
pub use types::{
    Bitboard, CastlingRights, Color, Move, MoveKind, MoveList, Piece, PieceKind, Square,
};
