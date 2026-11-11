//! [`Board`]: a complete chess position.

use crate::{
    attacks::{bishop_attacks, king_attacks, knight_attacks, pawn_attacks, rook_attacks},
    types::{Bitboard, Castle, CastlingRights, Color, Move, MoveKind, Piece, PieceKind, Square},
};
use std::fmt;

/// A chess position: piece placement plus everything else a FEN string
/// records, which is the side to move, castling rights, the en passant square
/// and both move clocks.
///
/// Pieces are stored as six bitboards by kind and two by color. A board never
/// changes from the outside: [`make_move`](Self::make_move) returns a new one.
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
    /// The FEN of the standard starting position.
    pub const START_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

    /// The standard starting position.
    #[must_use]
    pub fn start() -> Self {
        Self::from_fen(Self::START_FEN).unwrap_or_else(Self::empty)
    }

    /// Parses a position from FEN.
    ///
    /// The two move clocks may be left off, and then default to 0 and 1. Returns
    /// `None` for malformed input, such as the wrong number of ranks, a rank that
    /// does not fill eight files, an unknown piece letter or a field that does not
    /// parse.
    ///
    /// # Examples
    ///
    /// ```
    /// use engine::Board;
    ///
    /// assert!(Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").is_some());
    /// assert!(Board::from_fen("4k3/8/8/8/8/8/8/4K3 w - -").is_some());
    /// assert!(Board::from_fen("4k3/8/8/8 w - -").is_none());
    /// ```
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

    /// Formats the position as a FEN string, the inverse of
    /// [`from_fen`](Self::from_fen).
    ///
    /// ```
    /// use engine::Board;
    ///
    /// let fen = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1";
    /// assert_eq!(Board::from_fen(fen).unwrap().to_fen(), fen);
    /// ```
    #[must_use]
    pub fn to_fen(&self) -> String {
        let mut placement = String::new();
        for rank in (0..8).rev() {
            let mut empty = 0;
            for file in 0..8 {
                match Square::from_file_rank(file, rank).and_then(|sq| self.piece_at(sq)) {
                    Some(piece) => {
                        push_empty(&mut placement, empty);
                        empty = 0;
                        placement.push(piece.to_char());
                    }
                    None => empty = empty.saturating_add(1),
                }
            }
            push_empty(&mut placement, empty);
            if rank > 0 {
                placement.push('/');
            }
        }

        let side = match self.side_to_move {
            Color::White => 'w',
            Color::Black => 'b',
        };
        let en_passant = self
            .en_passant
            .map_or_else(|| "-".to_owned(), |sq| sq.to_string());
        format!(
            "{placement} {side} {} {en_passant} {} {}",
            self.castling.to_fen(),
            self.halfmove_clock,
            self.fullmove_number
        )
    }

    /// A board with no pieces, white to move, no castling rights and the clocks at 0 and 1.
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

    /// Every occupied square, of either color.
    #[must_use]
    pub const fn occupied(&self) -> Bitboard {
        self.white.union(self.black)
    }

    /// The color whose turn it is.
    #[must_use]
    pub const fn side_to_move(&self) -> Color {
        self.side_to_move
    }

    /// The castling rights both sides still have.
    #[must_use]
    pub const fn castling(&self) -> CastlingRights {
        self.castling
    }

    /// Half move clock.
    #[must_use]
    pub const fn halfmove_clock(&self) -> u16 {
        self.halfmove_clock
    }

    /// The square a pawn could capture onto en passant.
    ///
    /// It is set right after a double pawn push, even if no enemy pawn can
    /// actually make the capture.
    #[must_use]
    pub const fn en_passant(&self) -> Option<Square> {
        self.en_passant
    }

    /// The square of `color`'s king, or `None` if that side has no king.
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

    /// Every piece of `kind`, of both colors.
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

    /// Every square occupied by a piece of `color`.
    #[must_use]
    pub const fn occupied_by(&self, color: Color) -> Bitboard {
        match color {
            Color::White => self.white,
            Color::Black => self.black,
        }
    }

    /// The pieces of `kind` that belong to `color`.
    #[must_use]
    pub fn pieces_of(&self, color: Color, kind: PieceKind) -> Bitboard {
        self.pieces(kind) & self.occupied_by(color)
    }

    /// The total [`value`](PieceKind::value) of `color`'s pieces, in pawns.
    #[must_use]
    pub fn material(&self, color: Color) -> u32 {
        PieceKind::ALL
            .iter()
            .map(|&kind| kind.value().saturating_mul(self.pieces_of(color, kind).count()))
            .sum()
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

    /// The piece on `sq`, or `None` if it is empty.
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

    /// Whether any piece of color `by` attacks `sq`.
    ///
    /// Pieces of either color block sliding attacks. Whatever stands on `sq`
    /// itself does not matter.
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

    /// Plays `mv` and returns the resulting position, leaving `self` unchanged.
    ///
    /// Besides moving pieces, this updates castling rights, the en passant
    /// square, both clocks and the side to move.
    ///
    /// The move is trusted, not checked for legality, so pass moves from
    /// [`generate_legal`](crate::movegen::generate_legal). In debug builds a move
    /// from an empty square, or of the wrong color, panics.
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

    /// Whether neither side has enough material to checkmate: only kings, plus
    /// at most one knight or bishop, or any number of bishops all on squares
    /// of the same color.
    #[must_use]
    pub fn insufficient_material(&self) -> bool {
        if !(self.pawns | self.rooks | self.queens).is_empty() {
            return false;
        }
        if (self.knights | self.bishops).into_iter().count() <= 1 {
            return true;
        }
        if !self.knights.is_empty() {
            return false;
        }
        let mut colors = self
            .bishops
            .into_iter()
            .map(|sq| (sq.file() ^ sq.rank()) & 1);
        let first = colors.next();
        colors.all(|color| Some(color) == first)
    }

    /// Whether `self` and `other` are the same position for repetition: the
    /// same pieces, side to move, castling rights and en passant square. The
    /// move clocks are ignored.
    #[must_use]
    pub fn same_position(&self, other: &Self) -> bool {
        self.pawns == other.pawns
            && self.knights == other.knights
            && self.bishops == other.bishops
            && self.rooks == other.rooks
            && self.queens == other.queens
            && self.kings == other.kings
            && self.white == other.white
            && self.black == other.black
            && self.side_to_move == other.side_to_move
            && self.castling == other.castling
            && self.en_passant == other.en_passant
    }

    /// Whether the side to move is in check.
    #[must_use]
    pub fn in_check(&self) -> bool {
        let me = self.side_to_move();
        self.king_square(me)
            .is_some_and(|king| self.is_square_attacked(king, me.opposite()))
    }
}

fn push_empty(fen: &mut String, count: u32) {
    if count > 0
        && let Some(digit) = char::from_digit(count, 10)
    {
        fen.push(digit);
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
