use leptos::task::spawn_local;
use leptos::{ev::SubmitEvent, prelude::*};
use serde::{Deserialize, Serialize};

use tracing::info;

#[component]
pub fn App() -> impl IntoView {

    let on_action = move |_| {
        info!("On action.....");

        
    };

    view! {
        <main class="container">
            "Load the admin page!"
            <a href="http://192.168.1.22:8080/admin">"Load Admin"</a>
            <p>
                <button on:click=on_action>"Press Me"</button>
            </p>
        </main>
    }
}
