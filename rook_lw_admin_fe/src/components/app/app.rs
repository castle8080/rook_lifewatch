
use leptos::prelude::*;
use leptos_router::components::*;
use leptos_router::path;

use crate::components::admin::Admin;
use crate::components::daemon::DaemonControl;
use crate::components::image_search::ImageSearch;

use crate::services::ImageInfoService;
use crate::services::DaemonService;

#[component]
pub fn App() -> impl IntoView {
    provide_context(ImageInfoService::new(""));
    provide_context(DaemonService::new(""));
    view! {
        <h1>"Rook Life Watch"</h1>
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

