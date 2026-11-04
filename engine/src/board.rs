use crate::{
    attacks::{bishop_attacks, king_attacks, knight_attacks, pawn_attacks, rook_attacks},
    bitboard::Bitboard,
    castling::{Castle, CastlingRights},
    moves::{Move, MoveKind},
    piece::{Color, Piece, PieceKind},
    square::Square,
};
use std::fmt;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Board {
    pawns: Bitboard,
    knights: Bitboard,
    bishops: Bitboard,
    rooks: Bitboard,
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
            knights: Bitboard::EMPTY,
            bishops: Bitboard::EMPTY,
            rooks: Bitboard::EMPTY,
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
        self.pieces_of(color, PieceKind::King).into_iter().next()
    }

    #[must_use]
    const fn pieces_mut(&mut self, kind: PieceKind) -> &mut Bitboard {
        match kind {
            PieceKind::Pawn => &mut self.pawns,
            PieceKind::Knight => &mut self.knights,
            PieceKind::Bishop => &mut self.bishops,
            PieceKind::Rook => &mut self.rooks,
            PieceKind::Queen => &mut self.queens,
            PieceKind::King => &mut self.kings,
        }
    }

    #[must_use]
    pub const fn pieces(&self, kind: PieceKind) -> Bitboard {
        match kind {
            PieceKind::Pawn => self.pawns,
            PieceKind::Knight => self.knights,
            PieceKind::Bishop => self.bishops,
            PieceKind::Rook => self.rooks,
            PieceKind::Queen => self.queens,
            PieceKind::King => self.kings,
        }
    }

    #[must_use]
    pub const fn occupied_by(&self, color: Color) -> Bitboard {
        match color {
            Color::White => self.white,
            Color::Black => self.black,
        }
    }

    #[must_use]
    pub fn pieces_of(&self, color: Color, kind: PieceKind) -> Bitboard {
        self.pieces(kind) & self.occupied_by(color)
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
            .find(|&color| self.occupied_by(color).contains(sq))?;
        let kind = PieceKind::ALL
            .into_iter()
            .find(|&kind| self.pieces(kind).contains(sq))?;

        Some(Piece::new(color, kind))
    }

    #[must_use]
    pub fn is_square_attacked(&self, sq: Square, by: Color) -> bool {
        let attackers = |kind| self.pieces_of(by, kind);
        let occupied = self.occupied();

        if !(knight_attacks(sq) & attackers(PieceKind::Knight)).is_empty() {
            return true;
        }
        if !(king_attacks(sq) & attackers(PieceKind::King)).is_empty() {
            return true;
        }
        if pawn_attacks(by, attackers(PieceKind::Pawn)).contains(sq) {
            return true;
        }
        let queens = attackers(PieceKind::Queen);
        if !(rook_attacks(sq, occupied) & (attackers(PieceKind::Rook) | queens)).is_empty() {
            return true;
        }
        if !(bishop_attacks(sq, occupied) & (attackers(PieceKind::Bishop) | queens)).is_empty() {
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

    fn revoke_castling(&mut self, sq: Square) {
        for castle in Castle::ALL {
            if sq == castle.king_from || sq == castle.rook_from {
                self.castling.remove(castle.right);
            }
        }
    }

    #[must_use]
    pub fn make_move(&self, mv: Move) -> Self {
        let mut next = self.clone();
        let us = self.side_to_move;
        let (from, to, kind) = (mv.from(), mv.to(), mv.kind());
        debug_assert!(
            self.piece_at(from).is_some(),
            "make_move: no piece on {from} for {mv:?}"
        );
        let Some(mover) = self.piece_at(from) else {
            return next;
        };
        debug_assert_eq!(
            mover.color(),
            us,
            "moving a piece that isn't the side to move's"
        );

        next.en_passant = None;

        match kind {
            MoveKind::Quiet => {
                next.move_piece(mover, from, to);
            }
            MoveKind::DoublePush => {
                next.move_piece(mover, from, to);
                next.en_passant = Square::new(from.index().midpoint(to.index()));
            }
            MoveKind::Capture => {
                next.capture_at(to);
                next.move_piece(mover, from, to);
            }
            MoveKind::EnPassant => {
                if let Some(captured) = Square::from_file_rank(to.file(), from.rank()) {
                    next.capture_at(captured);
                }
                next.move_piece(mover, from, to);
            }
            MoveKind::KingCastle | MoveKind::QueenCastle => {
                next.move_piece(mover, from, to);
                if let Some(castle) = Castle::ALL
                    .iter()
                    .find(|castle| castle.kind == kind && castle.king_from == from)
                {
                    next.move_piece(
                        Piece::new(us, PieceKind::Rook),
                        castle.rook_from,
                        castle.rook_to,
                    );
                }
            }
            MoveKind::PromoKnight
            | MoveKind::PromoBishop
            | MoveKind::PromoRook
            | MoveKind::PromoQueen => {
                next.promote(mover, from, to, kind.promotion());
            }
            MoveKind::CapturePromoKnight
            | MoveKind::CapturePromoBishop
            | MoveKind::CapturePromoRook
            | MoveKind::CapturePromoQueen => {
                next.capture_at(to);
                next.promote(mover, from, to, kind.promotion());
            }
        }

        next.revoke_castling(from);
        next.revoke_castling(to);
        next.halfmove_clock = if mover.kind() == PieceKind::Pawn || kind.is_capture() {
            0
        } else {
            self.halfmove_clock.saturating_add(1)
        };
        if us == Color::Black {
            next.fullmove_number = self.fullmove_number.saturating_add(1);
        }
        next.side_to_move = us.opposite();

        next
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
