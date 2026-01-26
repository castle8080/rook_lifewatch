use leptos::*;
use leptos::prelude::*;

#[component]
pub fn ErrorDisplay(error: ReadSignal<Option<String>>) -> impl IntoView {
    view! {
        { move ||
            match error.get() {
                None => view! { }.into_any(),
                Some(e) => view! {
                    <div class="error_display">
                        <h2>Error:</h2>
                        <div>
                            { e }
                        </div>
                    </div>
                }.into_any()
            }
        }
    }
}