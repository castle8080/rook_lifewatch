use actix_web::{Responder, HttpResponse, web};
use actix_web::web::ServiceConfig;
use tokio::task::spawn_blocking;
use serde_qs::actix::QsQuery;

use rook_lw_models::image::ImageInfoSearchOptions;

use crate::RookLWAdminError;
use crate::app::AppState;

pub async fn search_image_info(
        state: web::Data<AppState>,
        query: QsQuery<ImageInfoSearchOptions>,
    ) -> Result<impl Responder, RookLWAdminError>
{
    let repo = state.image_info_repo.clone();

    // Query is sync, so run in thread pool.
    let image_info = spawn_blocking(move || {
        repo.search_image_info(&query)
    }).await??;

    Ok(HttpResponse::Ok().json(image_info))
}

pub fn register(sc: &mut ServiceConfig) {
    sc.route("/api/image_info", web::get().to(search_image_info));
}