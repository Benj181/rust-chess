use leptos::prelude::*;

pub const FLIP: &str = "M5 13V3M2 6l3-3 3 3M11 3v10M8 10l3 3 3-3";
pub const FIRST: &str = "M4 3v10M12 3 7 8l5 5";
pub const PREV: &str = "M10 3 5 8l5 5";
pub const NEXT: &str = "M6 3l5 5-5 5";
pub const LAST: &str = "M12 3v10M4 3l5 5-5 5";
pub const MORE: &str = "M3 8h.01M8 8h.01M13 8h.01";

#[component]
pub fn Icon(path: &'static str) -> impl IntoView {
    view! {
        <svg class="icon" viewBox="0 0 16 16" aria-hidden="true">
            <path d=path />
        </svg>
    }
}
