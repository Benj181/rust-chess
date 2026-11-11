//! UCI move notation, like `e2e4` or `e7e8q`.

use crate::types::{PieceKind, Square};

/// Parses a UCI move like `e2e4` or `e7e8q` into its from square, to square
/// and promotion piece. Returns `None` if `s` is not valid UCI.
pub fn parse_uci(s: &str) -> Option<(Square, Square, Option<PieceKind>)> {
    let from = Square::from_algebraic(s.get(..2)?)?;
    let to = Square::from_algebraic(s.get(2..4)?)?;
    let promo = match s.get(4..)? {
        "" => None,
        "n" => Some(PieceKind::Knight),
        "b" => Some(PieceKind::Bishop),
        "r" => Some(PieceKind::Rook),
        "q" => Some(PieceKind::Queen),
        _ => return None,
    };
    Some((from, to, promo))
}
