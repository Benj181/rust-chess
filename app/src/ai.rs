use crate::state::PlayState;
use engine::{
    search::{search, Limits},
    Board,
};
use leptos::prelude::*;
use std::time::Duration;

const LIMITS: Limits = Limits {
    depth: 12,
    nodes: 1_000_000,
};
const PAINT_FIRST: Duration = Duration::from_millis(50);

fn board_to_answer(state: PlayState) -> Option<Board> {
    let waiting = state.ai_to_move() && state.game.with(|g| g.result().is_none());
    waiting.then(|| state.game.with(|g| g.board().clone()))
}

fn think(state: PlayState, board: &Board) {
    let Some(mv) = search(board, LIMITS).best else {
        return;
    };
    if board_to_answer(state).as_ref() == Some(board) {
        state.game.update(|g| {
            let _ = g.play(mv);
        });
    }
}

pub fn start(state: PlayState) {
    Effect::new(move |_| {
        if let Some(board) = board_to_answer(state) {
            set_timeout(move || think(state, &board), PAINT_FIRST);
        }
    });
}
