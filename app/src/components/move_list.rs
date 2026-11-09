use crate::components::icons::{self, Icon};
use crate::state::PlayState;
use engine::game::Game;
use leptos::{ev, prelude::*};

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
fn ResignButton() -> impl IntoView {
    let state = expect_context::<PlayState>();
    let asking = RwSignal::new(false);
    let close = window_event_listener(ev::click, move |_| asking.set(false));
    on_cleanup(move || close.remove());

    let ask = move |e: ev::MouseEvent| {
        e.stop_propagation();
        asking.update(|a| *a = !*a);
    };
    let resign = move |_| {
        state.game.update(|g| {
            let _ = g.resign(g.board().side_to_move());
        });
        state.go_live();
    };

    view! {
        <div class="popover-anchor">
            <button on:click=ask class:active=asking data-tip="resign" aria-label="resign">
                <Icon path=icons::RESIGN />
            </button>
            <Show when=move || asking.get()>
                <div class="popover">
                    <span>"resign this game?"</span>
                    <div class="popover-buttons">
                        <button class="confirm" on:click=resign>
                            "resign"
                        </button>
                        <button>"cancel"</button>
                    </div>
                </div>
            </Show>
        </div>
    }
}

#[component]
fn ActionBar() -> impl IntoView {
    let state = expect_context::<PlayState>();
    let playing = move || state.game.with(|g| g.result().is_none());
    let cant_take_back = move || state.game.with(|g| g.moves().is_empty());
    let show_demo = move || cfg!(debug_assertions) && cant_take_back();

    let take_back = move |_| {
        state.game.update(|g| g.undo());
        state.go_live();
    };
    let draw = move |_| {
        state.game.update(|g| {
            let _ = g.agree_draw();
        });
        state.go_live();
    };
    let new_game = move |_| state.load(Game::new());
    let demo = move |_| state.load(crate::dev::demo_game());

    view! {
        <div class="move-actions">
            <Show
                when=playing
                fallback=move || {
                    view! {
                        <button on:click=new_game data-tip="new game" aria-label="new game">
                            <Icon path=icons::NEW_GAME />
                        </button>
                    }
                }
            >
                <button on:click=draw data-tip="offer draw" aria-label="offer draw">
                    <Icon path=icons::DRAW />
                </button>
                <ResignButton />
                <button on:click=take_back disabled=cant_take_back data-tip="take back" aria-label="take back">
                    <Icon path=icons::TAKE_BACK />
                </button>
            </Show>
            <Show when=show_demo>
                <button on:click=demo data-tip="load demo game" aria-label="load demo game">
                    "demo"
                </button>
            </Show>
        </div>
    }
}

#[component]
fn MoveControls() -> impl IntoView {
    let state = expect_context::<PlayState>();
    let flip = move |_| state.flip();
    let first = move |_| state.view_ply(0);
    let prev = move |_| state.step_back();
    let next = move |_| state.step_forward();
    let last = move |_| state.go_live();
    let at_start = move || state.shown_ply.get() == 0;
    let at_end = move || state.is_live();

    view! {
        <div class="move-controls">
            <button on:click=flip data-tip="flip board" aria-label="flip board">
                <Icon path=icons::FLIP />
            </button>
            <button on:click=first disabled=at_start data-tip="first move" aria-label="first move">
                <Icon path=icons::FIRST />
            </button>
            <button on:click=prev disabled=at_start data-tip="previous move" aria-label="previous move">
                <Icon path=icons::PREV />
            </button>
            <button on:click=next disabled=at_end data-tip="next move" aria-label="next move">
                <Icon path=icons::NEXT />
            </button>
            <button on:click=last disabled=at_end data-tip="last move" aria-label="last move">
                <Icon path=icons::LAST />
            </button>
        </div>
    }
}

#[component]
pub fn MoveList() -> impl IntoView {
    let state = expect_context::<PlayState>();
    let on_pick = move |p| state.view_ply(p);
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
            <Show when=move || !state.is_live()>
                <button class="back-to-live" on:click=move |_| state.go_live()>
                    "back to current move"
                </button>
            </Show>
            <MoveControls />
            <ActionBar />
        </div>
    }
}
