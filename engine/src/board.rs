use crate::attacks::{
    KING_ATTACKS, KNIGHT_ATTACKS, bishop_attacks, pawn_attacks, queen_attacks, rook_attacks,
};
use crate::bitboard::Bitboard;
use crate::castling::CastlingRights;
use crate::moves::{Move, MoveKind};
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
    pub const fn occupied(&self) -> Bitboard {
        self.white.union(self.black)
    }

    #[must_use]
    pub const fn side_to_move(&self) -> Color {
        self.side_to_move
    }

    #[must_use]
    pub const fn castling(&self) -> CastlingRights {
        self.castling
    }

    #[must_use]
    pub const fn en_passant(&self) -> Option<Square> {
        self.en_passant
    }

    #[must_use]
    pub fn king_square(&self, color: Color) -> Option<Square> {
        (self.kings & self.color_pieces(color)).into_iter().next()
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

    pub(crate) const fn put(&mut self, piece: Piece, sq: Square) {
        self.pieces_mut(piece.kind()).set(sq);
        match piece.color() {
            Color::White => self.white.set(sq),
            Color::Black => self.black.set(sq),
        }
    }

    pub(crate) const fn remove(&mut self, piece: Piece, sq: Square) {
        self.pieces_mut(piece.kind()).clear(sq);
        match piece.color() {
            Color::White => self.white.clear(sq),
            Color::Black => self.black.clear(sq),
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

    #[must_use]
    pub fn is_square_attacked(&self, sq: Square, by: Color) -> bool {
        let theirs = |kind| self.pieces(kind) & self.color_pieces(by);
        let occupied = self.occupied();

        let knight_squares = KNIGHT_ATTACKS
            .get(sq.as_usize())
            .copied()
            .unwrap_or(Bitboard::EMPTY);

        let king_squares = KING_ATTACKS
            .get(sq.as_usize())
            .copied()
            .unwrap_or(Bitboard::EMPTY);

        if !(knight_squares & theirs(PieceKind::Knight)).is_empty() {
            return true;
        }
        if !(king_squares & theirs(PieceKind::King)).is_empty() {
            return true;
        }
        if pawn_attacks(by, theirs(PieceKind::Pawn)).contains(sq) {
            return true;
        }
        if !(rook_attacks(sq, occupied) & theirs(PieceKind::Rook)).is_empty() {
            return true;
        }
        if !(bishop_attacks(sq, occupied) & theirs(PieceKind::Bishop)).is_empty() {
            return true;
        }
        if !(queen_attacks(sq, occupied) & theirs(PieceKind::Queen)).is_empty() {
            return true;
        }
        false
    }

    fn capture_at(&mut self, sq: Square) {
        if let Some(taken) = self.piece_at(sq) {
            self.remove(taken, sq);
        }
    }

    const fn move_piece(&mut self, piece: Piece, from: Square, to: Square) {
        self.remove(piece, from);
        self.put(piece, to);
    }

    const fn promote(
        &mut self,
        pawn: Piece,
        from: Square,
        to: Square,
        promoted: Option<PieceKind>,
    ) {
        self.remove(pawn, from);
        if let Some(kind) = promoted {
            self.put(Piece::new(pawn.color(), kind), to);
        }
    }

    fn move_rook(&mut self, color: Color, rank: u8, from_file: u8, to_file: u8) {
        if let (Some(from), Some(to)) = (
            Square::from_file_rank(from_file, rank),
            Square::from_file_rank(to_file, rank),
        ) {
            self.move_piece(Piece::new(color, PieceKind::Rook), from, to);
        }
    }

    const fn revoke_castling(&mut self, sq: Square) {
        match sq.index() {
            0 => self.castling.remove(CastlingRights::WHITE_QUEEN),
            4 => {
                self.castling.remove(CastlingRights::WHITE_KING);
                self.castling.remove(CastlingRights::WHITE_QUEEN);
            }
            7 => self.castling.remove(CastlingRights::WHITE_KING),
            56 => self.castling.remove(CastlingRights::BLACK_QUEEN),
            60 => {
                self.castling.remove(CastlingRights::BLACK_KING);
                self.castling.remove(CastlingRights::BLACK_QUEEN);
            }
            63 => self.castling.remove(CastlingRights::BLACK_KING),
            _ => {}
        }
    }

    #[must_use]
    pub fn make_move(&self, mv: Move) -> Self {
        let mut b = self.clone();
        let us = self.side_to_move;
        let (from, to, kind) = (mv.from(), mv.to(), mv.kind());
        let Some(mover) = self.piece_at(from) else {
            return b;
        };
        debug_assert_eq!(
            mover.color(),
            us,
            "moving a piece that isn't the side to move's"
        );

        b.en_passant = None;

        match kind {
            MoveKind::Quiet => {
                b.move_piece(mover, from, to);
            }
            MoveKind::DoublePush => {
                b.move_piece(mover, from, to);
                b.en_passant = Square::new(from.index().midpoint(to.index()));
            }
            MoveKind::Capture => {
                b.capture_at(to);
                b.move_piece(mover, from, to);
            }
            MoveKind::EnPassant => {
                if let Some(captured) = Square::from_file_rank(to.file(), from.rank()) {
                    b.capture_at(captured);
                }
                b.move_piece(mover, from, to);
            }
            MoveKind::KingCastle => {
                b.move_piece(mover, from, to);
                b.move_rook(us, from.rank(), 7, 5);
            }
            MoveKind::QueenCastle => {
                b.move_piece(mover, from, to);
                b.move_rook(us, from.rank(), 0, 3);
            }
            MoveKind::PromoN | MoveKind::PromoB | MoveKind::PromoR | MoveKind::PromoQ => {
                b.promote(mover, from, to, kind.promotion());
            }
            MoveKind::CapturePromoN
            | MoveKind::CapturePromoB
            | MoveKind::CapturePromoR
            | MoveKind::CapturePromoQ => {
                b.capture_at(to);
                b.promote(mover, from, to, kind.promotion());
            }
        }

        b.revoke_castling(from);
        b.revoke_castling(to);
        b.halfmove_clock = if mover.kind() == PieceKind::Pawn || kind.is_capture() {
            0
        } else {
            self.halfmove_clock.saturating_add(1)
        };
        if us == Color::Black {
            b.fullmove_number = self.fullmove_number.saturating_add(1);
        }
        b.side_to_move = us.opposite();

        b
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
