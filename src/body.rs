use std::io::{BufReader, Read};

use crate::Result;

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

        buf_reader
            .take(content_length.try_into()?)
            .read_to_end(&mut contents)?;

        Ok(Self { contents })
    }

    pub fn from_chunked_encoding<R: Read>(reader: R) -> Result<Self> {
        let mut buf_reader = BufReader::new(reader);

        let mut contents = Vec::new();

        let mut temp_contents = Vec::new();
        let mut chunk_length = None;

        let mut sink = [0; 2];

        loop {
            if let Some(length) = chunk_length.take() {
                let r = buf_reader
                    .by_ref()
                    .take(length)
                    .read_to_end(&mut contents)?;

                //read last two bytes
                buf_reader.by_ref().read_exact(&mut sink)?;

                if length == 0 || r == 0 {
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

        Ok(Self { contents })
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;

    use super::Body;

    #[test]
    fn from_fixed_length() {
        //In test we are reading from a file rather than a stream
        let f = File::open("./test_files/fixed_length_bytes").unwrap();

        let body = Body::from_fixed_length(f, 12).unwrap();

        assert_eq!(body.contents, b"Hello World!");
    }

    #[test]
    fn from_chunked_encoding() {
        //In test we are reading from a file rather than a stream
        let f = File::open("./test_files/chunked_encoding_bytes").unwrap();

        let body = Body::from_chunked_encoding(f).unwrap();

        assert_eq!(
            "MozillaDeveloperNetwork",
            std::str::from_utf8(&body.contents).unwrap()
        );
    }
}
