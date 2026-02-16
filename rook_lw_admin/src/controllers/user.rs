use actix_web::{Responder, HttpResponse, HttpRequest, web};
use actix_web::web::ServiceConfig;

use rook_lw_models::Status;

use crate::RookLWAdminError;
use crate::middleware::get_authenticated_user;

pub async fn login(req: HttpRequest)
    -> Result<impl Responder, RookLWAdminError>
{
    // Check if user info is available in request extensions
    match get_authenticated_user(&req) {
        Some(_user) => {
            Ok(HttpResponse::Ok().json(Status {
                message: "Logged in".into(),
                ..Default::default()
            }))
        },
        None => {
            Err(RookLWAdminError::Authentication(
                "Invalid authentication".into()
            ))
        }
    }
}

pub fn register(sc: &mut ServiceConfig) {
    sc.route("/api/login", web::post().to(login));
}
