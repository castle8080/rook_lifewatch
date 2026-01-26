
use leptos::prelude::*;
use leptos_router::components::*;
use leptos_router::path;

use crate::components::Admin;
use crate::components::DaemonControl;
use crate::components::ImageSearch;

use crate::components::NavBar;

use crate::services::ImageInfoService;
use crate::services::DaemonService;

#[component]
pub fn App() -> impl IntoView {
    provide_context(ImageInfoService::new(""));
    provide_context(DaemonService::new(""));
    view! {
        <NavBar />
        <Router base="/admin">
            <Routes fallback=Admin>
                <Route
                    path=path!("images")
                    view=ImageSearch
                />
                <Route
                    path=path!("daemon")
                    view=DaemonControl
                />
            </Routes>
        </Router>
    }
}

