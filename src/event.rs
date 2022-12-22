use std::io;
use std::net::TcpStream;

use strum_macros::Display;

use crate::connection_kind::ConnectionKind;
use crate::message::Message;

#[derive(Debug, Display)]
pub enum Event {
    Connection(ConnectionKind, io::Result<TcpStream>),
    Publish(Message),
}
