//! UCI move notation, like `e2e4` or `e7e8q`.

use crate::{
    board::Board,
    types::{Move, PieceKind, Square},
};

impl Board {
    /// Finds the legal move written in UCI notation, like `e2e4`, or `e7e8q`
    /// for a promotion. Returns `None` if `s` is not valid UCI or the move is
    /// not legal in this position.
    ///
    /// ```
    /// use engine::Board;
    ///
    /// let board = Board::start();
    /// assert_eq!(board.parse_uci("e2e4").unwrap().to_string(), "e2e4");
    /// assert!(board.parse_uci("e2e5").is_none());
    /// assert!(board.parse_uci("e2").is_none());
    /// ```
    #[must_use]
    pub fn parse_uci(&self, s: &str) -> Option<Move> {
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
        self.legal_moves()
            .iter()
            .find(|mv| mv.from() == from && mv.to() == to && mv.kind().promotion() == promo)
            .copied()
    }
}
