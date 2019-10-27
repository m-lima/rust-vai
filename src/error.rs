pub struct Error {
    component: String,
    message: String,
}

pub fn new<C: std::string::ToString, E: std::string::ToString>(component: C, err: E) -> Error {
    Error {
        component: component.to_string(),
        message: err.to_string(),
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(fmt, "[{}] {}", &self.component, &self.message)
    }
}
