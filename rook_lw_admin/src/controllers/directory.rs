
use actix_web::{HttpRequest, HttpResponse};
use actix_web::dev::ServiceResponse;
use actix_web::http::header;
use actix_files::Directory;

use std::fs::read_dir;
use std::time::UNIX_EPOCH;

use crate::templates::directory;

#[derive(Debug)]
pub struct DirEntryInfo {
    pub name: String,
    pub is_dir: bool,
    pub size: u64,
    pub mtime: u64,
}

/// Custom directory listing renderer that sorts files by name
pub fn sorted_listing(dir: &Directory, req: &HttpRequest) -> Result<ServiceResponse, std::io::Error> {

    let base_path = if req.path().ends_with("/") {
        req.path().to_string()
    } else {
        format!("{}/", req.path())
    };

    // list files and directories under dir_path, collect attributes
    let mut entries: Vec<DirEntryInfo> = read_dir(&dir.path)?
        .filter_map(Result::ok)
        .filter_map(|e| {
            let name = e.file_name().into_string().ok()?;
            if name.starts_with(".") { return None; }
            let meta = e.metadata().ok()?;
            let is_dir = meta.is_dir();
            let size = if is_dir { 0 } else { meta.len() };
            let mtime = meta.modified().ok()
                .and_then(|mtime| mtime.duration_since(UNIX_EPOCH).ok())
                .map(|d| d.as_secs())
                .unwrap_or(0);
            Some(DirEntryInfo { name, is_dir, size, mtime })
        })
        .collect();

    entries.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    let body = directory::directory_listing(&dir.base.to_string_lossy(), &entries, &base_path);

    let resp = HttpResponse::Ok()
        .insert_header((header::CONTENT_TYPE, "text/html; charset=utf-8"))
        .body(body.into_string());

    Ok(ServiceResponse::new(req.clone(), resp))
}