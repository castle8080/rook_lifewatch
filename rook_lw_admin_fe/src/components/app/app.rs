
use leptos::prelude::*;

use crate::components::image_search::ImageSearch;

#[component]
pub fn App() -> impl IntoView {
    view! {
        <h1>"Rook Life Watch"</h1>
        <ul>
            <li><a href="/var/images">"View Images"</a></li>
            <li><a href="/var/logs">"View Logs"</a></li>
        </ul>
        <hr/>
        <ImageSearch/>
    }
}

