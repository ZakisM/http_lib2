use crate::{convert_error, http_status::HttpStatus};

#[derive(Debug, Eq, PartialEq)]
pub struct HttpError {
    message: String,
    status: HttpStatus,
}

impl std::fmt::Display for HttpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for HttpError {}

impl HttpError {
    pub fn new<T: AsRef<str>>(message: T, status: HttpStatus) -> Self {
        Self {
            message: message.as_ref().to_owned(),
            status,
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum HttpInternalError {
    ConnectionTimeout,
    DataTimeout,
    Other(String),
}

impl HttpInternalError {
    pub fn new<T: AsRef<str>>(message: T) -> Self {
        Self::Other(message.as_ref().to_owned())
    }
}

impl std::fmt::Display for HttpInternalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            HttpInternalError::ConnectionTimeout => "Connection timed out",
            HttpInternalError::DataTimeout => "Data timed out.",
            HttpInternalError::Other(m) => m,
        };

        write!(f, "{}", message)
    }
}

impl std::error::Error for HttpInternalError {}

impl From<std::io::Error> for HttpInternalError {
    fn from(e: std::io::Error) -> Self {
        match e.kind() {
            std::io::ErrorKind::WouldBlock => Self::ConnectionTimeout,
            _ => Self::new(e.to_string()),
        }
    }
}

convert_error!(std::num::ParseIntError);
convert_error!(std::str::Utf8Error);
convert_error!(std::string::FromUtf8Error);
convert_error!(std::num::TryFromIntError);
convert_error!(std::env::VarError);

#[macro_export]
macro_rules! convert_error {
    ($err:path) => {
        impl From<$err> for HttpInternalError {
            fn from(e: $err) -> Self {
                Self::new(e.to_string())
            }
        }
    };
}
