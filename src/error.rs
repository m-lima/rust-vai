/// Error for all operations in this library
///
/// Possible kinds:
/// * Path
/// * Fecth
/// * Write
/// * Read
/// * Serialize
/// * Deserialize
/// * Parse
/// * Browser
#[derive(Debug, Clone)]
pub struct Error {
    kind: Kind,
}

impl Error {
    fn new(kind: Kind) -> Self {
        Self { kind }
    }
}

#[derive(Debug, Clone)]
enum Kind {
    Path,
    Fetch(u16),
    Write(String),
    Read(String),
    Serialize,
    Deserialize,
    Parse,
    Browser(String),
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        match &self.kind {
            Kind::Path => write!(fmt, "Could not infer configuration directory"),
            Kind::Fetch(status) => write!(fmt, "Got status code {} from suggest query", status),
            Kind::Write(path) => write!(fmt, "Could not write to {}", path),
            Kind::Read(path) => write!(fmt, "Could not read from {}", path),
            Kind::Serialize => write!(fmt, "Could not serialize configuration"),
            Kind::Deserialize => write!(fmt, "Could not deserialize configuration"),
            Kind::Parse => write!(fmt, "Failed to parse response"),
            Kind::Browser(url) => write!(fmt, "Could not open query in browser ({})", url),
        }
    }
}

#[must_use]
pub(crate) fn path() -> Error {
    Error::new(Kind::Path)
}

#[must_use]
pub(crate) fn fetch(status: u16) -> Error {
    Error::new(Kind::Fetch(status))
}

#[must_use]
pub(crate) fn write<P: AsRef<std::path::Path>>(path: P) -> Error {
    Error::new(Kind::Write(String::from(
        path.as_ref().to_str().unwrap_or("NULL"),
    )))
}

#[must_use]
pub(crate) fn read<P: AsRef<std::path::Path>>(path: P) -> Error {
    Error::new(Kind::Read(String::from(
        path.as_ref().to_str().unwrap_or("NULL"),
    )))
}

#[must_use]
pub(crate) fn serialize<E>(_: E) -> Error {
    Error::new(Kind::Serialize)
}

#[must_use]
pub(crate) fn deserialize<E>(_: E) -> Error {
    Error::new(Kind::Deserialize)
}

#[must_use]
pub(crate) fn parse<E>(_: E) -> Error {
    Error::new(Kind::Parse)
}

#[must_use]
pub(crate) fn browser(url: String) -> Error {
    Error::new(Kind::Browser(url))
}
