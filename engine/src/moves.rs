use crate::{piece::PieceKind, square::Square};
use std::fmt;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MoveKind {
    Quiet,
    DoublePush,
    KingCastle,
    QueenCastle,
    Capture,
    EnPassant,
    PromoN,
    PromoB,
    PromoR,
    PromoQ,
    CapturePromoN,
    CapturePromoB,
    CapturePromoR,
    CapturePromoQ,
}

macro_rules! move_kind_bits {
    ($($variant:ident = $bits:literal),+ $(,)?) => {
        impl MoveKind {
            #[must_use]
            pub const fn to_bits(self) -> u8 {
                match self {
                    $(Self::$variant => $bits),+
                }
            }

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
    PromoN = 6,
    PromoB = 7,
    PromoR = 8,
    PromoQ = 9,
    CapturePromoN = 10,
    CapturePromoB = 11,
    CapturePromoR = 12,
    CapturePromoQ = 13,
}

impl MoveKind {
    #[must_use]
    pub const fn is_capture(self) -> bool {
        matches!(
            self,
            Self::Capture
                | Self::EnPassant
                | Self::CapturePromoN
                | Self::CapturePromoB
                | Self::CapturePromoR
                | Self::CapturePromoQ
        )
    }

    #[must_use]
    pub const fn promotion(self) -> Option<PieceKind> {
        match self {
            Self::PromoN | Self::CapturePromoN => Some(PieceKind::Knight),
            Self::PromoB | Self::CapturePromoB => Some(PieceKind::Bishop),
            Self::PromoR | Self::CapturePromoR => Some(PieceKind::Rook),
            Self::PromoQ | Self::CapturePromoQ => Some(PieceKind::Queen),
            Self::Quiet
            | Self::DoublePush
            | Self::KingCastle
            | Self::QueenCastle
            | Self::Capture
            | Self::EnPassant => None,
        }
    }
}

/*
bits  0..=5   from square   (0..63, 6 bits)
bits  6..=11  to square     (0..63, 6 bits)
bits 12..=15  kind          (0..15, 4 bits)
*/
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

impl Move {
    const FROM_MASK: u16 = 0x3F;
    const TO_MASK: u16 = 0x3F;
    const KIND_MASK: u16 = 0xF;
    const TO_SHIFT: u16 = 6;
    const KIND_SHIFT: u16 = 12;

    const NULL: Self = Self(0);

    #[must_use]
    pub fn new(from: Square, to: Square, kind: MoveKind) -> Self {
        let from = u16::from(from.index()) & Self::FROM_MASK;
        let to = (u16::from(to.index()) & Self::TO_MASK) << Self::TO_SHIFT;
        let kind = (u16::from(kind.to_bits()) & Self::KIND_MASK) << Self::KIND_SHIFT;
        Self(from | to | kind)
    }

    #[must_use]
    pub fn from(self) -> Square {
        Square::from_index_masked(self.field(0, Self::FROM_MASK))
    }

    #[must_use]
    pub fn to(self) -> Square {
        Square::from_index_masked(self.field(Self::TO_SHIFT, Self::TO_MASK))
    }

    #[must_use]
    pub fn kind(self) -> MoveKind {
        MoveKind::from_bits(self.field(Self::KIND_SHIFT, Self::KIND_MASK))
            .unwrap_or(MoveKind::Quiet)
    }

    fn field(self, shift: u16, mask: u16) -> u8 {
        u8::try_from((self.0 >> shift) & mask).unwrap_or(0)
    }
}

#[derive(Clone)]
pub struct MoveList {
    moves: [Move; 256],
    len: usize,
}

impl MoveList {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            moves: [Move::NULL; 256],
            len: 0,
        }
    }

    pub fn push(&mut self, mv: Move) {
        if let Some(slot) = self.moves.get_mut(self.len) {
            *slot = mv;
            self.len = self.len.saturating_add(1);
        }
    }

    #[must_use]
    pub fn as_slice(&self) -> &[Move] {
        self.moves.get(..self.len).unwrap_or(&[])
    }

    #[must_use]
    pub const fn len(&self) -> usize {
        self.len
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Move> {
        self.as_slice().iter()
    }
}

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
