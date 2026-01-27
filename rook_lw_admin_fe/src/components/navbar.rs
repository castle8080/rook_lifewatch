use leptos::*;
use leptos::prelude::*;

#[component]
pub fn NavBar() -> impl IntoView {
    // Track if the menu is active (open)
    let (is_active, set_is_active) = signal(false);
    let toggle_menu = move |_| set_is_active.update(|v| *v = !*v);
    view! {
        <nav class="navbar is-primary" role="navigation" aria-label="main navigation">
            <div class="navbar-brand">
                <a class="navbar-item" href="/admin/">
                    <strong>Rook Life Watch</strong>
                </a>
                <a
                    role="button"
                    class=move || if is_active.get() { "navbar-burger is-active" } else { "navbar-burger" }
                    aria-label="menu"
                    aria-expanded=move || if is_active.get() { "true" } else { "false" }
                    data-target="navbarBasic"
                    on:click=toggle_menu
                >
                    <span aria-hidden="true"></span>
                    <span aria-hidden="true"></span>
                    <span aria-hidden="true"></span>
                </a>
            </div>
            <div
                id="navbarBasic"
                class=move || if is_active.get() { "navbar-menu is-active" } else { "navbar-menu" }
            >
                <div class="navbar-start">
                    <a class="navbar-item" href="/admin/images" on:click=move |_| set_is_active.set(false)>
                        "Image Search"
                    </a>
                    <a class="navbar-item" href="/admin/daemon" on:click=move |_| set_is_active.set(false)>
                        "Daemon Control"
                    </a>
                    <a class="navbar-item" href="/admin/serverops" on:click=move |_| set_is_active.set(false)>
                        "Server Operations"
                    </a>
                </div>
            </div>
        </nav>
    }
}