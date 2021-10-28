use std::{
    io::{BufReader, Read},
    net::TcpStream,
    str::FromStr,
};

use crate::{error::HttpError, header_map::HeaderMap, Result};

pub trait HeaderItem {
    fn header_map(&self) -> &HeaderMap;

    fn from_stream(buf_stream: &mut BufReader<TcpStream>) -> Result<Self>
    where
        Self: FromStr,
        HttpError: From<<Self as FromStr>::Err>,
    {
        let mut header_str = String::new();

        while !header_str.ends_with("\r\n\r\n") {
            buf_stream
                .by_ref()
                .take(1)
                .read_to_string(&mut header_str)?;
        }

        let item = Self::from_str(&header_str)?;

        Ok(item)
    }
}
