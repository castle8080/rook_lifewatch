use actix_web::{Responder, HttpResponse, web};
use actix_web::web::ServiceConfig;

use serde::Serialize;

use rook_lw_models::image::ImageInfo;
use rook_lw_image_repo::image_info::ImageInfoRepository;

async fn search_image_info() -> impl Responder {
    
    let image_info: Vec<ImageInfo> = vec![];

    return HttpResponse::Ok().json(image_info);
}

pub fn register(sc: &mut ServiceConfig) {
    sc.route("/api/image_info", web::get().to(search_image_info));
}