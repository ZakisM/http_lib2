use std::io::{BufReader, BufWriter};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

use crate::http_item::HttpItem;
use crate::method::Method;
use crate::request::RequestBuilder;
use crate::response::Response;
use crate::url::IntoUrl;
use crate::Result;

#[derive(Debug, Default)]
pub struct Client {}

impl Client {
    pub fn new() -> Self {
        Client {}
    }

    // pub fn get<U: IntoUrl>(&self, url: U) -> RequestBuilder {}

    // pub fn get<A: ToSocketAddrs>(&self, address: A) -> Result<Response> {
    //     let (mut read_buf, mut write_buf) = Self::setup_connection(address)?;
    //
    //     let request = RequestBuilder::new().method(Method::GET).build();
    //
    //     request.write_to(&mut write_buf)?;
    //
    //     let response = Response::from_stream(&mut read_buf)?;
    //
    //     Ok(response)
    // }

    // pub fn send(&self) -> Result<Response> {
    //     let (mut read_buf, mut write_buf) = Self::setup_connection(address)?;
    //
    //     let
    // }

    fn setup_connection<A: ToSocketAddrs>(
        address: A,
    ) -> Result<(BufReader<TcpStream>, BufWriter<TcpStream>)> {
        let stream = TcpStream::connect(address)?;

        stream.set_nodelay(true)?;
        stream.set_read_timeout(Some(Duration::from_secs(2)))?;
        stream.set_write_timeout(Some(Duration::from_secs(2)))?;

        let read_s = stream;
        let write_s = read_s.try_clone()?;

        let read_buf = BufReader::new(read_s);
        let write_buf = BufWriter::new(write_s);

        Ok((read_buf, write_buf))
    }
}

#[cfg(test)]
mod tests {
    use crate::client::Client;

    #[test]
    fn test_get() {
        let client = Client::new();

        // let res = client.get("127.0.0.1:1234").unwrap().text().unwrap();
        //
        // dbg!(&res);
    }
}
