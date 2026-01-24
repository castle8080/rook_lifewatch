use actix_web::{web, Responder};
use actix_web::Either;
use actix_web::web::ServiceConfig;
use actix_files::NamedFile;
use std::path::{Path, PathBuf};

use crate::{RookLWAdminError, RookLWAdminResult};
use crate::app::AppState;

/// Handler for /admin and /admin/*
pub async fn admin_handler(
    state: web::Data<AppState>,
    tail: Option<web::Path<String>>,
) -> RookLWAdminResult<impl Responder>
{
    let rel_path: PathBuf = match tail {
        Some(tail_path) => tail_path.into_inner().into(),
        None => PathBuf::new(),
    };

    // Validate the path to prevent directory traversal
    for component in rel_path.iter() {
        if component.to_string_lossy().starts_with(".") {
            return Err(RookLWAdminError::Input(format!("Invalid path component: {:?}", component)));
        }
    }

    let file_path = Path::new(&state.admin_static_dir).join(&rel_path);
    if file_path.exists() && file_path.is_file() {
        Ok(Either::Left(NamedFile::open(file_path)?))
    } else {
        // fallback to index.html, prevent caching
        use actix_web::http::header;
        let file = NamedFile::open(Path::new(&state.admin_static_dir).join("index.html"))?;
        Ok(Either::Right(file.customize()
            .insert_header((header::CACHE_CONTROL, "no-store, no-cache, must-revalidate, max-age=0"))
            .insert_header((header::PRAGMA, "no-cache"))
            .insert_header((header::EXPIRES, "0"))))
    }
}

pub fn register(sc: &mut ServiceConfig) {
    sc.route("/admin", web::get().to(admin_handler));
    sc.route("/admin/{tail:.*}", web::get().to(admin_handler));
}
