use std::str::FromStr;

use crate::body::Body;
use crate::error::HttpError;
use crate::header_item::HeaderItem;
use crate::header_map::HeaderMap;
use crate::http_item::HttpItem;
use crate::method::Method;

#[derive(Debug, Default)]
pub struct RequestBuilder {
    header: RequestHeader,
    body: Option<Body>,
}

impl RequestBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn method(mut self, method: Method) -> Self {
        self.header.method = method;
        self
    }

    pub fn uri(mut self, uri: &str) -> Self {
        self.header.uri = uri.to_owned();
        self
    }

    pub fn version(mut self, version: f32) -> Self {
        self.header.version = version;
        self
    }

    pub fn header_map(mut self, header_map: HeaderMap) -> Self {
        self.header.header_map = header_map;
        self
    }

    pub fn insert_header_key_val(mut self, key: &str, val: &str) -> Self {
        self.header.header_map.insert_by_str_key_value(key, val);
        self
    }

    pub fn body<T: AsRef<[u8]>>(mut self, body: T) -> Self {
        let body_len = body.as_ref().len();

        self.header
            .header_map
            .insert_by_str_key_value("Content-Length", &body_len.to_string());

        self.body = Some(Body::new(body));
        self
    }

    pub fn build(self) -> Request {
        Request {
            header: self.header,
            body: self.body.unwrap_or_else(Body::empty),
        }
    }
}

#[derive(Debug)]
pub struct Request {
    pub header: RequestHeader,
    pub body: Body,
}

impl HttpItem for Request {
    type Header = RequestHeader;

    fn from_header_body(header: Self::Header, body: Body) -> Self {
        Self { header, body }
    }
}

#[derive(Debug)]
pub struct RequestHeader {
    pub method: Method,
    pub uri: String,
    pub version: f32,
    header_map: HeaderMap,
}

impl std::default::Default for RequestHeader {
    fn default() -> Self {
        Self {
            method: Method::GET,
            uri: "/".to_owned(),
            version: 1.1,
            header_map: HeaderMap::default(),
        }
    }
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
            uri: path.to_owned(),
            version,
            header_map: headers,
        })
    }
}

impl HeaderItem for RequestHeader {
    fn header_map(&self) -> &HeaderMap {
        &self.header_map
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
        assert_eq!(request.uri, "/test");
        assert_eq!(request.version, 1.1);

        let headers = &request.header_map;

        assert_eq!(headers.get_by_str_key("host"), Some("www.example.com"));
        assert_eq!(headers.get_by_str_key("user-agent"), Some("Mozilla/5.0"));
        assert_eq!(
            headers.get_by_str_key("accept"),
            Some("text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8")
        );
        assert_eq!(
            headers.get_by_str_key("accept-language"),
            Some("en-GB,en;q=0.5")
        );
        assert_eq!(
            headers.get_by_str_key("accept-encoding"),
            Some("gzip, deflate, br")
        );
        assert_eq!(headers.get_by_str_key("connection"), Some("keep-alive"));
    }
}
