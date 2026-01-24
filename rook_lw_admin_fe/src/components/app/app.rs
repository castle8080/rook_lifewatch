
use leptos::prelude::*;
use leptos_router::components::*;
use leptos_router::path;

use crate::components::image_search::ImageSearch;
use crate::components::admin::Admin;

#[component]
pub fn App() -> impl IntoView {
    view! {
        <h1>"Rook Life Watch"</h1>
        <Router base="/admin">
            <Routes fallback=Admin>
                <Route
                    path=path!("images")
                    view=ImageSearch
                />
            </Routes>
        </Router>
    }
}

