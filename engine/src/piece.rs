//! Colors, piece kinds and pieces.

use std::fmt;

/// One of the two sides.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Color {
    White,
    Black,
}

impl Color {
    /// Both colors, white first.
    pub const ALL: [Self; 2] = [Self::White, Self::Black];

    /// The other color.
    #[must_use]
    pub const fn opposite(self) -> Self {
        match self {
            Self::White => Self::Black,
            Self::Black => Self::White,
        }
    }
}

/// The six kinds of chess piece, ordered by value with the king last.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PieceKind {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

impl PieceKind {
    /// All six kinds, in declaration order.
    pub const ALL: [Self; 6] = [
        Self::Pawn,
        Self::Knight,
        Self::Bishop,
        Self::Rook,
        Self::Queen,
        Self::King,
    ];
}

/// A piece: a kind together with the color it belongs to.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Piece {
    color: Color,
    kind: PieceKind,
}

impl Piece {
    /// A piece of `kind` belonging to `color`.
    #[must_use]
    pub const fn new(color: Color, kind: PieceKind) -> Self {
        Self { color, kind }
    }

    /// Parses a FEN piece letter: uppercase for white and lowercase for black,
    /// like `N` for a white knight. Returns `None` for any other character.
    #[must_use]
    pub const fn from_char(fen_char: char) -> Option<Self> {
        let color = if fen_char.is_ascii_lowercase() {
            Color::Black
        } else {
            Color::White
        };
        let kind = match fen_char.to_ascii_lowercase() {
            'p' => PieceKind::Pawn,
            'n' => PieceKind::Knight,
            'b' => PieceKind::Bishop,
            'r' => PieceKind::Rook,
            'q' => PieceKind::Queen,
            'k' => PieceKind::King,
            _ => return None,
        };
        Some(Self { color, kind })
    }

    /// The FEN letter for this piece, the inverse of [`from_char`](Self::from_char).
    #[must_use]
    pub const fn to_char(self) -> char {
        let c = match self.kind {
            PieceKind::Pawn => 'p',
            PieceKind::Knight => 'n',
            PieceKind::Bishop => 'b',
            PieceKind::Rook => 'r',
            PieceKind::Queen => 'q',
            PieceKind::King => 'k',
        };
        match self.color {
            Color::White => c.to_ascii_uppercase(),
            Color::Black => c,
        }
    }

    /// The color this piece belongs to.
    #[must_use]
    pub const fn color(self) -> Color {
        self.color
    }

    /// What kind of piece this is.
    #[must_use]
    pub const fn kind(self) -> PieceKind {
        self.kind
    }
}

impl fmt::Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_char())
    }
}
