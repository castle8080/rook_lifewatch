use actix_web::{Responder, HttpResponse, web};
use actix_web::web::ServiceConfig;
use tokio::task::spawn_blocking;

use rook_lw_models::process::ProcessInfo;

use crate::RookLWAdminError;
use crate::services::DaemonService;

async fn list_processes() -> Result<impl Responder, RookLWAdminError> {
    let process_list: Vec<ProcessInfo> = spawn_blocking(move || DaemonService::list_all()).await??;
    return Ok(HttpResponse::Ok().json(process_list));
}

pub fn register(sc: &mut ServiceConfig) {
    sc.route("/api/processes", web::get().to(list_processes));
}