use actix_web::{Responder, HttpResponse, web, HttpRequest};
use actix_web::web::ServiceConfig;
use actix_web::body::BodyStream;
use bytes::Bytes;
use futures_util::TryStreamExt;
use serde_qs::actix::QsQuery;
use tokio::task::spawn_blocking;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tracing::error;

use std::io::Read;

use rook_lw_models::image::ImageInfoSearchOptions;
use crate::RookLWAdminError;
use crate::app::AppState;

fn guess_content_type(image_path: &str) -> &'static str {
    let ext = image_path.rsplit('.').next().map(|s| s.to_ascii_lowercase());
    match ext.as_deref() {
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("png") => "image/png",
        Some("gif") => "image/gif",
        Some("bmp") => "image/bmp",
        Some("webp") => "image/webp",
        Some("tiff") => "image/tiff",
        _ => "application/octet-stream",
    }
}

pub async fn search_image_info(
    state: web::Data<AppState>,
    query: QsQuery<ImageInfoSearchOptions>,
) -> Result<impl Responder, RookLWAdminError> {
    let repo = state.image_info_repo.clone();
    let image_info = spawn_blocking(move || {
        repo.search_image_info(&query)
    }).await??;
    Ok(HttpResponse::Ok()
        .insert_header(("Cache-Control", "public, max-age=60"))
        .json(image_info))
}

pub async fn get_image_info_by_id(
    state: web::Data<AppState>,
    image_id: web::Path<String>,
) -> Result<impl Responder, RookLWAdminError> {
    let repo = state.image_info_repo.clone();
    let image_id = image_id.into_inner();
    let image_info = spawn_blocking(move || {
        repo.get_image_info(&image_id)
    }).await??;
    match image_info {
        Some(info) => Ok(HttpResponse::Ok()
            .insert_header(("Cache-Control", "public, max-age=600"))
            .json(info)),
        None => Ok(HttpResponse::NotFound().json(serde_json::json!({"error": "Image not found"}))),
    }
}

pub async fn get_image(
    state: web::Data<AppState>,
    path: web::Path<String>,
    _req: HttpRequest,
) -> Result<actix_web::HttpResponse, RookLWAdminError> {
    let repo = state.image_store_repo.clone();
    let image_path = path.into_inner();
    let content_type = guess_content_type(&image_path);

    // Channel for streaming chunks
    let (tx, rx) = mpsc::channel::<Result<Bytes, std::io::Error>>(8);

    // Spawn blocking task to read and send chunks
    tokio::task::spawn_blocking(move || {
        let mut reader = match repo.read(&image_path) {
            Err(e) => {
                error!("Error opening image: {} - {}", image_path, e);
                let _ = tx.blocking_send(Err(std::io::Error::other(format!("Error opening image: {} - {}", image_path, e))));
                return;
            }
            Ok(r) => r
        };
        let mut buf = [0u8; 8192];
        loop {
            match reader.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    if tx.blocking_send(Ok(Bytes::copy_from_slice(&buf[..n]))).is_err() {
                        break;
                    }
                }
                Err(e) => {
                    error!("Error transferring image: {}", e);
                    let _ = tx.blocking_send(Err(e));
                    break;
                }
            }
        }
    });

    let body_stream = BodyStream::new(ReceiverStream::new(rx).map_err(|e| actix_web::error::ErrorInternalServerError(e)));
    Ok(actix_web::HttpResponse::Ok()
        .content_type(content_type)
        .insert_header(("Cache-Control", "public, max-age=31536000, immutable"))
        .body(body_stream))
}

pub fn register(sc: &mut ServiceConfig) {
    sc.route("/api/image_info", web::get().to(search_image_info));
    sc.route("/api/image_info/{image_id}", web::get().to(get_image_info_by_id));
    sc.route("/api/image/{image_path:.*}", web::get().to(get_image));
}