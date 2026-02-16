use thiserror::Error;

pub type RookLWAppResult<T> = Result<T, RookLWAppError>;

#[derive(Debug, Error, Clone)]
pub enum RookLWAppError {
    #[error("I/O error: {0}")]
    Io(String),

    #[error("parse error: {0}")]
    Parse(String),

    #[error("request error: {0}")]
    Request(String),

    #[error("server error: {0}")]
    Server(String),
    
    #[error("{0}")]
    Other(String),
}

impl From<gloo_net::Error> for RookLWAppError {
    fn from(err: gloo_net::Error) -> Self {
        RookLWAppError::Io(err.to_string())
    }
}

impl From<serde_qs::Error> for RookLWAppError {
    fn from(err: serde_qs::Error) -> Self {
        RookLWAppError::Parse(err.to_string())
    }
}