
use leptos::prelude::*;
use leptos_router::components::*;
use leptos_router::path;

use crate::components::Admin;
use crate::components::DaemonControl;
use crate::components::ImageDisplay;
use crate::components::ImageSearch;
use crate::components::NavBar;
use crate::components::ScrollToTop;
use crate::components::ServerOperations;

use crate::services::ImageInfoService;
use crate::services::DaemonService;
use crate::services::ServeropsService;

#[component]
pub fn App() -> impl IntoView {
    let api_base_path = "";
    provide_context(ImageInfoService::new(api_base_path));
    provide_context(DaemonService::new(api_base_path));
    provide_context(ServeropsService::new(api_base_path));
    view! {
        <div class="app-shell">
            <NavBar />
            <div class="app-router">
                <Router base="/admin">
                    <main class="app-content">
                        <ScrollToTop />
                        <Routes fallback=Admin>
                            <Route
                                path=path!("images")
                                view=ImageSearch
                            />
                            <Route
                                path=path!("serverops")
                                view=ServerOperations
                            />
                            <Route
                                path=path!("daemon")
                                view=DaemonControl
                            />
                            <Route
                                path=path!("image_display/*image_id")
                                view=ImageDisplay
                            />
                        </Routes>
                    </main>
                </Router>
            </div>
        </div>
    }
}

