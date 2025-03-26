#[derive(Debug)]
pub enum AppError {
    Network(reqwest::Error),
    Io(std::io::Error),
    Json(serde_json::Error),
    Other(String),
}

impl std::error::Error for AppError {}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::Network(e) => write!(f, "Network error: {}", e),
            AppError::Io(e) => write!(f, "I/O error: {}", e),
            AppError::Json(e) => write!(f, "JSON error: {}", e),
            AppError::Other(s) => write!(f, "Error: {}", s),
        }
    }
}

impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        AppError::Network(err)
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::Io(err)
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::Json(err)
    }
}

impl From<String> for AppError {
    fn from(err: String) -> Self {
        AppError::Other(err)
    }
} 