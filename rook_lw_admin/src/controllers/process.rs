use actix_web::{Responder, HttpResponse, web};
use actix_web::web::ServiceConfig;
use sysinfo::{System, ProcessesToUpdate};

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
struct ProcessInfo {
    pid: u32,
    name: String,
    cmd: String,
    status: String,
    memory: u64,
    cpu: f32,
}

async fn list_processes() -> impl Responder {
    let mut sys = System::new_all();
    sys.refresh_processes(ProcessesToUpdate::All, true);

    let mut process_list: Vec<ProcessInfo> = Vec::new();

    for (pid, process) in sys.processes() {
        let name = process.name().to_string_lossy().to_string();
        let cmd = process
            .cmd().iter()
            .map(|f| f.to_string_lossy().to_string())
            .collect::<Vec<String>>().join(" ");

        process_list.push(ProcessInfo {
            pid: pid.as_u32(),
            name: name,
            cmd: cmd,
            status: format!("{:?}", process.status()),
            memory: process.memory(),
            cpu: process.cpu_usage(),
        });
    }

    return HttpResponse::Ok().json(process_list);
}

pub fn register(sc: &mut ServiceConfig) {
    sc.route("/api/processes", web::get().to(list_processes));
}