

use actix_web::{web, Responder, Scope};
use crate::templates::home;
use crate::templates::HtmlTemplate;

async fn hello() -> impl Responder {
    HtmlTemplate(home::home_page())
}

pub fn register() -> Scope {
    web::scope("")
        .route("/hello", web::get().to(hello))
}
