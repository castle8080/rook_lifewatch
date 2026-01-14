use actix_web::{web, Responder, Scope};
use crate::templates::{home, HtmlTemplate};

async fn home() -> impl Responder {
    HtmlTemplate(home::home_page())
}

pub fn register() -> Scope {
    web::scope("")
        .route("/", web::get().to(home))
}
