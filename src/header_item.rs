use std::{
    io::{BufReader, Read},
    net::TcpStream,
    str::FromStr,
};

use crate::{error::HttpInternalError, header_map::HeaderMap, Result};

pub trait HeaderItem {
    fn header_map(&self) -> &HeaderMap;

    fn from_stream(buf_stream: &mut BufReader<TcpStream>) -> Result<Self>
    where
        Self: FromStr,
        HttpInternalError: From<<Self as FromStr>::Err>,
    {
        let mut header_buf = Vec::new();

        while !header_buf.ends_with(&[13, 10, 13, 10]) {
            let r = buf_stream.by_ref().take(1).read_to_end(&mut header_buf)?;

            if r == 0 {
                return Err(HttpInternalError::DataTimeout);
            }
        }

        let header_str = std::str::from_utf8(&header_buf)?;

        let item = Self::from_str(header_str)?;

        Ok(item)
    }
}
