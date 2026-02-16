use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_navigate;

use crate::services::{UserService, LoginService};
use crate::components::ErrorDisplay;

#[component]
pub fn Login() -> impl IntoView {
    let navigate = use_navigate();
    
    let (username, set_username) = signal(String::new());
    let (password, set_password) = signal(String::new());
    let (error, set_error) = signal::<Option<String>>(None);
    let (is_loading, set_is_loading) = signal(false);

    // Get services from context
    let login_service = expect_context::<LoginService>();
    let user_service_signal = expect_context::<RwSignal<UserService>>();

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_error.set(None);
        set_is_loading.set(true);

        let username_val = username.get();
        let password_val = password.get();
        
        // Clone for async block
        let login_service = login_service.clone();
        let navigate = navigate.clone();

        spawn_local(async move {
            // First, initialize the UserService with credentials
            let mut user_service = user_service_signal.get_untracked();
            
            match user_service.login(username_val.clone(), password_val.clone()) {
                Ok(()) => {
                    // Now verify with the server
                    match login_service.verify_login(&user_service).await {
                        Ok(()) => {
                            // Update the context with authenticated user service
                            user_service_signal.set(user_service);
                            
                            // Redirect to home/admin page
                            navigate("/", Default::default());
                        }
                        Err(e) => {
                            // Server rejected the credentials
                            set_error.set(Some(format!("Authentication failed: {}", e)));
                            set_is_loading.set(false);
                        }
                    }
                }
                Err(e) => {
                    set_error.set(Some(format!("Failed to initialize credentials: {}", e)));
                    set_is_loading.set(false);
                }
            }
        });
    };

    view! {
        <section class="section">
            <div class="container">
                <div class="columns is-centered">
                    <div class="column is-5-tablet is-4-desktop">
                        <div class="box">
                            <h1 class="title is-4 has-text-centered mb-5">
                                "Rook Life Watch"
                            </h1>
                            <h2 class="subtitle is-6 has-text-centered mb-5">
                                "Admin Login"
                            </h2>

                            <Show when=move || error.get().is_some()>
                                <ErrorDisplay error=error />
                            </Show>

                            <form on:submit=on_submit>
                                <div class="field">
                                    <label class="label">"Username"</label>
                                    <div class="control">
                                        <input
                                            class="input"
                                            type="text"
                                            placeholder="Enter username"
                                            prop:value=move || username.get()
                                            on:input=move |ev| {
                                                set_username.set(event_target_value(&ev));
                                            }
                                            prop:disabled=move || is_loading.get()
                                            required
                                        />
                                    </div>
                                </div>

                                <div class="field">
                                    <label class="label">"Password"</label>
                                    <div class="control">
                                        <input
                                            class="input"
                                            type="password"
                                            placeholder="Enter password"
                                            prop:value=move || password.get()
                                            on:input=move |ev| {
                                                set_password.set(event_target_value(&ev));
                                            }
                                            prop:disabled=move || is_loading.get()
                                            required
                                        />
                                    </div>
                                </div>

                                <div class="field">
                                    <div class="control">
                                        <button
                                            class="button is-primary is-fullwidth"
                                            type="submit"
                                            prop:disabled=move || is_loading.get()
                                            class:is-loading=move || is_loading.get()
                                        >
                                            "Log In"
                                        </button>
                                    </div>
                                </div>
                            </form>
                        </div>
                    </div>
                </div>
            </div>
        </section>
    }
}
