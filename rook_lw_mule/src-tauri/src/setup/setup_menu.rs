use std::sync::Mutex;

use tauri::{App, AppHandle, Url};
use tauri::menu::{Menu, Submenu, PredefinedMenuItem, MenuItem, MenuEvent};
use tauri::Manager;

use tracing::{info, error};

use crate::RookLWMuleResult;
use crate::app::AppState;

/*
fn get_main_url() -> Url {
     Url::parse(match tauri::is_dev() {
        true => "http://localhost:1420/",
        false => "http://tauri.localhost/",
    }).unwrap()
}
    */

fn on_view_main(app: &AppHandle, _event: MenuEvent) -> RookLWMuleResult<()> {
    if let Some(w) = app.get_webview_window("main") {
        let home_url = {
            let app_state = app.state::<Mutex<AppState>>();
            let app_state = app_state.lock().unwrap();
            app_state
                .get_home_url()
                .as_ref()
                .and_then(|url| Url::parse(url).ok())
        };
        if let Some(home_url) = home_url {
            info!("Navigating back to: {}", &home_url);
            w.navigate(home_url)?;
        }
    }
    Ok(())
}

fn on_view_devtools(app: &AppHandle, _event: MenuEvent) ->RookLWMuleResult<()> {
    if let Some(w) = app.get_webview_window("main") {
        w.open_devtools();
    }
    Ok(())
}

fn _setup(app: &mut App) -> RookLWMuleResult<()> {
    let menu = Menu::new(app)?;

    let quit = PredefinedMenuItem::quit(app, None)?;
    let file_submenu = Submenu::with_items(app, "File", true, &[&quit])?;

    let view_main = MenuItem::new(app, "Main", true, None::<String>)?;
    let view_devtools = MenuItem::new(app, "Devtools", true, None::<String>)?;
    let view_submenu = Submenu::with_items(app, "View", true, &[&view_main, &view_devtools])?;

    menu.append(&file_submenu)?;
    menu.append(&view_submenu)?;

    app.on_menu_event(move |app, event| {
        let action_result = if event.id() == view_devtools.id() {
            on_view_devtools(app, event)
        }
        else if event.id() == view_main.id() {
            on_view_main(app, event)
        }
        else {
            Ok(())
        };
        if let Err(e) = action_result {
            // Todo: find a way to surface errors in the app.
            error!("Error: menu action: {}", e);
        }
    });

    app.set_menu(menu)?;

    Ok(())
}

pub fn setup(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    _setup(app).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
}