use leptos::*;
use leptos::prelude::*;
use leptos::task::spawn_local;
use rook_lw_models::process::ProcessInfo;

use crate::services::DaemonService;

#[component]
pub fn ProcessInfo(process_info: ReadSignal<Option<Option<ProcessInfo>>>) -> impl IntoView {
    view! {
        { move ||
            match process_info.get() {
                None => view! {
                    <div>
                        "Deamon status is unknown."
                    </div>
                }.into_any(),
                Some(None) => view! {
                    <div>
                        "Deamon status is not active."
                    </div>
                }.into_any(),
                Some(Some(pi)) => view! {
                    <div>
                        Daemon running.
                        <ul>
                            <li>"Pid: "{ pi.pid }</li>
                            <li>"Memory: "{ pi.memory }</li>
                            <li>"Started: "{ pi.started.to_rfc2822() }</li>
                            <li>"Accumulated CPU: "{ pi.cpu_accumulated_time }</li>
                        </ul>
                    </div>
                }.into_any()
            }
        }
    }
}

#[component]
pub fn DaemonControl() -> impl IntoView {
    let (error, set_error) = signal(None::<String>);
    let (process_info, set_process_info) = signal(None::<Option<ProcessInfo>>);

    let daemon_service = match use_context::<DaemonService>() {
        Some(s) => s,
        None => return view! {
            <div>"Error: could not find daemon service."</div>
        }.into_any()
    };

    Effect::new(move |_| {
        let set_process_info = set_process_info.clone();
        let daemon_service = daemon_service.clone();

        spawn_local(async move {
            match daemon_service.status().await {
                Err(e) => set_error.set(Some(format!("Could not get status: {}", e))),
                Ok(process_info) => set_process_info.set(Some(process_info))
            }
        })
    });

    view! {
        <h1>"Daemon Control"</h1>
        <ProcessInfo process_info=process_info/>
    }.into_any()
}