use leptos::*;
use leptos::prelude::*;

#[component]
pub fn Admin() -> impl IntoView {
    view! {
        <h1>"Admin Panel"</h1>
        <p>"Welcome to the admin panel."</p>
        <ul>
            <li><a href="/admin/images">"Image Search"</a></li>
        </ul>
    }
}