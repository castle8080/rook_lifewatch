use serde::{Serialize, Deserialize};

/// A very basic status result type.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Status {
    pub message: String,
    pub error: Option<String>
}

impl<S: Into<String>> From<S> for Status {
    fn from(s: S) -> Self {
        Self {
            message: s.into(),
            error: None
        }
    }
}