use crate::components::board::ChessBoard;
use engine::game::{Game, GameResult};
use leptos::prelude::*;

fn status_text(game: &Game) -> String {
    match game.result() {
        None => format!("{} to move", game.board().side_to_move()),
        Some(GameResult::Win { winner, reason }) => format!("{winner} wins by {reason}"),
        Some(GameResult::Draw(reason)) => format!("draw by {reason}"),
    }
}
#[component]
pub fn PlayPage() -> impl IntoView {
    let game = RwSignal::new(Game::new());
    let status = move || game.with(status_text);

    view! {
        <h1>"rust-chess"</h1>
        <p>{status}</p>
        <ChessBoard game />
    }
}
