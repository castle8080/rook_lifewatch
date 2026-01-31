use std::sync::Mutex;

use tauri::Manager;
use tauri::Webview;
use tauri::generate_handler;
use tauri::webview::PageLoadPayload;
use tracing::error;

use crate::RookLWMuleResult;

use crate::app::AppState;
use crate::setup::setup_logging;
use crate::setup::setup_menu;

fn on_page_load(wv: &Webview, page_load_payload: &PageLoadPayload<'_>) {
    let app_state = wv.state::<Mutex<AppState>>();
    let mut app_state = app_state.lock().unwrap();
    app_state.page_loaded(page_load_payload.url());
}

fn try_run() -> RookLWMuleResult<()> {
    // Init logging and use guard to make sure logs are flushed on exit.
    let _guard = setup_logging::setup()?;

    // Initialize the logical backend application state.
    let app_state = AppState::new();

    tauri::Builder::default()
        //.plugin(tauri_plugin_opener::init())
        .manage(Mutex::new(app_state))
        .setup(setup_menu::setup)
        .invoke_handler(generate_handler![
            crate::commands::greet
        ])
        .on_page_load(on_page_load)
        .run(tauri::generate_context!())?;
    
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    match try_run() {
        Ok(_) => {},
        Err(e) => {
            error!("Error running app: {}", e);
            panic!("Error running app: {}", e);
        }
    }
}
