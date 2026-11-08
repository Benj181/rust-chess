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

fn press_key(state: PlayState, key: &str) -> bool {
    match key {
        "ArrowLeft" => state.step_back(),
        "ArrowRight" => state.step_forward(),
        "ArrowUp" => state.view_ply(0),
        "ArrowDown" => state.go_live(),
        "Escape" => state.cancel_promotion(),
        "f" => state.flip(),
        _ => return false,
    }
    true
}

#[component]
pub fn PlayPage() -> impl IntoView {
    let state = PlayState::provide();
    let deselect = window_event_listener(ev::click, move |_| state.cancel_promotion());
    let keys = window_event_listener(ev::keydown, move |e| {
        let modified = e.ctrl_key() || e.meta_key() || e.alt_key();
        if !modified && press_key(state, &e.key()) {
            e.prevent_default();
        }
    });
    on_cleanup(move || {
        deselect.remove();
        keys.remove();
    });

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
