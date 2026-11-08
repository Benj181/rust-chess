use leptos::prelude::*;

pub const FLIP: &str = "M5 13V3M2 6l3-3 3 3M11 3v10M8 10l3 3 3-3";
pub const FIRST: &str = "M4 3v10M12 3 7 8l5 5";
pub const PREV: &str = "M10 3 5 8l5 5";
pub const NEXT: &str = "M6 3l5 5-5 5";
pub const LAST: &str = "M12 3v10M4 3l5 5-5 5";
pub const TAKE_BACK: &str = "M6 3.5 3 6.5l3 3M3 6.5h6.5a3.5 3.5 0 0 1 0 7H8";
pub const DRAW: &str = "M3 3.5l1.5-1V7.5M11.5 2l-7 12M9.5 10a1.75 1.75 0 1 1 3 1.2L9.5 14H13";
pub const RESIGN: &str = "M4 14V2M4 2.5h8l-2 3 2 3H4";
pub const NEW_GAME: &str = "M8 3v10M3 8h10";

#[component]
pub fn Icon(path: &'static str) -> impl IntoView {
    view! {
        <svg class="icon" viewBox="0 0 16 16" aria-hidden="true">
            <path d=path />
        </svg>
    }
}
