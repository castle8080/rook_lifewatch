
use leptos::*;
use leptos::prelude::*;

#[component]
pub fn Admin() -> impl IntoView {
    view! {
        <section class="section">
            <div class="container">
                <div class="box" style="max-width: 600px; margin: 0 auto;">
                    <h1 class="title is-3 has-text-centered mb-4">Rook Life Watch Admin</h1>
                    <p class="subtitle is-5 has-text-centered mb-5">Device Administration Console</p>
                    <div class="content">
                        <p>
                            Welcome to the administrative control panel for the Rook Life Watch system.
                        </p>
                        <ul>
                            <li>
                                <strong>Purpose:</strong> Manage and monitor devices that use cameras to detect motion and classify activity in the field.
                            </li>
                            <li>
                                <strong>Use Case:</strong> Designed for remote monitoring, such as trail cameras and similar deployments.
                            </li>
                            <li>
                                <strong>Features:</strong> Device control, image search, and server operations.
                            </li>
                        </ul>
                        <p>
                            Use the navigation bar above to access available functions.
                        </p>
                    </div>
                </div>
            </div>
        </section>
    }
}