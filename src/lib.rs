use error::HttpInternalError;

pub mod body;
pub mod client;
pub mod error;
pub mod header_item;
pub mod header_map;
pub mod http_item;
pub mod http_status;
pub mod method;
pub mod pool;
pub mod request;
pub mod response;
pub mod route;
pub mod server;
pub mod url;

type Result<T> = std::result::Result<T, HttpInternalError>;
