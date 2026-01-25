use sysinfo::{System, ProcessesToUpdate};
use chrono::DateTime;

use rook_lw_models::process::ProcessInfo;

use crate::{RookLWAdminError, RookLWAdminResult};

const DAEMON_PROCESS_NAME: &str = "rook_lw_daemon";

pub struct DaemonService {
    start_script: String
}

impl DaemonService {

    pub fn new(start_script: impl Into<String>) -> RookLWAdminResult<Self> {
        Ok(Self {
            start_script: start_script.into()
        })
    }

    pub fn create(app_dir: impl AsRef<str>) -> RookLWAdminResult<Self> {
        DaemonService::new("")
    }

    pub fn get_status(self: &Self) -> RookLWAdminResult<Option<ProcessInfo>> {
        let processes = Self::list_all()?;
        let mut found = processes
            .iter()
            .filter(|p|
                p.name.starts_with(DAEMON_PROCESS_NAME) &&
                p.thread_kind.is_none()
            );

        let process = found.next();
        if found.next().is_some() {
            return Err(RookLWAdminError::Other(format!("Unexpected number of matching processes.").into()));
        }

        Ok(process.cloned())
    }

    pub fn list_all() -> RookLWAdminResult<Vec<ProcessInfo>> {
        let mut sys = System::new_all();
        sys.refresh_processes(ProcessesToUpdate::All, true);

        let mut process_list: Vec<ProcessInfo> = Vec::new();

        for (pid, process) in sys.processes() {
            let name = process.name().to_string_lossy().to_string();
            let cmd = process
                .cmd().iter()
                .map(|f| f.to_string_lossy().to_string())
                .collect::<Vec<String>>().join(" ");

            let start_time =
                DateTime::from_timestamp(
                    process.start_time().try_into()?, 0
                ).ok_or(RookLWAdminError::Other("Invalid timestamp conversion.".into()))?;

            let t_kind = process.thread_kind()
                .map(|tk| format!("{:?}", tk).to_string());

            process_list.push(ProcessInfo {
                pid: pid.as_u32(),
                ppid: process.parent().map(|_pid| _pid.as_u32()),
                name: name,
                cmd: cmd,
                status: format!("{:?}", process.status()),
                started: start_time.into(),
                memory: process.memory(),
                cpu_usage_percent: process.cpu_usage(),
                cpu_accumulated_time: process.accumulated_cpu_time(),
                thread_kind: t_kind,
            });
        }

        Ok(process_list)
    }

}