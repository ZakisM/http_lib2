use crate::convert_error;

#[derive(Debug, Eq, PartialEq)]
pub enum HttpError {
    ConnectionClosed,
    DataTimeout,
    Other(String),
}

impl HttpError {
    pub fn new<T: AsRef<str>>(message: T) -> Self {
        Self::Other(message.as_ref().to_owned())
    }
}

impl std::fmt::Display for HttpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            HttpError::ConnectionClosed => "Connection was closed.",
            HttpError::DataTimeout => "Data timed out.",
            HttpError::Other(m) => m,
        };

        write!(f, "{}", message)
    }
}

impl std::error::Error for HttpError {}

convert_error!(std::io::Error);
convert_error!(std::num::ParseIntError);
convert_error!(std::str::Utf8Error);
convert_error!(std::num::TryFromIntError);

#[macro_export]
macro_rules! convert_error {
    ($err:path) => {
        impl From<$err> for HttpError {
            fn from(e: $err) -> Self {
                Self::new(e.to_string())
            }
        }
    };
}
