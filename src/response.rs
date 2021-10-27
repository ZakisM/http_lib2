use std::str::FromStr;

use crate::error::HttpError;
use crate::header_map::HeaderMap;

#[derive(Debug)]
pub struct ResponseHeader {
    pub version: f32,
    pub status_code: u16,
    pub reason_phrase: String,
    pub headers: HeaderMap,
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
            headers,
        })
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

        let headers = &response.headers;

        assert_eq!(
            headers.get_by_key("date"),
            Some("Mon, 23 May 2005 22:38:34 GMT")
        );
        assert_eq!(
            headers.get_by_key("content-type"),
            Some("text/html; charset=UTF-8")
        );
        assert_eq!(
            headers.get_by_key("last-modified"),
            Some("Wed, 08 Jan 2003 23:11:55 GMT")
        );
        assert_eq!(
            headers.get_by_key("server"),
            Some("Apache/1.3.3.7 (Unix) (Red-Hat/Linux)")
        );
        assert_eq!(headers.get_by_key("etag"), Some("\"3f80f-1b6-3e1cb03b\""));
        assert_eq!(headers.get_by_key("accept-ranges"), Some("bytes"));
        assert_eq!(headers.get_by_key("connection"), Some("close"));
    }
}
