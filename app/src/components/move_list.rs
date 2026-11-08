use crate::state::PlayState;
use engine::game::Game;
use leptos::prelude::*;

#[component]
fn MoveCell(
    san: Option<String>,
    ply: usize,
    #[prop(into)] on_pick: Callback<usize>,
    #[prop(into)] current: Signal<usize>,
) -> impl IntoView {
    san.map(|san| {
        view! {
            <span
                class="move"
                on:click=move |_| on_pick.run(ply)
                class:current=move || current.get() == ply
            >
                {san}
            </span>
        }
    })
}

#[component]
fn MoveRow(
    index: usize,
    white: Option<String>,
    black: Option<String>,
    #[prop(into)] on_pick: Callback<usize>,
    #[prop(into)] current: Signal<usize>,
) -> impl IntoView {
    let number = index + 1;
    let ply = 2 * index + 1;
    view! {
        <div class="move-row">
            <span class="move-number">{number}"."</span>
            <MoveCell san=white ply on_pick current />
            <MoveCell san=black ply=ply + 1 on_pick current />
        </div>
    }
}

#[component]
fn MoveControls() -> impl IntoView {
    let state = expect_context::<PlayState>();
    let new_game = move |_| state.load(Game::new());
    let undo = move |_| {
        state.game.update(|g| g.undo());
        state.go_live();
    };
    let cant_undo = move || state.game.with(|g| g.moves().is_empty());
    let flip = move |_| state.perspective.update(|p| *p = p.opposite());
    let demo = move |_| state.load(crate::dev::demo_game());
    let demo_button =
        cfg!(debug_assertions).then(|| view! { <button on:click=demo>"demo"</button> });
    view! {
        <div class="move-controls">
            <button on:click=new_game>"new game"</button>
            <button on:click=undo disabled=cant_undo>
                "undo"
            </button>
            <button on:click=flip>"flip"</button>
            {demo_button}
        </div>
    }
}

#[component]
pub fn MoveList() -> impl IntoView {
    let state = expect_context::<PlayState>();
    let on_pick = move |p| state.viewing.set(Some(p));
    let current = state.shown_ply;
    let rows = move || {
        state.game.with(|g| {
            g.san_moves()
                .chunks(2)
                .enumerate()
                .map(|(index, pair)| {
                    let white = pair.first().cloned();
                    let black = pair.get(1).cloned();
                    view! { <MoveRow index white black on_pick current /> }
                })
                .collect_view()
        })
    };

    view! {
        <div class="move-pane">
            <span class="pane-title">"moves"</span>
            <div class="move-rows">{rows}</div>
            <MoveControls />
        </div>
    }
}
