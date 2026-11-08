use crate::components::board::PieceIcon;
use crate::state::PlayState;
use engine::{
    board::Board,
    piece::{Color, Piece, PieceKind},
};
use leptos::prelude::*;

const START: [(PieceKind, u32); 5] = [
    (PieceKind::Queen, 1),
    (PieceKind::Rook, 2),
    (PieceKind::Bishop, 2),
    (PieceKind::Knight, 2),
    (PieceKind::Pawn, 8),
];

fn captured_by(board: &Board, player: Color) -> Vec<(Piece, u32)> {
    let victim = player.opposite();
    START
        .iter()
        .map(|&(kind, start)| {
            let missing = start.saturating_sub(board.pieces_of(victim, kind).count());
            (Piece::new(victim, kind), missing)
        })
        .filter(|&(_, missing)| missing > 0)
        .collect()
}

fn lead(board: &Board, player: Color) -> Option<u32> {
    board
        .material(player)
        .checked_sub(board.material(player.opposite()))
        .filter(|&n| n > 0)
}

#[component]
pub fn MaterialStrip(at_top: bool) -> impl IntoView {
    let state = expect_context::<PlayState>();
    let player = move || {
        let bottom = state.perspective.get();
        if at_top { bottom.opposite() } else { bottom }
    };
    let pieces = move || {
        let player = player();
        state
            .shown
            .with(|b| captured_by(b, player))
            .into_iter()
            .map(|(piece, count)| {
                let stack = (0..count).map(|_| view! { <PieceIcon piece /> }).collect_view();
                view! { <span class="captured">{stack}</span> }
            })
            .collect_view()
    };
    let lead = move || {
        let player = player();
        state
            .shown
            .with(|b| lead(b, player))
            .map(|n| view! { <span class="lead">"+"{n}</span> })
    };

    view! {
        <div class="material" class:top=at_top class:bottom=!at_top>
            {lead}
            {pieces}
        </div>
    }
}
