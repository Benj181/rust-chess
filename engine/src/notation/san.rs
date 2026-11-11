//! Standard algebraic notation (SAN) for moves, like `Nf3`, `exd5` or `O-O`.

use crate::{
    board::Board,
    types::{Move, MoveKind, Piece, PieceKind},
};

impl Board {
    /// `mv` in standard algebraic notation, as played from this position:
    /// `e4`, `Nbd2`, `exd5`, `e8=Q`, `O-O`, with `+` for check and `#` for
    /// checkmate.
    ///
    /// `mv` should be legal in this position.
    ///
    /// ```
    /// use engine::Board;
    ///
    /// let board = Board::start();
    /// let mv = board.parse_uci("g1f3").unwrap();
    /// assert_eq!(board.san(mv), "Nf3");
    /// ```
    #[must_use]
    pub fn san(&self, mv: Move) -> String {
        format!("{}{}", self.san_body(mv), self.san_suffix(mv))
    }

    fn san_body(&self, mv: Move) -> String {
        let kind = mv.kind();
        if kind == MoveKind::KingCastle {
            return "O-O".to_owned();
        }
        if kind == MoveKind::QueenCastle {
            return "O-O-O".to_owned();
        }
        let Some(piece) = self.piece_at(mv.from()) else {
            return mv.to_string();
        };

        if piece.kind() == PieceKind::Pawn {
            pawn_san(mv)
        } else {
            self.piece_san(mv, piece)
        }
    }

    fn piece_san(&self, mv: Move, piece: Piece) -> String {
        let capture = if mv.kind().is_capture() { "x" } else { "" };
        format!(
            "{}{}{capture}{}",
            upper_letter(piece.kind()),
            self.disambiguation(mv, piece),
            mv.to()
        )
    }

    fn disambiguation(&self, mv: Move, piece: Piece) -> String {
        let rivals: Vec<_> = self
            .legal_moves()
            .iter()
            .filter(|other| {
                other.to() == mv.to()
                    && other.from() != mv.from()
                    && self.piece_at(other.from()) == Some(piece)
            })
            .map(|other| other.from())
            .collect();

        let from = mv.from();
        if rivals.is_empty() {
            String::new()
        } else if rivals.iter().all(|sq| sq.file() != from.file()) {
            from.file_char().to_string()
        } else if rivals.iter().all(|sq| sq.rank() != from.rank()) {
            from.rank_char().to_string()
        } else {
            from.to_string()
        }
    }

    fn san_suffix(&self, mv: Move) -> &'static str {
        let after = self.make_move(mv);
        if !after.in_check() {
            return "";
        }
        if after.legal_moves().is_empty() {
            "#"
        } else {
            "+"
        }
    }
}

fn pawn_san(mv: Move) -> String {
    let kind = mv.kind();
    let mut san = String::new();
    if kind.is_capture() {
        san.push(mv.from().file_char());
        san.push('x');
    }
    san.push_str(&mv.to().to_string());
    if let Some(promoted) = kind.promotion() {
        san.push('=');
        san.push(upper_letter(promoted));
    }
    san
}

const fn upper_letter(kind: PieceKind) -> char {
    kind.letter().to_ascii_uppercase()
}
