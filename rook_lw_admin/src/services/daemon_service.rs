use sysinfo::{System, ProcessesToUpdate, Process};
use chrono::DateTime;
use tracing::info;

use rook_lw_models::process::ProcessInfo;

use crate::{RookLWAdminError, RookLWAdminResult};

const DAEMON_PROCESS_NAME: &str = "rook_lw_daemon";

pub struct DaemonService {
    app_dir: String
}

impl DaemonService {

    pub fn new(app_dir: impl Into<String>) -> RookLWAdminResult<Self> {
        Ok(Self {
            app_dir: app_dir.into()
        })
    }

    pub fn stop(self: &Self) -> RookLWAdminResult<String> {
        info!("Stop requested for daemon.");
        let mut sys = System::new_all();
        sys.refresh_processes(ProcessesToUpdate::All, true);

        match Self::find_daemon_process(&mut sys)? {
            None => Ok("Daemon process not running.".into()),
            Some(p) => {
                info!("Sending kill signal to: {}", p.pid());
                if !p.kill() {
                    Err(RookLWAdminError::Other("Unable to kill process.".into()))
                }
                else {
                    Ok("Daemon process kill signal sent.".into())
                }
            }
        }
    }

    pub fn get_status(self: &Self) -> RookLWAdminResult<Option<ProcessInfo>> {
        let mut sys = System::new_all();
        sys.refresh_processes(ProcessesToUpdate::All, true);

        Ok(match Self::find_daemon_process(&mut sys)? {
            Some(p) => Some(Self::transform(p)?),
            None => None
        })
    }

    pub fn list_all() -> RookLWAdminResult<Vec<ProcessInfo>> {
        let mut sys = System::new_all();
        sys.refresh_processes(ProcessesToUpdate::All, true);

        sys
            .processes()
            .iter()
            .map(|(_pid, process)| Self::transform(process))
            .collect()
    }

    fn find_daemon_process<'a>(system: &'a mut System) -> RookLWAdminResult<Option<&'a Process>> {
        let mut found = system.processes()
            .iter()
            .filter(|(_pid, p)|
                p.name().to_string_lossy().starts_with(DAEMON_PROCESS_NAME) &&
                p.thread_kind().is_none()
            )
            .map(|(_pid, p)| p);

        let proc = found.next();

        if found.next().is_some() {
            Err(RookLWAdminError::Other("More than one daemon process found.".into()))
        }
        else {
            Ok(proc)
        }
    }

    fn transform(process: &Process) -> RookLWAdminResult<ProcessInfo> {
        let pid = process.pid();
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

        Ok(ProcessInfo {
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
        })
    }

}