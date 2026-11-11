//! FEN, the one-line text format for a position.
use crate::{
    board::Board,
    types::{CastlingRights, Color, Piece, Square},
};

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

        board.castling = parse_castling(fields.next()?)?;

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
            format_castling(self.castling),
            self.halfmove_clock,
            self.fullmove_number
        )
    }
}

/// Parses the castling field of a FEN string: `-`, or any combination of
/// `K`, `Q`, `k` and `q`. Returns `None` for any other character.
fn parse_castling(s: &str) -> Option<CastlingRights> {
    let mut rights = CastlingRights::NONE;
    if s == "-" {
        return Some(rights);
    }
    for c in s.chars() {
        let right = match c {
            'K' => CastlingRights::WHITE_KING,
            'Q' => CastlingRights::WHITE_QUEEN,
            'k' => CastlingRights::BLACK_KING,
            'q' => CastlingRights::BLACK_QUEEN,
            _ => return None,
        };
        rights.insert(right);
    }
    Some(rights)
}

/// Formats `rights` as the castling field of a FEN string, in the order
/// `KQkq`, or `-` if no side may castle.
fn format_castling(rights: CastlingRights) -> String {
    let field: String = [
        (CastlingRights::WHITE_KING, 'K'),
        (CastlingRights::WHITE_QUEEN, 'Q'),
        (CastlingRights::BLACK_KING, 'k'),
        (CastlingRights::BLACK_QUEEN, 'q'),
    ]
    .into_iter()
    .filter(|&(right, _)| rights.contains(right))
    .map(|(_, symbol)| symbol)
    .collect();
    if field.is_empty() {
        "-".to_owned()
    } else {
        field
    }
}

fn push_empty(fen: &mut String, count: u32) {
    if count > 0
        && let Some(digit) = char::from_digit(count, 10)
    {
        fen.push(digit);
    }
}
