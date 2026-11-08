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
fn GameMenu() -> impl IntoView {
    let state = expect_context::<PlayState>();
    let open = RwSignal::new(false);
    let toggle = move |e: ev::MouseEvent| {
        e.stop_propagation();
        open.update(|o| *o = !*o);
    };
    let close = window_event_listener(ev::click, move |_| open.set(false));
    on_cleanup(move || close.remove());

    let new_game = move |_| state.load(Game::new());
    let undo = move |_| {
        state.game.update(|g| g.undo());
        state.go_live();
    };
    let cant_undo = move || state.game.with(|g| g.moves().is_empty());
    let demo = move |_| state.load(crate::dev::demo_game());

    let over = move || state.game.with(|g| g.result().is_some());
    let draw = move |_| {
        state.game.update(|g| {
            let _ = g.agree_draw();
        });
        state.go_live();
    };
    let resign = move |_| {
        state.game.update(|g| {
            let _ = g.resign(g.board().side_to_move());
        });
        state.go_live();
    };

    view! {
        <div class="menu-anchor">
            <button on:click=toggle class:open=open title="more">
                <Icon path=icons::MORE />
            </button>
            <Show when=move || open.get()>
                <div class="menu">
                    <button on:click=draw disabled=over>
                        "offer draw"
                    </button>
                    <button on:click=resign disabled=over>
                        "resign"
                    </button>
                    <button on:click=new_game>"new game"</button>
                    <button on:click=undo disabled=cant_undo>
                        "take back"
                    </button>
                    {cfg!(debug_assertions)
                        .then(|| view! { <button on:click=demo>"demo"</button> })}
                </div>
            </Show>
        </div>
    }
}

#[component]
fn MoveControls() -> impl IntoView {
    let state = expect_context::<PlayState>();
    let flip = move |_| state.perspective.update(|p| *p = p.opposite());

    let ply = state.shown_ply;
    let first = move |_| state.view_ply(0);
    let prev = move |_| state.view_ply(ply.get().saturating_sub(1));
    let next = move |_| state.view_ply(ply.get() + 1);
    let last = move |_| state.go_live();
    let at_start = move || ply.get() == 0;
    let at_end = move || state.is_live();

    view! {
        <div class="move-controls">
            <button on:click=flip title="flip board">
                <Icon path=icons::FLIP />
            </button>
            <button on:click=first disabled=at_start title="first move">
                <Icon path=icons::FIRST />
            </button>
            <button on:click=prev disabled=at_start title="previous move">
                <Icon path=icons::PREV />
            </button>
            <button on:click=next disabled=at_end title="next move">
                <Icon path=icons::NEXT />
            </button>
            <button on:click=last disabled=at_end title="last move">
                <Icon path=icons::LAST />
            </button>
            <GameMenu />
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
        </div>
    }
}
