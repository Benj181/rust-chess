//! Move generation: produces the moves available in a position.

use crate::{
    attacks::{KING_ATTACKS, KNIGHT_ATTACKS, bishop_attacks, pawn_attacks, rook_attacks},
    bitboard::Bitboard,
    board::Board,
    castling::CastlingRights,
    direction::Direction,
    moves::{Move, MoveKind, MoveList},
    piece::{Color, PieceKind},
    square::Square,
};

fn push_targets(
    from: Square,
    targets: Bitboard,
    ours: Bitboard,
    theirs: Bitboard,
    list: &mut MoveList,
) {
    for to in targets & !ours {
        if theirs.contains(to) {
            list.push(Move::new(from, to, MoveKind::Capture));
        } else {
            list.push(Move::new(from, to, MoveKind::Quiet));
        }
    }
}

fn knight_moves(board: &Board, list: &mut MoveList) {
    let us = board.side_to_move();
    let ours = board.color_pieces(us);
    let theirs = board.color_pieces(us.opposite());

    for from in board.pieces(PieceKind::Knight) & ours {
        if let Some(targets) = KNIGHT_ATTACKS.get(from.as_usize()) {
            push_targets(from, *targets, ours, theirs, list);
        }
    }
}

fn king_moves(board: &Board, list: &mut MoveList) {
    let us = board.side_to_move();
    let ours = board.color_pieces(us);
    let theirs = board.color_pieces(us.opposite());

    for from in board.pieces(PieceKind::King) & ours {
        if let Some(targets) = KING_ATTACKS.get(from.as_usize()) {
            push_targets(from, *targets, ours, theirs, list);
        }
    }
}

fn slider_moves(
    board: &Board,
    list: &mut MoveList,
    kind: PieceKind,
    attacks_fn: fn(Square, Bitboard) -> Bitboard,
) {
    let us = board.side_to_move();
    let ours = board.color_pieces(us);
    let theirs = board.color_pieces(us.opposite());
    let occupied = board.occupied();

    for from in board.pieces(kind) & ours {
        let targets = attacks_fn(from, occupied);
        push_targets(from, targets, ours, theirs, list);
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
                MoveKind::CapturePromoQ,
                MoveKind::CapturePromoR,
                MoveKind::CapturePromoB,
                MoveKind::CapturePromoN,
            ]
        } else {
            [
                MoveKind::PromoQ,
                MoveKind::PromoR,
                MoveKind::PromoB,
                MoveKind::PromoN,
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
    let ours = board.color_pieces(us);
    let theirs = board.color_pieces(us.opposite());
    let empty = !board.occupied();

    let (forward, start_rank, promo_rank) = match us {
        Color::White => (Direction::North, Bitboard::RANK_2, Bitboard::RANK_8),
        Color::Black => (Direction::South, Bitboard::RANK_7, Bitboard::RANK_1),
    };

    for from in board.pieces(PieceKind::Pawn) & ours {
        let from_bb = Bitboard::from_square(from);

        let single = forward.step(from_bb) & empty;
        for to in single {
            push_pawn_move(from, to, false, promo_rank, list);
        }
        if !(from_bb & start_rank).is_empty() {
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

struct CastleSide {
    kind: MoveKind,
    king_to: u8,
    must_be_empty: &'static [u8],
    must_be_safe: &'static [u8],
}

const KING_SIDE: CastleSide = CastleSide {
    kind: MoveKind::KingCastle,
    king_to: 6,
    must_be_empty: &[5, 6],
    must_be_safe: &[4, 5, 6],
};

const QUEEN_SIDE: CastleSide = CastleSide {
    kind: MoveKind::QueenCastle,
    king_to: 2,
    must_be_empty: &[1, 2, 3],
    must_be_safe: &[4, 3, 2],
};

fn on_rank(files: &'static [u8], rank: u8) -> impl Iterator<Item = Square> {
    files
        .iter()
        .filter_map(move |&file| Square::from_file_rank(file, rank))
}

fn castling_moves(board: &Board, list: &mut MoveList) {
    let us = board.side_to_move();
    let them = us.opposite();
    let (rank, sides) = match us {
        Color::White => (
            0,
            [
                (CastlingRights::WHITE_KING, &KING_SIDE),
                (CastlingRights::WHITE_QUEEN, &QUEEN_SIDE),
            ],
        ),
        Color::Black => (
            7,
            [
                (CastlingRights::BLACK_KING, &KING_SIDE),
                (CastlingRights::BLACK_QUEEN, &QUEEN_SIDE),
            ],
        ),
    };

    let Some(from) = Square::from_file_rank(4, rank) else {
        return;
    };
    if !(board.pieces(PieceKind::King) & board.color_pieces(us)).contains(from) {
        return;
    }

    let occupied = board.occupied();
    for (right, side) in sides {
        if !board.castling().contains(right)
            || on_rank(side.must_be_empty, rank).any(|sq| occupied.contains(sq))
            || on_rank(side.must_be_safe, rank).any(|sq| board.is_square_attacked(sq, them))
        {
            continue;
        }
        if let Some(to) = Square::from_file_rank(side.king_to, rank) {
            list.push(Move::new(from, to, side.kind));
        }
    }
}

pub fn generate_pseudo_legal(board: &Board, list: &mut MoveList) {
    pawn_moves(board, list);
    castling_moves(board, list);
    knight_moves(board, list);
    king_moves(board, list);
    slider_moves(board, list, PieceKind::Rook, rook_attacks);
    slider_moves(board, list, PieceKind::Bishop, bishop_attacks);
    slider_moves(board, list, PieceKind::Queen, |sq, occupied| {
        rook_attacks(sq, occupied) | bishop_attacks(sq, occupied)
    });
}

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

#[must_use]
pub fn perft(board: &Board, depth: u32) -> u64 {
    if depth == 0 {
        return 1;
    }
    let mut list = MoveList::new();
    generate_legal(board, &mut list);
    if depth == 1 {
        return u64::try_from(list.len()).unwrap_or(u64::MAX);
    }

    let mut nodes: u64 = 0;
    for &mv in &list {
        nodes = nodes.saturating_add(perft(&board.make_move(mv), depth.saturating_sub(1)));
    }
    nodes
}
