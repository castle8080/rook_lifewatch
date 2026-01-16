

use actix_web::web::ServiceConfig;
use actix_web::{web, Responder};
use crate::templates::home;
use crate::templates::HtmlTemplate;

async fn hello() -> impl Responder {
    HtmlTemplate(home::home_page())
}

pub fn register(sc: &mut ServiceConfig) {
    sc.route("/hello", web::get().to(hello));
}
