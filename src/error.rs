pub struct Error {
    component: String,
    message: String,
}

pub fn new<E: std::string::ToString>(component: &str, err: E) -> Error {
    Error {
        component: String::from(component),
        message: err.to_string(),
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(fmt, "[{}] {}", &self.component, &self.message)
    }
}
