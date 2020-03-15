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

pub fn path() -> Error {
    Error::new(Kind::Path)
}

pub fn fetch(status: u16) -> Error {
    Error::new(Kind::Fetch(status))
}

pub fn write<P: AsRef<std::path::Path>>(path: P) -> Error {
    Error::new(Kind::Write(String::from(
        path.as_ref().to_str().unwrap_or("NULL"),
    )))
}

pub fn read<P: AsRef<std::path::Path>>(path: P) -> Error {
    Error::new(Kind::Read(String::from(
        path.as_ref().to_str().unwrap_or("NULL"),
    )))
}

pub fn serialize<E>(_: E) -> Error {
    Error::new(Kind::Serialize)
}

pub fn deserialize<E>(_: E) -> Error {
    Error::new(Kind::Deserialize)
}

pub fn parse<E>(_: E) -> Error {
    Error::new(Kind::Parse)
}

pub fn browser(url: String) -> Error {
    Error::new(Kind::Browser(url))
}
