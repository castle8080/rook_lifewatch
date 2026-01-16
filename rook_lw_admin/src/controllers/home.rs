use actix_web::{Responder, HttpResponse};
use actix_web::http::header;
use actix_web::web;
use actix_web::web::ServiceConfig;

async fn home() -> impl Responder {
    HttpResponse::Found()
        .append_header((header::LOCATION, "/admin"))
        .finish()
}

pub fn register(sc: &mut ServiceConfig) {
    sc.route("/", web::get().to(home));
}
