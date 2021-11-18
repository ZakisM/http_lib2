use crate::{body::Body, error::HttpInternalError, header_item::HeaderItem, Result};
use std::{
    io::{BufReader, Read},
    net::TcpStream,
    str::FromStr,
};

pub trait HttpItem {
    type Header: HeaderItem;

    fn from_header_body(header: Self::Header, body: Body) -> Self;

    fn from_stream(buf_stream: &mut BufReader<TcpStream>) -> Result<Self>
    where
        Self::Header: FromStr,
        HttpInternalError: From<<Self::Header as FromStr>::Err>,
        Self: Sized,
    {
        let header = Self::Header::from_stream(buf_stream.by_ref())?;

        let header_map = header.header_map();

        let body =
            if let Some(content_length) = header_map.get_by_str_key_as::<usize>("content-length") {
                Body::from_fixed_length(buf_stream, content_length)?
            } else if header_map.contains_by_str_key_value("transfer-encoding", "chunked") {
                Body::from_chunked_encoding(buf_stream)?
            } else {
                Body::empty()
            };

        let res = Self::from_header_body(header, body);

        Ok(res)
    }
}
