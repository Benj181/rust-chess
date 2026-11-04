use std::fmt;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Color {
    White,
    Black,
}

impl Color {
    pub const ALL: [Self; 2] = [Self::White, Self::Black];

    #[must_use]
    pub const fn opposite(self) -> Self {
        match self {
            Self::White => Self::Black,
            Self::Black => Self::White,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PieceKind {
    Pawn,
    Rook,
    Knight,
    Bishop,
    Queen,
    King,
}

impl PieceKind {
    pub const ALL: [Self; 6] = [
        Self::Pawn,
        Self::Rook,
        Self::Knight,
        Self::Bishop,
        Self::Queen,
        Self::King,
    ];
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Piece {
    color: Color,
    kind: PieceKind,
}

impl Piece {
    #[must_use]
    pub const fn new(color: Color, kind: PieceKind) -> Self {
        Self { color, kind }
    }

    #[must_use]
    pub const fn from_char(fen_char: char) -> Option<Self> {
        let color = if fen_char.is_ascii_lowercase() {
            Color::Black
        } else {
            Color::White
        };
        let kind = match fen_char.to_ascii_lowercase() {
            'p' => PieceKind::Pawn,
            'r' => PieceKind::Rook,
            'n' => PieceKind::Knight,
            'b' => PieceKind::Bishop,
            'q' => PieceKind::Queen,
            'k' => PieceKind::King,
            _ => return None,
        };
        Some(Self { color, kind })
    }

    #[must_use]
    pub const fn to_char(self) -> char {
        let c = match self.kind {
            PieceKind::Pawn => 'p',
            PieceKind::Rook => 'r',
            PieceKind::Knight => 'n',
            PieceKind::Bishop => 'b',
            PieceKind::Queen => 'q',
            PieceKind::King => 'k',
        };
        match self.color {
            Color::White => c.to_ascii_uppercase(),
            Color::Black => c,
        }
    }

    #[must_use]
    pub const fn color(self) -> Color {
        self.color
    }

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
