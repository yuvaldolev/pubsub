use std::io;
use std::net::TcpStream;

use strum_macros::Display;
use uuid::Uuid;

use crate::connection_kind::ConnectionKind;
use crate::message::Message;
use crate::subscription_request::SubscriptionRequest;

#[derive(Debug, Display)]
pub enum Event {
    Connection(ConnectionKind, io::Result<TcpStream>),
    Publish(Message),
    SubscriptionRequest(Uuid, SubscriptionRequest),
    Termination,
}
