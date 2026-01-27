use leptos::*;
use leptos::task::spawn_local;
use leptos::prelude::*;

use tracing::info;

use crate::components::ErrorDisplay;

use crate::services::ServeropsService;

#[component]
pub fn ServerOperations() -> impl IntoView {
    let (error, set_error) = signal(None::<String>);

    let serverops_service = match use_context::<ServeropsService>() {
        Some(s) => s,
        None => return view! {
            <div>"Error: could not find serverops service."</div>
        }.into_any()
    };

    let on_server_shutdown = {
        let serverops_service = serverops_service.clone();
        let set_error = set_error.clone();
        info!("On server shutdown.");
        
        move |_| {
            let serverops_service = serverops_service.clone();
            let set_error = set_error.clone();

            spawn_local(async move {
                match serverops_service.shutdown().await {
                    Err(e) => {
                        set_error.set(Some(format!("Failed to start: {}", e)));
                    },
                    Ok(status) => {
                        info!("Shutdown succeeded: {:?}", &status);
                    }
                }
            });
        }
    };

    view! {
        <div class="severops card" style="max-width: 500px; margin: 2em auto;">
            <header class="card-header">
                <p class="card-header-title">
                    <span class="icon has-text-info" style="margin-right: 0.5em;"><i class="fas fa-cogs"></i></span>
                    "Sever Operations"
                </p>
            </header>
            <div class="card-content">
                <ErrorDisplay error=error/>
                <button class="button is-danger" on:click=on_server_shutdown>
                    <span class="icon"><i class="fas fa-play"></i></span>
                    <span>"Shutdown Server"</span>
                </button>
            </div>
        </div>
    }.into_any()
}