use std::{
    io::{BufReader, BufWriter, Read, Write},
    net::{Ipv4Addr, SocketAddrV4, TcpListener, TcpStream},
    sync::Arc,
    time::Duration,
};

use crate::{
    error::HttpInternalError,
    http_item::HttpItem,
    http_status::HttpStatus,
    pool::ThreadPool,
    request::{Request, ServerRequest},
    response::ResponseBuilder,
    route::{Route, RouteKey, RouteMap},
    Result,
};

pub struct Server {
    address: SocketAddrV4,
    pub(crate) routes: RouteMap,
}

impl std::fmt::Debug for Server {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Server")
            .field("address", &self.address)
            .finish()
    }
}

impl Server {
    pub fn new(address: [u8; 4], port: u16) -> Self {
        let address = SocketAddrV4::new(
            Ipv4Addr::new(address[0], address[1], address[2], address[3]),
            port,
        );

        let routes = RouteMap::new();

        Self { address, routes }
    }

    pub fn at(&mut self, location: &'static str) -> Route {
        Route::new(self, location)
    }

    pub fn start(self) -> Result<()> {
        let listener = TcpListener::bind(self.address)?;

        let mut pool = ThreadPool::new()?;

        let routes = Arc::new(self.routes);

        for stream in listener.incoming() {
            let routes = routes.clone();

            pool.spawn(move || {
                if let Err(e) = Self::handle_connection(stream, &*routes) {
                    eprintln!("{}", e);
                }
            });
        }

        Ok(())
    }

    fn handle_connection(
        stream: std::result::Result<TcpStream, std::io::Error>,
        routes: &RouteMap,
    ) -> Result<()> {
        let stream = stream?;

        stream.set_nodelay(true)?;
        stream.set_read_timeout(Some(Duration::from_secs(2)))?;
        stream.set_write_timeout(Some(Duration::from_secs(2)))?;

        let peer_address = stream.peer_addr()?;

        let read_s = stream;
        let write_s = read_s.try_clone()?;

        let mut read_buf = BufReader::new(read_s);
        let mut write_buf = BufWriter::new(write_s);

        loop {
            match Request::from_stream(read_buf.by_ref()) {
                Ok(req) => {
                    let uri = RouteKey(req.header.uri.to_owned());

                    let response = if let Some((route_key, route_handlers)) = routes.get(&uri) {
                        if let Some(handler) = route_handlers.get(&req.header.method) {
                            let server_req =
                                ServerRequest::new(route_key.clone(), req, peer_address);
                            let res = (handler)(server_req);

                            res.into_response()
                        } else {
                            ResponseBuilder::new()
                                .status(HttpStatus::MethodNotAllowed)
                                .build()
                        }
                    } else {
                        ResponseBuilder::new().status(HttpStatus::NotFound).build()
                    };

                    response.write_to(write_buf.by_ref())?;
                }
                Err(e) => {
                    if e != HttpInternalError::DataTimeout
                        && e != HttpInternalError::ConnectionTimeout
                    {
                        eprintln!("{}", e);
                    }

                    break;
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::response::ResponseBuilder;

    use super::*;

    #[test]
    fn test_server() {
        let mut s = Server::new([127, 0, 0, 1], 1234);

        s.at("/hello")
            .get(|_| ResponseBuilder::new().build())
            .connect(|_| ResponseBuilder::new().build());
    }
}
