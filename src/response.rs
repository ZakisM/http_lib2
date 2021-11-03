use std::io::Write;
use std::str::FromStr;

use crate::body::Body;
use crate::error::HttpError;
use crate::header_item::HeaderItem;
use crate::header_map::HeaderMap;
use crate::http_item::HttpItem;
use crate::Result;

#[derive(Debug, Default)]
pub struct ResponseBuilder {
    header: ResponseHeader,
    body: Option<Body>,
}

impl ResponseBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn version(mut self, version: f32) -> Self {
        self.header.version = version;
        self
    }

    pub fn status_code(mut self, code: u16) -> Self {
        self.header.status_code = code;
        self
    }

    pub fn reason_phrase(mut self, phrase: &str) -> Self {
        self.header.reason_phrase = phrase.to_owned();
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

    pub fn build(self) -> Response {
        Response {
            header: self.header,
            body: self.body,
        }
    }
}

#[derive(Debug)]
pub struct Response {
    pub header: ResponseHeader,
    pub body: Option<Body>,
}

impl HttpItem for Response {
    type Header = ResponseHeader;

    fn from_header_body(header: Self::Header, body: Option<Body>) -> Self {
        Self { header, body }
    }
}

impl Response {
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        let mut bytes = Vec::new();

        write!(
            bytes,
            "HTTP/{} {} {}\r\n",
            self.header.version, self.header.status_code, self.header.reason_phrase
        )?;

        self.header.header_map.write_to(&mut bytes)?;

        write!(bytes, "\r\n")?;

        if let Some(body) = &self.body {
            bytes.extend_from_slice(&body.contents);
        }

        Ok(bytes)
    }

    pub fn write_to<T: Write>(&self, writer: &mut T) -> Result<()> {
        let bytes = self.to_bytes()?;

        writer.write_all(&bytes)?;
        writer.flush()?;

        Ok(())
    }
}

#[derive(Debug)]
pub struct ResponseHeader {
    pub version: f32,
    pub status_code: u16,
    pub reason_phrase: String,
    header_map: HeaderMap,
}

impl std::default::Default for ResponseHeader {
    fn default() -> Self {
        Self {
            version: 1.1,
            status_code: 200,
            reason_phrase: "OK".to_owned(),
            header_map: HeaderMap::default(),
        }
    }
}

impl FromStr for ResponseHeader {
    type Err = HttpError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let mut lines = s.lines();

        let mut status_line = lines
            .next()
            .map(|l| l.split_whitespace())
            .ok_or_else(|| HttpError::new("Failed to read response status line"))?;

        let version = status_line
            .next()
            .and_then(|v| v.strip_prefix("HTTP/"))
            .and_then(|v| v.parse::<f32>().ok())
            .ok_or_else(|| HttpError::new("Failed to read response version"))?;

        let status_code = status_line
            .next()
            .and_then(|r| r.parse::<u16>().ok())
            .ok_or_else(|| HttpError::new("Failed to read response status code"))?;

        let reason_phrase = status_line
            .next()
            .map(|r| r.to_owned())
            .ok_or_else(|| HttpError::new("Failed to read response reason phrase"))?;

        let headers = HeaderMap::from_lines(lines);

        Ok(Self {
            version,
            status_code,
            reason_phrase,
            header_map: headers,
        })
    }
}

impl HeaderItem for ResponseHeader {
    fn header_map(&self) -> &HeaderMap {
        &self.header_map
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::ResponseHeader;

    #[test]
    fn read_response() {
        let sample_response = r#"HTTP/1.1 200 OK
        Date: Mon, 23 May 2005 22:38:34 GMT
        Content-Type: text/html; charset=UTF-8
        Last-Modified: Wed, 08 Jan 2003 23:11:55 GMT
        Server: Apache/1.3.3.7 (Unix) (Red-Hat/Linux)
        ETag: "3f80f-1b6-3e1cb03b"
        Accept-Ranges: bytes
        Connection: close"#;

        let response = ResponseHeader::from_str(sample_response).unwrap();

        assert_eq!(response.version, 1.1);
        assert_eq!(response.status_code, 200);
        assert_eq!(response.reason_phrase, "OK".to_owned());

        let headers = &response.header_map;

        assert_eq!(
            headers.get_by_str_key("date"),
            Some("Mon, 23 May 2005 22:38:34 GMT")
        );
        assert_eq!(
            headers.get_by_str_key("content-type"),
            Some("text/html; charset=UTF-8")
        );
        assert_eq!(
            headers.get_by_str_key("last-modified"),
            Some("Wed, 08 Jan 2003 23:11:55 GMT")
        );
        assert_eq!(
            headers.get_by_str_key("server"),
            Some("Apache/1.3.3.7 (Unix) (Red-Hat/Linux)")
        );
        assert_eq!(
            headers.get_by_str_key("etag"),
            Some("\"3f80f-1b6-3e1cb03b\"")
        );
        assert_eq!(headers.get_by_str_key("accept-ranges"), Some("bytes"));
        assert_eq!(headers.get_by_str_key("connection"), Some("close"));
    }
}
