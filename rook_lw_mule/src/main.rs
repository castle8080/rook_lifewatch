use leptos::prelude::*;

use rook_lw_mule_ui::components;

fn main() {
    console_error_panic_hook::set_once();
    wasm_tracing::set_as_global_default();
    mount_to_body(|| {
        view! {
            <components::App/>
        }
    })
}