use engine::{game::Game, piece::Color, square::Square};
use leptos::prelude::*;

#[component]
fn MoveRow(number: usize, white: Option<String>, black: Option<String>) -> impl IntoView {
    view! {
        <div class="move-row">
            <span class="move-number">{number}"."</span>
            <span class="move">{white}</span>
            <span class="move">{black}</span>
        </div>
    }
}

#[component]
fn MoveControls(
    game: RwSignal<Game>,
    selected: RwSignal<Option<Square>>,
    perspective: RwSignal<Color>,
) -> impl IntoView {
    let new_game = move |_| {
        game.set(Game::new());
        selected.set(None);
    };
    let undo = move |_| {
        game.update(|g| g.undo());
        selected.set(None);
    };
    let cant_undo = move || game.with(|g| g.moves().is_empty());
    let flip = move |_| perspective.update(|p| *p = p.opposite());
    let demo = move |_| {
        game.set(crate::dev::demo_game());
        selected.set(None);
    };
    let demo_button = cfg!(debug_assertions).then(|| view! { <button on:click=demo>"demo"</button> });

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
pub fn MoveList(
    game: RwSignal<Game>,
    selected: RwSignal<Option<Square>>,
    perspective: RwSignal<Color>,
) -> impl IntoView {
    let rows = move || {
        game.with(|g| {
            g.san_moves()
                .chunks(2)
                .enumerate()
                .map(|(i, pair)| {
                    let number = i + 1;
                    let white = pair.first().cloned();
                    let black = pair.get(1).cloned();
                    view! { <MoveRow number white black /> }
                })
                .collect_view()
        })
    };

    view! {
        <div class="move-pane">
            <span class="pane-title">"moves"</span>
            <div class="move-rows">{rows}</div>
            <MoveControls game selected perspective />
        </div>
    }
}
