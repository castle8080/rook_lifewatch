use std::time::Duration;

use leptos::ev::MouseEvent;
use leptos::*;
use leptos::prelude::*;
use leptos::task::spawn_local;
use num_format::{Locale, ToFormattedString};
use humantime::format_duration;

use rook_lw_models::process::ProcessInfo;

use tracing::info;

use crate::components::ErrorDisplay;
use crate::services::DaemonService;

#[component]
fn ProcessInfoTable(process_info: ProcessInfo) -> impl IntoView {
    let cpu_accumulated = Duration::from_millis(process_info.cpu_accumulated_time);
    view! {
        <table class="table is-bordered is-striped is-narrow is-hoverable is-fullwidth">
            <tbody>
                <tr>
                    <th>"Pid:"</th>
                    <td>{process_info.pid}</td>
                </tr>
                <tr>
                    <th>"Command:"</th>
                    <td style="overflow-wrap: anywhere; word-break: break-word;">{process_info.cmd}</td>
                </tr>
                <tr>
                    <th>"Memory:"</th>
                    <td>{ process_info.memory.to_formatted_string(&Locale::en) }</td>
                </tr>
                <tr>
                    <th>"Started:"</th>
                    <td>{ process_info.started.to_rfc2822() }</td>
                </tr>
                <tr>
                    <th>"CPU Accumulated Time:"</th>
                    <td>{ format!("{}", format_duration(cpu_accumulated)) }</td>
                </tr>
            </tbody>
        </table>
    }
}

#[component]
pub fn ProcessInfo(process_info: ReadSignal<Option<Option<ProcessInfo>>>) -> impl IntoView {
    view! {
        <div class="daemon_control_status">
            { move ||
                match process_info.get() {
                    None => view! {
                        <div class="notification is-warning is-light">
                            <span>"Daemon status is unknown."</span>
                        </div>
                    }.into_any(),
                    Some(None) => view! {
                        <div class="notification is-danger is-light">
                            <span>"Daemon is not active."</span>
                        </div>
                    }.into_any(),
                    Some(Some(pi)) => view! {
                        <div class="notification is-success is-light">
                            <span class="has-text-weight-semibold">"Daemon running."</span>
                            <div style="margin-top: 0.5em;">
                                <ProcessInfoTable process_info=pi.clone()/>
                            </div>
                        </div>
                    }.into_any()
                }
            }
        </div>
    }
}

#[component]
pub fn ControlButtons<F1,F2>(on_start: F1, on_stop: F2) -> impl IntoView
where
    F1: Fn(MouseEvent) + 'static,
    F2: Fn(MouseEvent) + 'static,
{
    view! {
        <div class="buttons is-centered" style="margin-top: 1em;">
            <button class="button is-success" on:click=on_start>
                <span class="icon"><i class="fas fa-play"></i></span>
                <span>"Start"</span>
            </button>
            <button class="button is-danger" on:click=on_stop>
                <span class="icon"><i class="fas fa-stop"></i></span>
                <span>"Stop"</span>
            </button>
        </div>
    }
}

#[component]
pub fn DaemonControl() -> impl IntoView {
    let (error, set_error) = signal(None::<String>);
    let (process_info, set_process_info) = signal(None::<Option<ProcessInfo>>);
    let (process_info_trigger, set_process_info_trigger) = signal(false);

    let base_service = match use_context::<DaemonService>() {
        Some(s) => s,
        None => return view! {
            <div>"Error: could not find daemon service."</div>
        }.into_any()
    };
    
    let user_service_signal = expect_context::<RwSignal<crate::services::UserService>>();

    // Trigger call on component startup.
    let get_process_status = {
        let set_process_info = set_process_info.clone();
        let base_service = base_service.clone();
        let set_error = set_error.clone();

        move |_: Option<()>| {
            let set_process_info = set_process_info.clone();
            let base_service = base_service.clone();
            let process_info_trigger = process_info_trigger.clone();

            // Get value to re-trigger lookup.
            let _ = process_info_trigger.get();

            // clear error
            set_error.set(None);

            spawn_local(async move {
                let user_service = user_service_signal.get_untracked();
                let daemon_service = base_service.with_user_service(user_service);
                
                match daemon_service.status().await {
                    Err(e) => set_error.set(Some(format!("Could not get status: {}", e))),
                    Ok(process_info) => set_process_info.set(Some(process_info))
                }
            })
        }
    };

    // Trigger the get_process_status
    Effect::new(get_process_status);

    // Trigger call to start service
    let on_start = {
        let base_service = base_service.clone();
        let set_error = set_error.clone();
        let set_process_info_trigger = set_process_info_trigger.clone();

        info!("Calling start.");

        move |_| {
            let base_service = base_service.clone();
            let set_error = set_error.clone();
            let set_process_info_trigger = set_process_info_trigger.clone();

            spawn_local(async move {
                let user_service = user_service_signal.get_untracked();
                let daemon_service = base_service.with_user_service(user_service);
                
                match daemon_service.start().await {
                    Err(e) => {
                        set_error.set(Some(format!("Failed to start: {}", e)));
                    },
                    Ok(process_info) => {
                        info!("Start call succeeded: {}", process_info.pid);
                        set_process_info_trigger.set(true);
                    }
                }
            });
        }
    };
    
    let on_stop = {
        let base_service = base_service.clone();
        let set_error = set_error.clone();
        let set_process_info_trigger = set_process_info_trigger.clone();

        info!("Calling stop.");

        move |_| {
            let base_service = base_service.clone();
            let set_error = set_error.clone();
            let set_process_info_trigger = set_process_info_trigger.clone();

            spawn_local(async move {
                let user_service = user_service_signal.get_untracked();
                let daemon_service = base_service.with_user_service(user_service);
                
                match daemon_service.stop().await {
                    Err(e) => {
                        set_error.set(Some(format!("Failed to stop: {}", e)));
                    },
                    Ok(status) => {
                        info!("Stop call succeeded: {}", status.message);
                        set_process_info_trigger.set(true);
                    }
                }
            });
        }
    };

    view! {
        <div class="daemon_control card" style="max-width: 500px; margin: 2em auto;">
            <header class="card-header">
                <p class="card-header-title">
                    <span class="icon has-text-info" style="margin-right: 0.5em;"><i class="fas fa-cogs"></i></span>
                    Daemon Control
                </p>
            </header>
            <div class="card-content">
                <ErrorDisplay error=error/>
                <ProcessInfo process_info=process_info/>
                <ControlButtons on_start=on_start on_stop=on_stop />
            </div>
        </div>
    }.into_any()

}