use std::collections::HashMap;

use crate::{
    make_handler, method::Method, request::ServerRequest, response::HttpResponse, server::Server,
};

type HandlerFn = dyn Fn(ServerRequest) -> Box<dyn HttpResponse + Send + Sync> + Send + Sync;
type RouteHandlers = HashMap<Method, Box<HandlerFn>>;

#[derive(Default)]
pub struct RouteMap {
    pub all_routes: Vec<(RouteKey, RouteHandlers)>,
}

impl RouteMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, key: &RouteKey) -> Option<&(RouteKey, RouteHandlers)> {
        self.all_routes.iter().find(|(k, _)| k == key)
    }

    pub fn get_mut(&mut self, key: &RouteKey) -> Option<&mut RouteHandlers> {
        self.all_routes
            .iter_mut()
            .find(|(k, _)| k == key)
            .map(|(_, h)| h)
    }

    pub fn insert(&mut self, key: RouteKey, method: Method, handler: Box<HandlerFn>) {
        let mut h = HashMap::with_capacity(1);
        h.insert(method, handler);

        self.all_routes.push((key, h));
    }
}

impl std::fmt::Debug for RouteMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RouteMap").finish()
    }
}

#[derive(Clone, Debug, Eq)]
pub struct RouteKey(pub String);

pub fn named_path_filter(s: &str) -> bool {
    s.starts_with('{') && s.ends_with('}')
}

impl std::cmp::PartialEq for RouteKey {
    fn eq(&self, other: &Self) -> bool {
        let self_split = self.0.split('/');
        let other_split = other.0.split('/');

        if self_split.clone().count() != other_split.clone().count() {
            false
        } else {
            self_split
                .zip(other_split)
                .filter(|(s, o)| !named_path_filter(s) && !named_path_filter(o))
                .all(|(s, o)| s.eq_ignore_ascii_case(o))
        }
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

#[macro_export]
macro_rules! make_handler {
    ($name: ident, $method: path) => {
        pub fn $name<F, R>(self, handler: F) -> Self
        where
            F: Fn(ServerRequest) -> R + Send + Sync + 'static,
            R: HttpResponse + Send + Sync + 'static,
        {
            let uri = RouteKey(self.uri.to_owned());

            let h =
                Box::new(move |req| Box::new(handler(req)) as Box<dyn HttpResponse + Send + Sync>);

            if let Some(handlers) = self.server.routes.get_mut(&uri) {
                handlers.insert($method, h);
            } else {
                self.server.routes.insert(uri, $method, h);
            }

            self
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eq() {
        let route_0 = RouteKey("/".to_owned());
        let route_1 = RouteKey("/hello/{name}".to_owned());
        let route_2 = RouteKey("/hello/{name}/{age}".to_owned());
        let route_3 = RouteKey("/hello".to_owned());

        let request_uri_0 = RouteKey("/".to_owned());
        let request_uri_1 = RouteKey("/hello/Zak".to_owned());
        let request_uri_2 = RouteKey("/hello/Zak/24".to_owned());
        let request_uri_3 = RouteKey("/hello".to_owned());

        assert_eq!(route_0, request_uri_0);
        assert_ne!(route_0, request_uri_1);
        assert_ne!(route_0, request_uri_2);
        assert_ne!(route_0, request_uri_3);

        assert_ne!(route_1, request_uri_0);
        assert_eq!(route_1, request_uri_1);
        assert_ne!(route_1, request_uri_2);
        assert_ne!(route_1, request_uri_3);

        assert_ne!(route_2, request_uri_0);
        assert_ne!(route_2, request_uri_1);
        assert_eq!(route_2, request_uri_2);
        assert_ne!(route_2, request_uri_3);

        assert_ne!(route_3, request_uri_0);
        assert_ne!(route_3, request_uri_1);
        assert_ne!(route_3, request_uri_2);
        assert_eq!(route_3, request_uri_3);
    }
}
