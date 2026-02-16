use leptos::*;
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

#[component]
pub fn NavBar() -> impl IntoView {
    // Track if the menu is active (open)
    let (is_active, set_is_active) = signal(false);
    let toggle_menu = move |_| set_is_active.update(|v| *v = !*v);
    
    // Get user service for logout functionality
    let user_service_signal = use_context::<RwSignal<crate::services::UserService>>();
    let navigate = use_navigate();
    
    // Check if user is logged in to show/hide logout button
    let is_authenticated = move || {
        user_service_signal
            .map(|us| us.get().is_authenticated())
            .unwrap_or(false)
    };
    
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
                <div class="navbar-end">
                    <div class="navbar-item">
                        <Show when=is_authenticated>
                            {
                                let navigate = navigate.clone();
                                view! {
                                    <button class="button is-light" on:click=move |_| {
                                        if let Some(user_service_signal) = user_service_signal {
                                            let mut user_service = user_service_signal.get();
                                            let _ = user_service.logout();
                                            user_service_signal.set(user_service);
                                            navigate("/login", Default::default());
                                        }
                                        set_is_active.set(false);
                                    }>
                                        <span class="icon">
                                            <i class="fas fa-sign-out-alt"></i>
                                        </span>
                                        <span>"Logout"</span>
                                    </button>
                                }
                            }
                        </Show>
                    </div>
                </div>
            </div>
        </nav>
    }
}