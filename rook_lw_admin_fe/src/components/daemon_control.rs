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
        <table>
            <tr>
                <th>"Pid:"</th>
                <td>{process_info.pid}</td>
            </tr>
            <tr>
                <th>"Command:"</th>
                <td>{process_info.cmd}</td>
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
                        <span>
                            "Deamon status is unknown."
                        </span>
                    }.into_any(),
                    Some(None) => view! {
                        <span>
                            "Deamon status is not active."
                        </span>
                    }.into_any(),
                    Some(Some(pi)) => view! {
                        <span>
                            "Daemon running."
                            <br/>
                            <ProcessInfoTable process_info=pi.clone()/>
                        </span>
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
        <button class="button" on:click=on_start>
            "Start"
        </button>
        <button class="button" on:click=on_stop>
            "Stop"
        </button>
    }
}

#[component]
pub fn DaemonControl() -> impl IntoView {
    let (error, set_error) = signal(None::<String>);
    let (process_info, set_process_info) = signal(None::<Option<ProcessInfo>>);
    let (process_info_trigger, set_process_info_trigger) = signal(false);

    let daemon_service = match use_context::<DaemonService>() {
        Some(s) => s,
        None => return view! {
            <div>"Error: could not find daemon service."</div>
        }.into_any()
    };

    // Trigger call on component startup.
    let get_process_status = {
        let set_process_info = set_process_info.clone();
        let daemon_service = daemon_service.clone();
        let set_error = set_error.clone();

        move |_: Option<()>| {
            let set_process_info = set_process_info.clone();
            let daemon_service = daemon_service.clone();
            let process_info_trigger = process_info_trigger.clone();

            // Get value to re-trigger lookup.
            let _ = process_info_trigger.get();

            // clear error
            set_error.set(None);

            spawn_local(async move {
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
        let daemon_service = daemon_service.clone();
        let set_error = set_error.clone();
        let set_process_info_trigger = set_process_info_trigger.clone();

        info!("Calling start.");

        move |_| {
            let daemon_service = daemon_service.clone();
            let set_error = set_error.clone();
            let set_process_info_trigger = set_process_info_trigger.clone();

            spawn_local(async move {
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
        let daemon_service = daemon_service.clone();
        let set_error = set_error.clone();
        let set_process_info_trigger = set_process_info_trigger.clone();

        info!("Calling stop.");

        move |_| {
            let daemon_service = daemon_service.clone();
            let set_error = set_error.clone();
            let set_process_info_trigger = set_process_info_trigger.clone();

            spawn_local(async move {
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
        <div class="daemon_control">
            <h1>"Daemon Control"</h1>
            <ErrorDisplay error=error/>
            <ProcessInfo process_info=process_info/>
            <ControlButtons on_start=on_start on_stop=on_stop />
        </div>
    }.into_any()

}