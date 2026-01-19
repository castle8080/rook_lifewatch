use actix_web::{Responder, HttpResponse, web};
use actix_web::web::ServiceConfig;
use rook_lw_models::image::ImageInfo;

use tracing::info;

use crate::RookLWAdminError;
use crate::app::AppState;

pub async fn search_image_info(state: web::Data<AppState>) -> Result<impl Responder, RookLWAdminError> {
    let repo = state.image_info_repo.clone();
    info!("In search_image_info, repo_address = {:p}", &*repo);

    // Placeholder: will call repo method later
    let image_info: Vec<ImageInfo> = repo
        .search_image_info_by_date_range(None, None)?;

    Ok(HttpResponse::Ok().json(image_info))
}

pub fn register(sc: &mut ServiceConfig) {
    sc.route("/api/image_info", web::get().to(search_image_info));
}