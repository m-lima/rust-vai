/// Error for all operations in this library
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Could not infer configuration directory")]
    Path,
    #[error("Failed to fetch suggestions: {0}")]
    Fetch(#[from] ureq::Error),
    #[error("Could not write to {0}: {1}")]
    Write(std::path::PathBuf, std::io::Error),
    #[error("Could not read from {0}: {1}")]
    Read(std::path::PathBuf, std::io::Error),
    #[error("Could not serialize configuration: {0}")]
    Serialize(Serialize),
    #[error("Could not deserialize configuration: {0}")]
    Deserialize(Deserialize),
    #[error("Failed to parse query response: {0}")]
    Parse(Parse),
    #[error("Could not open query in a browser: {0}")]
    Browser(std::io::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum Serialize {
    #[error(transparent)]
    Binary(#[from] bincode::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum Deserialize {
    #[error(transparent)]
    Binary(#[from] bincode::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum Parse {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}
