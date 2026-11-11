use crate::state::PlayState;
use engine::Color;
use leptos::prelude::*;

fn white_share(score: i32) -> f64 {
    100.0 / (1.0 + (-0.003_68 * f64::from(score)).exp())
}

#[component]
pub fn EvalBar() -> impl IntoView {
    let state = expect_context::<PlayState>();
    let white_on_top = move || state.perspective.get() == Color::Black;
    let height = move || format!("{:.1}%", white_share(state.eval.get()));

    view! {
        <div class="eval-bar" class:flipped=white_on_top>
            <div class="eval-white" style:height=height></div>
        </div>
    }
}
