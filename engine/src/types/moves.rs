//! Moves: [`MoveKind`], the compact [`Move`] and the fixed-capacity [`MoveList`].

use crate::types::{PieceKind, Square};
use std::fmt;

const MAX_MOVES: usize = 256;

/// What kind of move a [`Move`] is.
///
/// The kind tells [`Board::make_move`](crate::board::Board::make_move) about
/// everything beyond moving one piece: captures, the en passant square, the
/// castling rook and promotions.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MoveKind {
    Quiet,
    DoublePush,
    KingCastle,
    QueenCastle,
    Capture,
    EnPassant,
    PromoKnight,
    PromoBishop,
    PromoRook,
    PromoQueen,
    CapturePromoKnight,
    CapturePromoBishop,
    CapturePromoRook,
    CapturePromoQueen,
}

macro_rules! move_kind_bits {
    ($($variant:ident = $bits:literal),+ $(,)?) => {
        impl MoveKind {
            /// The 4-bit code stored inside a [`Move`].
            #[must_use]
            pub const fn to_bits(self) -> u8 {
                match self {
                    $(Self::$variant => $bits),+
                }
            }

            /// The kind for a 4-bit code, or `None` if no kind uses that code.
            #[must_use]
            pub const fn from_bits(bits: u8) -> Option<Self> {
                match bits {
                    $($bits => Some(Self::$variant),)+
                    _ => None,
                }
            }
        }
    };
}

move_kind_bits! {
    Quiet = 0,
    DoublePush = 1,
    KingCastle = 2,
    QueenCastle = 3,
    Capture = 4,
    EnPassant = 5,
    PromoKnight = 6,
    PromoBishop = 7,
    PromoRook = 8,
    PromoQueen = 9,
    CapturePromoKnight = 10,
    CapturePromoBishop = 11,
    CapturePromoRook = 12,
    CapturePromoQueen = 13,
}

impl MoveKind {
    /// Whether the move captures a piece, en passant included.
    #[must_use]
    pub const fn is_capture(self) -> bool {
        matches!(
            self,
            Self::Capture
                | Self::EnPassant
                | Self::CapturePromoKnight
                | Self::CapturePromoBishop
                | Self::CapturePromoRook
                | Self::CapturePromoQueen
        )
    }

    /// The piece a pawn promotes to, or `None` if the move is not a promotion.
    #[must_use]
    pub const fn promotion(self) -> Option<PieceKind> {
        match self {
            Self::PromoKnight | Self::CapturePromoKnight => Some(PieceKind::Knight),
            Self::PromoBishop | Self::CapturePromoBishop => Some(PieceKind::Bishop),
            Self::PromoRook | Self::CapturePromoRook => Some(PieceKind::Rook),
            Self::PromoQueen | Self::CapturePromoQueen => Some(PieceKind::Queen),
            Self::Quiet
            | Self::DoublePush
            | Self::KingCastle
            | Self::QueenCastle
            | Self::Capture
            | Self::EnPassant => None,
        }
    }
}

/// A move packed into 16 bits.
///
/// | Bits  | Field         |
/// |-------|---------------|
/// | 0–5   | from square   |
/// | 6–11  | to square     |
/// | 12–15 | [`MoveKind`]  |
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Move(u16);

impl fmt::Debug for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Move")
            .field("from", &format_args!("{}", self.from()))
            .field("to", &format_args!("{}", self.to()))
            .field("kind", &self.kind())
            .finish()
    }
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.from(), self.to())?;
        if let Some(kind) = self.kind().promotion() {
            write!(f, "{}", kind.letter())?;
        }
        Ok(())
    }
}

impl Move {
    const FROM_MASK: u16 = 0x3F;
    const TO_MASK: u16 = 0x3F;
    const KIND_MASK: u16 = 0xF;
    const TO_SHIFT: u16 = 6;
    const KIND_SHIFT: u16 = 12;

    const NULL: Self = Self(0);

