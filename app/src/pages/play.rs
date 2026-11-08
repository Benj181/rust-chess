use crate::components::board::ChessBoard;
use crate::components::move_list::MoveList;
use crate::state::PlayState;
use engine::game::{Game, GameResult};
use leptos::{ev, prelude::*};

fn status_text(game: &Game) -> String {
    match game.result() {
        None => format!("{} to move", game.board().side_to_move()),
        Some(GameResult::Win { winner, reason }) => format!("{winner} wins by {reason}"),
        Some(GameResult::Draw(reason)) => format!("draw by {reason}"),
    }
}

#[component]
pub fn PlayPage() -> impl IntoView {
    let state = PlayState::provide();
    let deselect = window_event_listener(ev::click, move |_| state.selected.set(None));
    on_cleanup(move || deselect.remove());

    view! {
        <div class="play">
            <div class="board-frame">
                <span class="frame-label top">"rust-chess"</span>
                <ChessBoard />
                <span class="frame-label bottom">{move || state.game.with(status_text)}</span>
            </div>
            <MoveList />
        </div>
    }
}
