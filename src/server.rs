use std::{
    collections::HashMap,
    net::{Ipv4Addr, SocketAddrV4},
};

use crate::{method::Method, request::Request, response::Response};

#[derive(Debug)]
pub struct Server<'a> {
    address: SocketAddrV4,
    routes: Vec<Route<'a>>,
}

type Handler = fn(Request) -> Response;

#[derive(Debug)]
pub struct Route<'a> {
    server: &'a mut Server<'a>,
    uri: &'static str,
    handlers: HashMap<Method, Handler>,
}

impl<'a> Route<'a> {
    pub fn new(server: &'a mut Server<'a>, uri: &'static str) -> Self {
        Self {
            server,
            uri,
            handlers: HashMap::new(),
        }
    }

    pub fn get(mut self, handler: Handler) -> Self {
        self.handlers.insert(Method::GET, handler);
        self
    }

    pub fn head(mut self, handler: Handler) -> Self {
        self.handlers.insert(Method::HEAD, handler);
        self
    }

    pub fn post(mut self, handler: Handler) -> Self {
        self.handlers.insert(Method::POST, handler);
        self
    }

    pub fn put(mut self, handler: Handler) -> Self {
        self.handlers.insert(Method::PUT, handler);
        self
    }

    pub fn delete(mut self, handler: Handler) -> Self {
        self.handlers.insert(Method::DELETE, handler);
        self
    }

    pub fn connect(mut self, handler: Handler) -> Self {
        self.handlers.insert(Method::CONNECT, handler);
        self
    }

    pub fn options(mut self, handler: Handler) -> Self {
        self.handlers.insert(Method::OPTIONS, handler);
        self
    }

    pub fn trace(mut self, handler: Handler) -> Self {
        self.handlers.insert(Method::TRACE, handler);
        self
    }

    pub fn patch(mut self, handler: Handler) -> Self {
        self.handlers.insert(Method::PATCH, handler);
        self
    }
}

impl<'a> Server<'a> {
    pub fn new(address: [u8; 4], port: u16) -> Self {
        let address = SocketAddrV4::new(
            Ipv4Addr::new(address[0], address[1], address[2], address[3]),
            port,
        );

        let routes = Vec::new();

        Self { address, routes }
    }

    pub fn at(&'a mut self, location: &'static str) -> Route {
        Route::new(self, location)
    }
}

#[cfg(test)]
mod tests {
    use crate::response::ResponseBuilder;

    use super::*;

    #[test]
    fn my_test() {
        let mut s = Server::new([127, 0, 0, 1], 1234);

        s.at("/hello")
            .get(|_| ResponseBuilder::new().build())
            .connect(|_| ResponseBuilder::new().build());
    }
}