    /// Packs a move of `kind` from `from` to `to`.
    #[must_use]
    pub fn new(from: Square, to: Square, kind: MoveKind) -> Self {
        let from = u16::from(from.index()) & Self::FROM_MASK;
        let to = (u16::from(to.index()) & Self::TO_MASK) << Self::TO_SHIFT;
        let kind = (u16::from(kind.to_bits()) & Self::KIND_MASK) << Self::KIND_SHIFT;
        Self(from | to | kind)
    }

    /// The square the piece moves from.
    #[must_use]
    pub fn from(self) -> Square {
        Square::from_index_masked(self.field(0, Self::FROM_MASK))
    }

    /// The square the piece moves to.
    #[must_use]
    pub fn to(self) -> Square {
        Square::from_index_masked(self.field(Self::TO_SHIFT, Self::TO_MASK))
    }

    /// The kind of move.
    #[must_use]
    pub fn kind(self) -> MoveKind {
        MoveKind::from_bits(self.field(Self::KIND_SHIFT, Self::KIND_MASK))
            .unwrap_or(MoveKind::Quiet)
    }

    fn field(self, shift: u16, mask: u16) -> u8 {
        u8::try_from((self.0 >> shift) & mask).unwrap_or(0)
    }
}

/// A list of moves stored inline, so generating moves never allocates.
///
/// It holds up to 256 moves, more than the 218 of any legal position. A move
/// pushed past that is dropped.
#[derive(Clone)]
pub struct MoveList {
    moves: [Move; MAX_MOVES],
    len: usize,
}

impl MoveList {
    /// An empty list.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            moves: [Move::NULL; MAX_MOVES],
            len: 0,
        }
    }

    /// Appends `mv`, or drops it if the list is full.
    pub fn push(&mut self, mv: Move) {
        if let Some(slot) = self.moves.get_mut(self.len) {
            *slot = mv;
            self.len = self.len.saturating_add(1);
        }
    }

    /// The moves pushed so far, in order.
    #[must_use]
    pub fn as_slice(&self) -> &[Move] {
        self.moves.get(..self.len).unwrap_or(&[])
    }

    /// Sorts the moves so that the smallest `key` comes first. Moves with
    /// equal keys end up in no particular order.
    pub fn sort_by_key<K: Ord>(&mut self, key: impl FnMut(&Move) -> K) {
        if let Some(moves) = self.moves.get_mut(..self.len) {
            moves.sort_unstable_by_key(key);
        }
    }

    /// Swaps the moves at positions `a` and `b`. Does nothing if either is
    /// past the end of the list.
    pub const fn swap(&mut self, a: usize, b: usize) {
        if a < self.len && b < self.len {
            self.moves.swap(a, b);
        }
    }

    /// The number of moves in the list.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Whether the list has no moves.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Iterates over the moves in the order they were pushed.
    pub fn iter(&self) -> std::slice::Iter<'_, Move> {
        self.as_slice().iter()
    }

    /// Checks if the list contains a move.
    #[must_use]
    pub fn contains(&self, mv: Move) -> bool {
        self.as_slice().contains(&mv)
    }

    /// The move from `from` to `to` that promotes to `promotion`, or `None`
    /// if the list has no such move. Pass `None` as `promotion` for a move
    /// that is not a promotion.
    #[must_use]
    pub fn find(&self, from: Square, to: Square, promotion: Option<PieceKind>) -> Option<Move> {
        self.iter()
            .find(|mv| mv.from() == from && mv.to() == to && mv.kind().promotion() == promotion)
            .copied()
    }
}

impl PartialEq for MoveList {
    fn eq(&self, other: &Self) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl Eq for MoveList {}

impl Default for MoveList {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for MoveList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.as_slice()).finish()
    }
}

impl<'a> IntoIterator for &'a MoveList {
    type Item = &'a Move;
    type IntoIter = std::slice::Iter<'a, Move>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
