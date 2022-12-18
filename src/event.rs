use std::io;
use std::net::TcpStream;

use crate::connection_kind::ConnectionKind;

#[derive(Debug)]
pub enum Event {
    Connection(ConnectionKind, io::Result<TcpStream>),
}
