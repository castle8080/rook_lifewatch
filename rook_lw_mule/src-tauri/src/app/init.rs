use tracing::error;

use crate::RookLWMuleResult;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

fn try_run() -> RookLWMuleResult<()> {
    let _guard = crate::app::logging::init_logging()?;

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet])
        .setup(crate::app::menu::setup_menu)
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

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
