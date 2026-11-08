use engine::game::Game;
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
fn MoveControls() -> impl IntoView {}

#[component]
pub fn MoveList(game: RwSignal<Game>) -> impl IntoView {
    let rows = move || {
        game.with(|g| {
            g.moves()
                .chunks(2)
                .enumerate()
                .map(|(i, pair)| {
                    let number = i + 1;
                    let white = pair.first().map(|mv| mv.to_string());
                    let black = pair.get(1).map(|mv| mv.to_string());
                    view! { <MoveRow number white black /> }
                })
                .collect_view()
        })
    };
    view! {
        <div class="move-pane">
            <p>"moves"</p>
            <div>{rows}</div>
        </div>
    }
}
