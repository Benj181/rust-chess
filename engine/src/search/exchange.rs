//! Static exchange evaluation: who comes out ahead if both sides keep
//! capturing on one square, without searching any moves.

use crate::{
    board::{
        Board,
        attacks::{bishop_attacks, king_attacks, knight_attacks, pawn_attacks, rook_attacks},
    },
    search::{eval::material_value, ordering::material_gain},
    types::{Bitboard, Color, Move, MoveKind, Piece, PieceKind, Square},
};

const KING_VALUE: i32 = 10_000;
const MAX_CAPTURES: usize = 32;

const fn exchange_value(kind: PieceKind) -> i32 {
    match kind {
        PieceKind::King => KING_VALUE,
        _ => material_value(kind),
    }
}

fn attackers_of(board: &Board, sq: Square, occupied: Bitboard) -> Bitboard {
    let target = Bitboard::from_square(sq);
    let pawns = |color| board.pieces_of(color, PieceKind::Pawn);
    let queens = board.pieces(PieceKind::Queen);

    let white_pawns = pawn_attacks(Color::Black, target) & pawns(Color::White);
    let black_pawns = pawn_attacks(Color::White, target) & pawns(Color::Black);
    let knights = knight_attacks(sq) & board.pieces(PieceKind::Knight);
    let kings = king_attacks(sq) & board.pieces(PieceKind::King);
    let diagonal = bishop_attacks(sq, occupied) & (board.pieces(PieceKind::Bishop) | queens);
    let straight = rook_attacks(sq, occupied) & (board.pieces(PieceKind::Rook) | queens);

    (white_pawns | black_pawns | knights | kings | diagonal | straight) & occupied
}

fn cheapest(board: &Board, attackers: Bitboard, side: Color) -> Option<(Square, PieceKind)> {
    PieceKind::ALL.into_iter().find_map(|kind| {
        let sq = (attackers & board.pieces_of(side, kind))
            .into_iter()
            .next()?;
        Some((sq, kind))
    })
}

/// The material the capture `mv` wins or loses once every sensible
/// recapture on its target square has been made, in centipawns. Each side
/// recaptures with its cheapest piece and stops when going on would lose
/// material.
///
/// A queen taking a defended pawn scores `100 - 900`; a pawn taking a
/// defended queen scores `900 - 100`.
fn exchange(board: &Board, mv: Move) -> i32 {
    let to = mv.to();
    let mut occupied = board.occupied();
    occupied.clear(mv.from());
    if mv.kind() == MoveKind::EnPassant
        && let Some(taken) = Square::from_file_rank(to.file(), mv.from().rank())
    {
        occupied.clear(taken);
    }

    let mut gains = [0; MAX_CAPTURES];
    let mut count: usize = 1;
    if let Some(first) = gains.first_mut() {
        *first = material_gain(board, mv);
    }
    let mut standing = mv
        .kind()
        .promotion()
        .or_else(|| board.piece_at(mv.from()).map(Piece::kind))
        .map_or(0, exchange_value);
    let mut side = board.side_to_move().opposite();

    while let Some((from, kind)) = cheapest(board, attackers_of(board, to, occupied), side)
        && let Some(previous) = count.checked_sub(1).and_then(|i| gains.get(i)).copied()
        && let Some(slot) = gains.get_mut(count)
    {
        *slot = standing.saturating_sub(previous);
        count = count.saturating_add(1);
        standing = exchange_value(kind);
        occupied.clear(from);
        side = side.opposite();
    }

    let mut result = 0;
    for (index, &gain) in gains.iter().enumerate().take(count).rev() {
        result = if index.saturating_add(1) == count {
            gain
        } else {
            gain.min(result.saturating_neg())
        };
    }
    result
}

/// Whether the capture `mv` loses material once the recaptures are counted.
///
/// Taking something worth at least as much as the capturing piece can never
/// lose, so the exchange is only played out for the other captures.
pub fn loses_material(board: &Board, mv: Move) -> bool {
    let attacker = board
        .piece_at(mv.from())
        .map_or(0, |piece| exchange_value(piece.kind()));
    material_gain(board, mv) < attacker && exchange(board, mv) < 0
}
