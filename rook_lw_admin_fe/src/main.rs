use leptos::prelude::*;

use rook_lw_admin_fe::components::app::App;

fn main() {
    console_error_panic_hook::set_once();
    wasm_tracing::set_as_global_default();
    leptos::mount::mount_to_body(|| view! { <App /> })
}