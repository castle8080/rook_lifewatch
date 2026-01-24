use thiserror::Error;
use actix_web::ResponseError;
use actix_web::http::StatusCode;

use rook_lw_image_repo::ImageRepoError;
use serde_json;
use tokio::task::JoinError;
use tracing::error;

pub type RookLWAdminResult<T> = Result<T, RookLWAdminError>;

#[derive(Debug, Error)]
pub enum RookLWAdminError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Concurrency error: {0}")]
    Concurrency(String),

    #[error("IO error: {0}")]
    Io(String),

    #[error("Invalid input")]
    Input(String),
}

impl From<std::io::Error> for RookLWAdminError {
    fn from(err: std::io::Error) -> Self {
        RookLWAdminError::Io(format!("IO error: {}", err))
    }
}

impl From<JoinError> for RookLWAdminError {
    fn from(err: JoinError) -> Self {
        RookLWAdminError::Concurrency(format!("Join error: {}", err))
    }
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
            RookLWAdminError::Concurrency(_) => StatusCode::INTERNAL_SERVER_ERROR,
            RookLWAdminError::Io(_) => StatusCode::INTERNAL_SERVER_ERROR,
            RookLWAdminError::Input(_) => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> actix_web::HttpResponse {
        error!(
            status_code = %self.status_code(),
            error = %&self,
            "Error response."
        );
        actix_web::HttpResponse::build(self.status_code())
            .json(serde_json::json!({"error": self.to_string()}))
    }
}