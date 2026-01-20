
use leptos::prelude::*;

use crate::components::image_search::ImageSearch;

#[component]
pub fn App() -> impl IntoView {
    let (count, set_count) = signal(0);

    view! {
        <h1>"Rook Life Watch"</h1>
        <ul>
            <li><a href="/var/images">"View Images"</a></li>
            <li><a href="/var/logs">"View Logs"</a></li>
        </ul>
        <ImageSearch />
        <hr/>
        <button
            on:click=move |_| set_count.set(count.get() + 1)
        >
            "Click me: "
            {count}
        </button>
        <p>
            "Double count: "
            {move || count.get() * 2}
        </p>
    }
}

