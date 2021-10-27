#[derive(Debug)]
pub struct HttpError {
    message: String,
}

impl HttpError {
    pub fn new<T: AsRef<str>>(message: T) -> Self {
        Self {
            message: message.as_ref().to_owned(),
        }
    }
}

impl std::fmt::Display for HttpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl<E: std::error::Error> From<E> for HttpError {
    fn from(e: E) -> Self {
        Self {
            message: e.to_string(),
        }
    }
}
