use std::net::ToSocketAddrs;

pub struct BackgroundTcpListener<T: ToSocketAddrs> {
    address: T,
}

impl<T: ToSocketAddrs> BackgroundTcpListener<T> {
    pub fn new(address: T) -> Self {
        Self { address }
    }
}
