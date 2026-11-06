use crate::START_FEN;
use engine::{board::Board, movegen::generate_legal, moves::MoveList};
use leptos::prelude::*;

#[component]
pub fn PlayPage() -> impl IntoView {
    let legal_moves = Board::from_fen(START_FEN).map_or(0, |board| {
        let mut moves = MoveList::new();
        generate_legal(&board, &mut moves);
        moves.len()
    });

    view! {
        <h1>"rust-chess"</h1>
        <p>"The engine finds " {legal_moves} " legal moves in the starting position."</p>
    }
}
