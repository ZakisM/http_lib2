use std::io::{BufReader, Read};

use crate::{error::HttpError, Result};

#[derive(Debug)]
pub struct Body {
    pub contents: Vec<u8>,
}

impl Body {
    pub fn new<T: AsRef<[u8]>>(bytes: T) -> Self {
        Self {
            contents: bytes.as_ref().to_vec(),
        }
    }

    pub fn from_fixed_length<R: Read>(reader: R, content_length: usize) -> Result<Self> {
        let buf_reader = BufReader::new(reader);

        let mut contents = Vec::with_capacity(content_length);

        let r = buf_reader
            .take(content_length.try_into()?)
            .read_to_end(&mut contents)?;

        if r != content_length {
            Err(HttpError::new(format!(
                "Failed to read all of the fixed content length, expected: {} but received: {}.",
                content_length, r
            )))
        } else {
            Ok(Self { contents })
        }
    }

    pub fn from_chunked_encoding<R: Read>(reader: R) -> Result<Self> {
        let mut buf_reader = BufReader::new(reader);

        let mut contents = Vec::new();

        let mut temp_contents = Vec::new();
        let mut chunk_length = None;

        let mut sink = [0; 2];

        let mut expected_length = 0;

        loop {
            if let Some(length) = chunk_length.take() {
                let r = buf_reader
                    .by_ref()
                    .take(length)
                    .read_to_end(&mut contents)?;

                //read last two bytes which should be CRLF
                buf_reader.by_ref().read_exact(&mut sink)?;

                if length == 0 {
                    break;
                }

                expected_length += length;

                if r == 0 {
                    break;
                }

                temp_contents.clear();
            } else {
                let r = buf_reader
                    .by_ref()
                    .take(1)
                    .read_to_end(&mut temp_contents)?;

                if r == 0 {
                    break;
                }
            }

            if temp_contents.ends_with(&[13, 10]) {
                let chunk_len_str = std::str::from_utf8(&temp_contents[..temp_contents.len() - 2])?;

                let chunk_len = u64::from_str_radix(chunk_len_str, 16)?;
                chunk_length = Some(chunk_len);
            } else if temp_contents.ends_with(&[13, 10, 13, 10]) {
                break;
            }
        }

        let contents_len = contents.len().try_into()?;

        if expected_length != contents_len {
            Err(HttpError::new(format!(
                "Failed to read all of the chunked content length, expected: {} but received: {}.",
                expected_length, contents_len
            )))
        } else {
            Ok(Self { contents })
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::Body;

    #[test]
    fn from_fixed_length() {
        let bytes_str = "Hello World!";
        let bytes = Cursor::new(bytes_str);

        let body = Body::from_fixed_length(bytes, 12).unwrap();

        assert_eq!(body.contents, b"Hello World!");
    }

    #[test]
    fn from_chunked_encoding() {
        let bytes_str = "7\r\nMozilla\r\n9\r\nDeveloper\r\n7\r\nNetwork\r\n0\r\n\r\n";
        let bytes = Cursor::new(bytes_str);

        let body = Body::from_chunked_encoding(bytes).unwrap();

        assert_eq!(
            "MozillaDeveloperNetwork",
            std::str::from_utf8(&body.contents).unwrap()
        );
    }
}
