use actix_web::{Responder, HttpResponse, web};
use actix_web::web::ServiceConfig;
use tokio::task::spawn_blocking;

use rook_lw_models::Status;

use crate::RookLWAdminError;
use crate::app::AppState;

pub async fn daemon_status(state: web::Data<AppState>)
    -> Result<impl Responder, RookLWAdminError>
{
    let status = spawn_blocking(move || state.daemon_service.get_status()).await??;
    
    match status {
        Some(p_info) => Ok(HttpResponse::Ok().json(p_info)),
        None => Ok(HttpResponse::NotFound().json(Status {
            message: "No daemon process found.".into(),
            ..Default::default()
        }))
    }
}

pub async fn daemon_stop(state: web::Data<AppState>)
    -> Result<impl Responder, RookLWAdminError>
{
    let status = spawn_blocking(move || state.daemon_service.stop()).await??;
    Ok(HttpResponse::Ok().json(Status { message: status, ..Default::default() }))
}

pub fn register(sc: &mut ServiceConfig) {
    sc.route("/api/daemon/status", web::get().to(daemon_status));
    sc.route("/api/daemon/stop", web::post().to(daemon_status));
}