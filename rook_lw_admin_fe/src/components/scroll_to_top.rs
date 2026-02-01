use leptos::*;
use leptos::prelude::*;
use leptos_router::hooks::use_location;

use tracing::info;

#[component]
pub fn ScrollToTop() -> impl IntoView {
    let location = use_location();

    Effect::new(move || {
        // When the pathname or query changes, scroll back to the top.
        let pathname = location.pathname.get();
        let query = location.query.get();

        info!(pathname = pathname, query = ?query, "Location change.");

        if let Some(w) = web_sys::window() {
            w.scroll_to_with_x_and_y(0.0, 0.0);
        }
    });

    view! {}
}