use actix_web::{Responder, HttpResponse, web};
use actix_web::web::ServiceConfig;
use tokio::task::spawn_blocking;

use crate::RookLWAdminError;
use crate::app::AppState;

pub async fn daemon_status(
        state: web::Data<AppState>
    ) -> Result<impl Responder, RookLWAdminError>
{
    let status = spawn_blocking(move || state.daemon_service.get_status()).await??;
    
    match status {
        Some(p_info) => Ok(HttpResponse::Ok().json(p_info)),
        None => Ok(HttpResponse::NotFound().body("No running daemon process found."))
    }
}

pub fn register(sc: &mut ServiceConfig) {
    sc.route("/api/daemon/status", web::get().to(daemon_status));
}