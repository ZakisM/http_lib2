use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::str::FromStr;

use crate::HttpInternalError;

pub trait IntoUrl {}

#[derive(Debug)]
pub struct Url {
    pub address: SocketAddr,
    pub uri: String,
}

impl FromStr for Url {
    type Err = HttpInternalError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split('/');

        let scheme = parts
            .next()
            .ok_or_else(|| HttpInternalError::new("Missing HTTP scheme from URL."))?;

        let allowed_scheme = "http:";

        if !scheme.eq_ignore_ascii_case(allowed_scheme) {
            return Err(HttpInternalError::new(format!(
                "Expected URL to begin with '{}'",
                allowed_scheme
            )));
        }

        let address = parts.nth(1).ok_or_else(|| {
            HttpInternalError::new("Invalid address passed, expected Ipv4 Address.")
        })?;

        let mut uri = &s[scheme.len() + address.len() + 2..];

        if uri.is_empty() {
            uri = "/";
        }

        let address = SocketAddr::from_str(address)
            .map_err(|e| HttpInternalError::new(format!("Invalid address passed: {}", e)))?;

        let uri = uri.to_owned();

        Ok(Self { address, uri })
    }
}

#[cfg(test)]
mod tests {
    use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
    use std::str::FromStr;

    use crate::url::Url;

    #[test]
    fn test_from_str() {
        let url = Url::from_str("http://127.0.0.1:1234/hello_world/123").unwrap();

        assert_eq!(
            url.address,
            SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 1234))
        );

        assert_eq!(&url.uri, "/hello_world/123");

        let second_url = Url::from_str("http://0.0.0.0:65535").unwrap();

        assert_eq!(
            second_url.address,
            SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 65535))
        );

        assert_eq!(&second_url.uri, "/");
    }
}
