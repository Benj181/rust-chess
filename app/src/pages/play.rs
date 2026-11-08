use crate::components::board::ChessBoard;
use crate::components::move_list::MoveList;
use engine::{
    game::{Game, GameResult},
    piece::Color,
};
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
    let selected = RwSignal::new(None);
    let status = move || game.with(status_text);
    let perspective = RwSignal::new(Color::White);
    let de_select = move |_| selected.set(None);

    view! {
        <div class="play" on:click=de_select>
            <div class="board-column">
                <h1>"rust-chess"</h1>
                <p>{status}</p>
                <ChessBoard game selected perspective />
            </div>
            <MoveList game selected perspective />
        </div>
    }
}
