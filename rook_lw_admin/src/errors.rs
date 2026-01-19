use thiserror::Error;
use actix_web::ResponseError;
use actix_web::http::StatusCode;

use rook_lw_image_repo::ImageRepoError;
use serde_json;

pub type RookLWAdminResult<T> = Result<T, RookLWAdminError>;

#[derive(Debug, Error)]
pub enum RookLWAdminError {
    #[error("Database error: {0}")]
    Database(String),
}

impl From<ImageRepoError> for RookLWAdminError {
    fn from(err: ImageRepoError) -> Self {
        RookLWAdminError::Database(format!("{err}"))
    }
}

impl ResponseError for RookLWAdminError {
    fn status_code(&self) -> StatusCode {
        match self {
            RookLWAdminError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> actix_web::HttpResponse {
        actix_web::HttpResponse::build(self.status_code())
            .json(serde_json::json!({"error": self.to_string()}))
    }
}