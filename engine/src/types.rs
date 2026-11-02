pub enum Color {
    White,
    Black,
}

impl Color {
    pub const fn opposite(&self) -> Self {
        match &self {
            Self::White => Self::Black,
            Self::Black => Self::White,
        }
    }
}

pub enum PieceKind {
    Pawn,
    Rook,
    Knight,
    Bishop,
    Queen,
    King,
}

pub struct Piece {
    color: Color,
    kind: PieceKind,
}

impl Piece {
    pub fn to_fen(&self) -> char {
        'K'
    }

    pub fn from_fen(fen_char: char) -> Self {
        if fen_char {
            match char {}
        }
    }
}
