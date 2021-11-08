use std::net::{Ipv4Addr, SocketAddrV4};

#[derive(Debug)]
pub struct Server {
    address: SocketAddrV4,
}

impl Server {
    pub fn new(address: [u8; 4], port: u16) -> Self {
        let address = SocketAddrV4::new(
            Ipv4Addr::new(address[0], address[1], address[2], address[3]),
            port,
        );

        Self { address }
    }
}
