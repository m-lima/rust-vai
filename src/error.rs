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
    Serialize(Box<dyn Serde>),
    #[error("Could not deserialize configuration: {0}")]
    Deserialize(Box<dyn Serde>),
    #[error("Failed to parse query response: {0}")]
    Parse(Box<dyn Parse>),
    #[error("Could not open query in a browser: {0}")]
    Browser(std::io::Error),
}

pub trait Serde: std::fmt::Debug + std::error::Error {}
impl<E: serde::de::Error> Serde for E {}

pub trait Parse: std::fmt::Debug + std::error::Error {}
impl Parse for std::io::Error {}
impl Parse for serde_json::Error {}
