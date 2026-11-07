use crate::components::board::ChessBoard;
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
    let new_game = move |_| {
        game.set(Game::new());
        selected.set(None);
    };
    let undo = move |_| {
        game.update(|g| g.undo());
        selected.set(None);
    };
    let cant_undo = move || game.with(|g| g.moves().is_empty());
    let perspective = RwSignal::new(Color::White);
    let flip = move |_| perspective.update(|p| *p = p.opposite());

    view! {
        <h1>"rust-chess"</h1>
        <p>{status}</p>
        <ChessBoard game selected perspective />
        <div>
            <button on:click=new_game>"new game"</button>
            <button on:click=undo disabled=cant_undo>
                "undo"
            </button>
            <button on:click=flip>"flip"</button>
        </div>
    }
}
