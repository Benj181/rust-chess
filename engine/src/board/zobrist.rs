//! Zobrist hashing: a position squeezed into one `u64`.

use crate::{
    board::Board,
    types::{CastlingRights, Color, Piece, PieceKind, Square},
};

const SIDE_SLOT: u64 = 768;
const EN_PASSANT_SLOT: u64 = 769;
const CASTLING: [(CastlingRights, u64); 4] = [
    (CastlingRights::WHITE_KING, 833),
    (CastlingRights::WHITE_QUEEN, 834),
    (CastlingRights::BLACK_KING, 835),
    (CastlingRights::BLACK_QUEEN, 836),
];

const fn key(slot: u64) -> u64 {
    let mut x = slot.wrapping_add(1).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^ (x >> 31)
}

/// The number that stands for `piece` on `sq`. [`Board`] combines these
/// into a running hash as pieces are put down and removed.
pub fn piece_key(piece: Piece, sq: Square) -> u64 {
    let kind = match piece.kind() {
        PieceKind::Pawn => 0,
        PieceKind::Knight => 1,
        PieceKind::Bishop => 2,
        PieceKind::Rook => 3,
        PieceKind::Queen => 4,
        PieceKind::King => 5,
    };
    let piece: u64 = match piece.color() {
        Color::White => kind,
        Color::Black => kind | 8,
    };
    key((piece << 6) | u64::from(sq.index()))
}

impl Board {
    /// A 64-bit fingerprint of the position (a Zobrist hash): every fact
    /// that [`same_position`](Self::same_position) compares has its own
    /// random-looking number, and the hash is all of them combined with XOR.
    ///
    /// The piece part is kept up to date as pieces move, so this only has to
    /// add the side to move, the en passant square and the castling rights.
    ///
    /// The same position always gives the same hash, however it was reached.
    /// Two different positions almost never share one.
    ///
    /// ```
    /// use engine::Board;
    ///
    /// let play = |moves: &[&str]| {
    ///     moves.iter().fold(Board::start(), |board, uci| {
    ///         board.make_move(board.parse_uci(uci).unwrap())
    ///     })
    /// };
    ///
    /// let one = play(&["g1f3", "g8f6", "b1c3"]);
    /// let other = play(&["b1c3", "g8f6", "g1f3"]);
    /// assert_eq!(one.zobrist(), other.zobrist());
    /// assert_ne!(one.zobrist(), Board::start().zobrist());
    /// ```
    #[must_use]
    pub fn zobrist(&self) -> u64 {
        let mut hash = self.placement_hash;

        if self.side_to_move == Color::Black {
            hash ^= key(SIDE_SLOT);
        }
        if let Some(sq) = self.en_passant {
            hash ^= key(EN_PASSANT_SLOT.saturating_add(u64::from(sq.index())));
        }
        for (right, slot) in CASTLING {
            if self.castling.contains(right) {
                hash ^= key(slot);
            }
        }

        hash
    }
}
