use leptos::task::spawn_local;
use leptos::prelude::*;
use serde_json::json;
use serde_wasm_bindgen::to_value;

use tracing::info;

use crate::services::command;


#[component]
pub fn App() -> impl IntoView {

    let on_action = move |_| {
        info!("On action.....");
        spawn_local(async move {
            let json = json!({
                "name": "Mr. Bryan"
            });
            let payload = to_value(&json).unwrap();
            let r = command::invoke("greet", payload).await;
            tracing::info!("Result: {:?}", r);
        });
    };

    view! {
        <main class="container">
            "Load the admin page!"
            <a href="http://127.0.0.1:8081/admin/">"Load Admin"</a>
            <p>
                <button on:click=on_action>"Press Me"</button>
            </p>
        </main>
    }
}
