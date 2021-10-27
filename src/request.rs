use std::str::FromStr;

use crate::error::HttpError;
use crate::header_map::HeaderMap;
use crate::method::Method;

#[derive(Debug)]
pub struct RequestHeader {
    pub method: Method,
    pub path: String,
    pub version: f32,
    pub headers: HeaderMap,
}

impl FromStr for RequestHeader {
    type Err = HttpError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let mut lines = s.lines();

        let mut req_line = lines
            .next()
            .map(|l| l.split_whitespace())
            .ok_or_else(|| HttpError::new("Failed to read request line"))?;

        let method = req_line
            .next()
            .and_then(|r| Method::from_str(r).ok())
            .ok_or_else(|| HttpError::new("Failed to read request method"))?;

        let path = req_line
            .next()
            .ok_or_else(|| HttpError::new("Failed to read request path"))?;

        let version = req_line
            .next()
            .and_then(|v| v.strip_prefix("HTTP/"))
            .and_then(|v| v.parse::<f32>().ok())
            .ok_or_else(|| HttpError::new("Failed to read request version"))?;

        let headers = HeaderMap::from_lines(lines);

        Ok(Self {
            method,
            path: path.to_owned(),
            version,
            headers,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::method::Method;

    use super::RequestHeader;

    #[test]
    fn read_request() {
        let sample_request = r#"GET /test HTTP/1.1
        Host: www.example.com
        User-Agent: Mozilla/5.0
        Accept: text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8
        Accept-Language: en-GB,en;q=0.5
        Accept-Encoding: gzip, deflate, br
        Connection: keep-alive"#;

        let request = RequestHeader::from_str(sample_request).unwrap();

        assert_eq!(request.method, Method::GET);
        assert_eq!(request.path, "/test");
        assert_eq!(request.version, 1.1);

        let headers = &request.headers;

        assert_eq!(headers.get_by_key("host"), Some("www.example.com"));
        assert_eq!(headers.get_by_key("user-agent"), Some("Mozilla/5.0"));
        assert_eq!(
            headers.get_by_key("accept"),
            Some("text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8")
        );
        assert_eq!(
            headers.get_by_key("accept-language"),
            Some("en-GB,en;q=0.5")
        );
        assert_eq!(
            headers.get_by_key("accept-encoding"),
            Some("gzip, deflate, br")
        );
        assert_eq!(headers.get_by_key("connection"), Some("keep-alive"));
    }
}
