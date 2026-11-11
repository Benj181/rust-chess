//! Move generation: produces the moves available in a position.
//!
//! Generation runs in two stages. [`generate_pseudo_legal`] produces every
//! move that follows the movement rules, and [`generate_legal`] keeps only
//! those that do not leave the mover's own king attacked.

use crate::{
    board::{
        Board,
        attacks::{
            bishop_attacks, king_attacks, knight_attacks, pawn_attacks, queen_attacks, rook_attacks,
        },
        direction::Direction,
    },
    types::{Bitboard, Castle, Color, Move, MoveKind, MoveList, PieceKind, Square},
};

type AttacksFn = fn(Square, Bitboard) -> Bitboard;

const PIECE_ATTACKS: [(PieceKind, AttacksFn); 5] = [
    (PieceKind::Knight, |sq, _| knight_attacks(sq)),
    (PieceKind::Bishop, bishop_attacks),
    (PieceKind::Rook, rook_attacks),
    (PieceKind::Queen, queen_attacks),
    (PieceKind::King, |sq, _| king_attacks(sq)),
];

fn push_moves_to(from: Square, targets: Bitboard, kind: MoveKind, list: &mut MoveList) {
    for to in targets {
        list.push(Move::new(from, to, kind));
    }
}

fn non_pawn_moves(board: &Board, list: &mut MoveList) {
    let us = board.side_to_move();
    let theirs = board.occupied_by(us.opposite());
    let occupied = board.occupied();

    for (kind, attacks) in PIECE_ATTACKS {
        for from in board.pieces_of(us, kind) {
            let targets = attacks(from, occupied);
            push_moves_to(from, targets & theirs, MoveKind::Capture, list);
            push_moves_to(from, targets & !occupied, MoveKind::Quiet, list);
        }
    }
}

fn push_pawn_move(
    from: Square,
    to: Square,
    capture: bool,
    promo_rank: Bitboard,
    list: &mut MoveList,
) {
    if promo_rank.contains(to) {
        let kinds = if capture {
            [
                MoveKind::CapturePromoQueen,
                MoveKind::CapturePromoRook,
                MoveKind::CapturePromoBishop,
                MoveKind::CapturePromoKnight,
            ]
        } else {
            [
                MoveKind::PromoQueen,
                MoveKind::PromoRook,
                MoveKind::PromoBishop,
                MoveKind::PromoKnight,
            ]
        };
        for kind in kinds {
            list.push(Move::new(from, to, kind));
        }
    } else {
        let kind = if capture {
            MoveKind::Capture
        } else {
            MoveKind::Quiet
        };
        list.push(Move::new(from, to, kind));
    }
}

fn pawn_moves(board: &Board, list: &mut MoveList) {
    let us = board.side_to_move();
    let theirs = board.occupied_by(us.opposite());
    let empty = !board.occupied();

    let (forward, start_rank, promo_rank) = match us {
        Color::White => (Direction::North, Bitboard::RANK_2, Bitboard::RANK_8),
        Color::Black => (Direction::South, Bitboard::RANK_7, Bitboard::RANK_1),
    };

    for from in board.pieces_of(us, PieceKind::Pawn) {
        let from_bb = Bitboard::from_square(from);

        let single = forward.step(from_bb) & empty;
        for to in single {
            push_pawn_move(from, to, false, promo_rank, list);
        }
        if start_rank.contains(from) {
            for to in forward.step(single) & empty {
                list.push(Move::new(from, to, MoveKind::DoublePush));
            }
        }

        let attacks = pawn_attacks(us, from_bb);
        for to in attacks & theirs {
            push_pawn_move(from, to, true, promo_rank, list);
        }

        if let Some(ep) = board.en_passant()
            && attacks.contains(ep)
        {
            list.push(Move::new(from, ep, MoveKind::EnPassant));
        }
    }
}

fn castling_moves(board: &Board, list: &mut MoveList) {
    let us = board.side_to_move();
    let them = us.opposite();
    let kings = board.pieces_of(us, PieceKind::King);
    let occupied = board.occupied();

    for castle in Castle::for_color(us) {
        if board.castling().contains(castle.right)
            && kings.contains(castle.king_from)
            && !occupied.intersects(castle.must_be_empty)
            && !castle
                .must_be_safe
                .into_iter()
                .any(|sq| board.is_square_attacked(sq, them))
        {
            list.push(Move::new(castle.king_from, castle.king_to, castle.kind));
        }
    }
}

/// Appends every pseudo-legal move to `list`: moves that follow the movement
/// rules but may leave the mover's own king attacked.
///
/// Castling moves are already fully checked, including that the king does not
/// pass through an attacked square.
pub fn generate_pseudo_legal(board: &Board, list: &mut MoveList) {
    pawn_moves(board, list);
    non_pawn_moves(board, list);
    castling_moves(board, list);
}

/// Appends every legal move to `list`: the pseudo-legal moves that do not
/// leave the mover's own king attacked.
pub fn generate_legal(board: &Board, list: &mut MoveList) {
    let us = board.side_to_move();
    let mut pseudo = MoveList::new();
    generate_pseudo_legal(board, &mut pseudo);

    for &mv in &pseudo {
        let next = board.make_move(mv);
        let king_safe = next
            .king_square(us)
            .is_some_and(|king| !next.is_square_attacked(king, us.opposite()));
        if king_safe {
            list.push(mv);
        }
    }
}

impl Board {
    /// Every legal move in this position, generated fresh on each call.
    ///
    /// ```
    /// use engine::Board;
    ///
    /// assert_eq!(Board::start().legal_moves().len(), 20);
    /// ```
    #[must_use]
    pub fn legal_moves(&self) -> MoveList {
        let mut moves = MoveList::new();
        generate_legal(self, &mut moves);
        moves
    }
}

/// Counts the positions reachable from `board` in exactly `depth` moves.
///
/// Comparing the count with published numbers is the standard check that
/// move generation and [`Board::make_move`] are correct.
///
/// # Examples
///
/// ```
/// use engine::{movegen::perft, Board};
///
/// let start = Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
/// assert_eq!(perft(&start, 2), 400);
/// ```
#[must_use]
pub fn perft(board: &Board, depth: u32) -> u64 {
    if depth == 0 {
        return 1;
    }
    let moves = board.legal_moves();
    if depth == 1 {
        return u64::try_from(moves.len()).unwrap_or(u64::MAX);
    }

    let mut nodes: u64 = 0;
    for &mv in &moves {
        nodes = nodes.saturating_add(perft(&board.make_move(mv), depth.saturating_sub(1)));
    }
    nodes
}
