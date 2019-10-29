#[derive(Debug, Clone)]
pub struct Error {
    component: String,
    cause: String,
}

pub fn new<C: std::string::ToString, E: std::string::ToString>(component: C, err: E) -> Error {
    Error {
        component: component.to_string(),
        cause: err.to_string(),
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(fmt, "[{}] {}", &self.component, &self.cause)
    }
}

impl std::error::Error for Error {}
