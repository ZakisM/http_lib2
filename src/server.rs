use std::{
    collections::HashMap,
    io::{BufReader, BufWriter, Read, Write},
    net::{Ipv4Addr, SocketAddrV4, TcpListener, TcpStream},
    sync::Arc,
    time::Duration,
};

use crate::{
    error::HttpError,
    http_item::HttpItem,
    http_status::HttpStatus,
    make_handler,
    method::Method,
    pool::ThreadPool,
    request::Request,
    response::{HttpResponse, ResponseBuilder},
    Result,
};

type HandlerFn = dyn Fn(Request) -> Box<dyn HttpResponse + Send + Sync> + Send + Sync;

type Routes = HashMap<&'static str, HashMap<Method, Box<HandlerFn>>>;

pub struct Server {
    address: SocketAddrV4,
    routes: Routes,
}

impl std::fmt::Debug for Server {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Server")
            .field("address", &self.address)
            .finish()
    }
}

#[derive(Debug)]
pub struct Route<'a> {
    server: &'a mut Server,
    uri: &'static str,
}

impl<'a> Route<'a> {
    pub fn new(server: &'a mut Server, uri: &'static str) -> Self {
        Self { server, uri }
    }

    make_handler!(get, Method::GET);
    make_handler!(head, Method::HEAD);
    make_handler!(post, Method::POST);
    make_handler!(put, Method::PUT);
    make_handler!(delete, Method::DELETE);
    make_handler!(connect, Method::CONNECT);
    make_handler!(options, Method::OPTIONS);
    make_handler!(trace, Method::TRACE);
    make_handler!(patch, Method::PATCH);
}

impl Server {
    pub fn new(address: [u8; 4], port: u16) -> Self {
        let address = SocketAddrV4::new(
            Ipv4Addr::new(address[0], address[1], address[2], address[3]),
            port,
        );

        let routes = HashMap::new();

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
        routes: &Routes,
    ) -> Result<()> {
        let stream = stream?;

        stream.set_nodelay(true)?;
        stream.set_read_timeout(Some(Duration::from_secs(2)))?;
        stream.set_write_timeout(Some(Duration::from_secs(2)))?;

        let req_s = stream;
        let res_s = req_s.try_clone()?;

        let mut req_buf = BufReader::new(req_s);
        let mut res_buf = BufWriter::new(res_s);

        loop {
            match Request::from_stream(req_buf.by_ref()) {
                Ok(req) => {
                    let response = if let Some(all_handlers) = routes.get(req.header.uri.as_str()) {
                        if let Some(handler) = all_handlers.get(&req.header.method) {
                            let res = (handler)(req);

                            res.into_response()
                        } else {
                            ResponseBuilder::new()
                                .status(HttpStatus::MethodNotAllowed)
                                .build()
                        }
                    } else {
                        ResponseBuilder::new().status(HttpStatus::NotFound).build()
                    };

                    response.write_to(res_buf.by_ref())?;
                }
                Err(e) => {
                    if e != HttpError::DataTimeout {
                        eprintln!("{}", e);
                    }

                    break;
                }
            }
        }

        Ok(())
    }
}

#[macro_export]
macro_rules! make_handler {
    ($name: ident, $method: path) => {
        pub fn $name<F, R>(self, handler: F) -> Self
        where
            F: Fn(Request) -> R + Send + Sync + 'static,
            R: HttpResponse + Send + Sync + 'static,
        {
            let r = self
                .server
                .routes
                .entry(self.uri)
                .or_insert_with(HashMap::new);

            let h =
                Box::new(move |req| Box::new(handler(req)) as Box<dyn HttpResponse + Send + Sync>);

            r.insert($method, h);

            self
        }
    };
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
