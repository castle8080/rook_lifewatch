
use leptos::prelude::*;
use leptos_router::components::*;
use leptos_router::path;

use crate::components::Admin;
use crate::components::DaemonControl;
use crate::components::ImageDisplay;
use crate::components::ImageSearch;
use crate::components::Login;
use crate::components::NavBar;
use crate::components::ProtectedRoute;
use crate::components::ServerOperations;

use crate::services::ImageInfoService;
use crate::services::DaemonService;
use crate::services::LoginService;
use crate::services::ServeropsService;
use crate::services::UserService;

#[component]
pub fn App() -> impl IntoView {
    let api_base_path = "";
    
    // Create user service signal for reactive authentication state
    let user_service = RwSignal::new(UserService::new());
    
    // Provide services via context
    provide_context(user_service);
    provide_context(ImageInfoService::new(api_base_path));
    provide_context(DaemonService::new(api_base_path));
    provide_context(LoginService::new(api_base_path));
    provide_context(ServeropsService::new(api_base_path));
    
    view! {
        <div class="app-shell">
            <Router base="/admin">
                <NavBar />
                <main class="app-content">
                    <Routes fallback=|| view! { <ProtectedRoute view=|| view! { <Admin /> } /> }>
                        <Route
                            path=path!("login")
                            view=Login
                        />
                        <Route
                            path=path!("images")
                            view=|| view! { <ProtectedRoute view=ImageSearch /> }
                        />
                        <Route
                            path=path!("serverops")
                            view=|| view! { <ProtectedRoute view=ServerOperations /> }
                        />
                        <Route
                            path=path!("daemon")
                            view=|| view! { <ProtectedRoute view=DaemonControl /> }
                        />
                        <Route
                            path=path!("image_display/*image_id")
                            view=|| view! { <ProtectedRoute view=ImageDisplay /> }
                        />
                    </Routes>
                </main>
            </Router>
        </div>
    }
}

