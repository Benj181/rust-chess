use crate::components::board::ChessBoard;
use engine::game::Game;
use leptos::prelude::*;

#[component]
pub fn PlayPage() -> impl IntoView {
    view! {
        <h1>"rust-chess"</h1>
        <p>"The engine finds " {Game::new().legal_moves().len()} " legal moves in the starting position."</p>
        <ChessBoard/>
    }
}
