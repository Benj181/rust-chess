//! Standard algebraic notation (SAN) for moves, like `Nf3`, `exd5` or `O-O`.

use crate::{
    board::Board,
    movegen::generate_legal,
    types::{Color, Move, MoveKind, MoveList, Piece, PieceKind},
};

impl Board {
    /// `mv` in standard algebraic notation, as played from this position:
    /// `e4`, `Nbd2`, `exd5`, `e8=Q`, `O-O`, with `+` for check and `#` for
    /// checkmate.
    ///
    /// `mv` should be legal in this position.
    ///
    /// ```
    /// use engine::{Board, Game};
    ///
    /// let mut game = Game::new();
    /// game.play_uci("g1f3").unwrap();
    /// let mv = game.last_move().unwrap();
    /// assert_eq!(Board::start().san(mv), "Nf3");
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

        let capture = if kind.is_capture() { "x" } else { "" };
        let promotion = kind
            .promotion()
            .map_or(String::new(), |promo| format!("={}", letter(promo)));

        if piece.kind() == PieceKind::Pawn {
            let file = if kind.is_capture() {
                mv.from().file_char().to_string()
            } else {
                String::new()
            };
            return format!("{file}{capture}{}{promotion}", mv.to());
        }

        format!(
            "{}{}{capture}{}",
            letter(piece.kind()),
            self.disambiguation(mv, piece),
            mv.to()
        )
    }

    fn disambiguation(&self, mv: Move, piece: Piece) -> String {
        let mut legal = MoveList::new();
        generate_legal(self, &mut legal);
        let rivals: Vec<_> = legal
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
        let mut replies = MoveList::new();
        generate_legal(&after, &mut replies);
        if replies.is_empty() { "#" } else { "+" }
    }
}

const fn letter(kind: PieceKind) -> char {
    Piece::new(Color::White, kind).to_char()
}
