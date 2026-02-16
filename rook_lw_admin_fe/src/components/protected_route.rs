use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

use crate::services::UserService;

/// A wrapper component that protects routes requiring authentication.
/// If the user is not authenticated, they are redirected to the login page.
#[component]
pub fn ProtectedRoute<F, IV>(
    /// The child content to render if authenticated
    view: F,
) -> impl IntoView
where
    F: Fn() -> IV + 'static + Send,
    IV: IntoView + 'static,
{
    let user_service = expect_context::<RwSignal<UserService>>();
    let navigate = use_navigate();
    
    let is_authenticated = Memo::new(move |_| {
        user_service.get().is_authenticated()
    });

    // Redirect to login if not authenticated
    Effect::new(move || {
        if !is_authenticated.get() {
            navigate("/login", Default::default());
        }
    });

    move || {
        if is_authenticated.get() {
            view().into_any()
        } else {
            view! {
                <div class="section">
                    <div class="container">
                        <p>"Redirecting to login..."</p>
                    </div>
                </div>
            }.into_any()
        }
    }
}
