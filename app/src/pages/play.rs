use crate::components::board::ChessBoard;
use engine::{game::Game, piece::Color};
use leptos::prelude::*;

#[component]
pub fn PlayPage() -> impl IntoView {
    let game = RwSignal::new(Game::new());
    let moves = move || game.with(|g| g.legal_moves().len());
    let side = move || match game.with(|g| g.board().side_to_move()) {
        Color::White => "white",
        Color::Black => "black",
    };

    view! {
        <h1>"rust-chess"</h1>
        <p>"The engine finds " {move || { moves }} " legal moves for " {move || { side }}</p>
        <ChessBoard game />
    }
}
