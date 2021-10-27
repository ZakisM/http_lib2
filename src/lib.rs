use std::error::Error;

pub mod error;
pub mod header_map;
pub mod method;
pub mod request;
pub mod response;

type Result<T> = std::result::Result<T, Box<dyn Error>>;
