//! A chess engine built on bitboards.
//!
//! A position is a [`board::Board`], usually created with
//! [`board::Board::from_fen`]. [`movegen::generate_legal`] lists the moves
//! in a position, and [`board::Board::make_move`] plays one, returning the
//! new position.
//!
//! ```
//! use engine::{board::Board, movegen::generate_legal, moves::MoveList};
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

mod attacks;
pub mod bitboard;
pub mod board;
pub mod castling;
mod direction;
pub mod game;
pub mod movegen;
pub mod moves;
pub mod piece;
mod san;
pub mod square;
