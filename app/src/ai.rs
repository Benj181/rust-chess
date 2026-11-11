use crate::state::PlayState;
use engine::{
    search::{search, Limits},
    Board, Color, GameResult,
};
use leptos::prelude::*;
use std::time::Duration;

const LIMITS: Limits = Limits {
    depth: 12,
    nodes: 1_000_000,
};
const EVAL_LIMITS: Limits = Limits {
    depth: 12,
    nodes: 30_000,
};
const PAINT_FIRST: Duration = Duration::from_millis(50);

fn board_to_answer(state: PlayState) -> Option<Board> {
    let waiting = state.ai_to_move() && state.game.with(|g| g.result().is_none());
    waiting.then(|| state.game.with(|g| g.board().clone()))
}

fn think(state: PlayState, board: &Board) {
    let started = js_sys::Date::now();
    let report = search(board, LIMITS);
    let ms = js_sys::Date::now() - started;
    leptos::logging::log!(
        "search: depth {}, {} nodes, {ms} ms",
        report.depth,
        report.nodes
    );

    let Some(mv) = report.best else {
        return;
    };
    if board_to_answer(state).as_ref() == Some(board) {
        state.game.update(|g| {
            let _ = g.play(mv);
        });
    }
}

fn white_score(board: &Board) -> i32 {
    let score = search(board, EVAL_LIMITS).score;
    match board.side_to_move() {
        Color::White => score,
        Color::Black => -score,
    }
}

fn shown_score(state: PlayState) -> i32 {
    let drawn = state.is_live()
        && state
            .game
            .with(|g| matches!(g.result(), Some(GameResult::Draw(_))));
    if drawn {
        0
    } else {
        state.shown.with(white_score)
    }
}

pub fn start(state: PlayState) {
    Effect::new(move |_| state.eval.set(shown_score(state)));

    Effect::new(move |_| {
        if let Some(board) = board_to_answer(state) {
            set_timeout(move || think(state, &board), PAINT_FIRST);
        }
    });
}
