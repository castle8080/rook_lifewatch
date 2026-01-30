use thiserror::Error;

pub type RookLWMuleResult<T> = Result<T, RookLWMuleError>;

#[derive(Debug, Error, Clone)]
pub enum RookLWMuleError {
    #[error("I/O error: {0}")]
    Io(String),

    #[error("parse error: {0}")]
    Parse(String),

    #[error("tauri error: {0}")]
    Tauri(String),
}

impl From<std::io::Error> for RookLWMuleError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err.to_string())
    }
}

impl From<tauri::Error> for RookLWMuleError {
    fn from(err: tauri::Error) -> Self {
        Self::Tauri(err.to_string())
    }
}