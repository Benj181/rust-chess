use crate::bitboard::Bitboard;
use crate::castling::CastlingRights;
use crate::piece::{Color, Piece, PieceKind};
use crate::square::Square;
use std::fmt;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Board {
    pawns: Bitboard,
    rooks: Bitboard,
    knights: Bitboard,
    bishops: Bitboard,
    queens: Bitboard,
    kings: Bitboard,
    white: Bitboard,
    black: Bitboard,
    side_to_move: Color,
    castling: CastlingRights,
    en_passant: Option<Square>,
    halfmove_clock: u16,
    fullmove_number: u16,
}

impl Board {
    #[must_use]
    pub fn from_fen(fen: &str) -> Option<Self> {
        let mut board = Self::empty();
        let mut fields = fen.split_whitespace();
        let placement = fields.next()?;
        if placement.split('/').count() != 8 {
            return None;
        }
        for (chunk, rank) in placement.split('/').zip((0..8).rev()) {
            let mut file: u8 = 0;
            for c in chunk.chars() {
                if let Some(incr) = c.to_digit(10) {
                    if !(1..=8).contains(&incr) {
                        return None;
                    }
                    file = file.checked_add(u8::try_from(incr).ok()?)?;
                } else {
                    let piece = Piece::from_char(c)?;
                    let sq = Square::from_file_rank(file, rank)?;
                    board.put(piece, sq);
                    file = file.checked_add(1)?;
                }
            }
            if file != 8 {
                return None;
            }
        }
        let stm = match fields.next()? {
            "w" => Color::White,
            "b" => Color::Black,
            _ => return None,
        };
        board.side_to_move = stm;

        board.castling = CastlingRights::from_fen(fields.next()?)?;

        board.en_passant = match fields.next()? {
            "-" => None,
            pos => Some(Square::from_algebraic(pos)?),
        };

        board.halfmove_clock = fields.next().map_or(Some(0), |s| s.parse().ok())?;
        board.fullmove_number = fields.next().map_or(Some(1), |s| s.parse().ok())?;

        Some(board)
    }

    #[must_use]
    pub const fn empty() -> Self {
        Self {
            pawns: Bitboard::EMPTY,
            rooks: Bitboard::EMPTY,
            knights: Bitboard::EMPTY,
            bishops: Bitboard::EMPTY,
            queens: Bitboard::EMPTY,
            kings: Bitboard::EMPTY,
            white: Bitboard::EMPTY,
            black: Bitboard::EMPTY,
            side_to_move: Color::White,
            castling: CastlingRights::NONE,
            en_passant: None,
            halfmove_clock: 0,
            fullmove_number: 1,
        }
    }

    #[must_use]
    const fn pieces_mut(&mut self, kind: PieceKind) -> &mut Bitboard {
        match kind {
            PieceKind::Pawn => &mut self.pawns,
            PieceKind::Rook => &mut self.rooks,
            PieceKind::Knight => &mut self.knights,
            PieceKind::Bishop => &mut self.bishops,
            PieceKind::Queen => &mut self.queens,
            PieceKind::King => &mut self.kings,
        }
    }

    #[must_use]
    pub const fn pieces(&self, kind: PieceKind) -> Bitboard {
        match kind {
            PieceKind::Pawn => self.pawns,
            PieceKind::Rook => self.rooks,
            PieceKind::Knight => self.knights,
            PieceKind::Bishop => self.bishops,
            PieceKind::Queen => self.queens,
            PieceKind::King => self.kings,
        }
    }

    #[must_use]
    pub const fn color_pieces(&self, color: Color) -> Bitboard {
        match color {
            Color::White => self.white,
            Color::Black => self.black,
        }
    }

    pub const fn put(&mut self, piece: Piece, sq: Square) {
        self.pieces_mut(piece.kind()).set(sq);
        match piece.color() {
            Color::White => self.white.set(sq),
            Color::Black => self.black.set(sq),
        }
    }

    #[must_use]
    pub fn piece_at(&self, sq: Square) -> Option<Piece> {
        let color = Color::ALL
            .into_iter()
            .find(|&color| self.color_pieces(color).contains(sq))?;
        let kind = PieceKind::ALL
            .into_iter()
            .find(|&kind| self.pieces(kind).contains(sq))?;

        Some(Piece::new(color, kind))
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (rank, label) in (0..8).rev().zip("87654321".chars()) {
            write!(f, "{label}")?;
            for file in 0..8 {
                if let Some(sq) = Square::from_file_rank(file, rank) {
                    let symbol = self.piece_at(sq).map_or('.', Piece::to_char);
                    write!(f, " {symbol}")?;
                }
            }
            writeln!(f)?;
        }
        write!(f, "  a b c d e f g h")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn testing() {
        let board =
            Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
        println!("{board}");
        println!("{}", board.pieces(PieceKind::Knight));
        assert!(
            board
                .knights
                .contains(Square::from_algebraic("b1").unwrap())
        );
    }
}
