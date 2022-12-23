use std::net::TcpStream;
use std::thread::{self, JoinHandle};

use crossbeam::channel::{self, Receiver, Sender};

use crate::event::Event;
use crate::message::Message;

pub struct SubscriberHandler {
    message_sender: Sender<Message>,
    subscriber_thread: Option<JoinHandle<()>>,
}

impl SubscriberHandler {
    pub fn new(stream: TcpStream, event_sender: Sender<Event>) -> Self {
        let (message_sender, message_receiver): (Sender<Message>, Receiver<Message>) =
            channel::unbounded();

        let subscriber_thread =
            thread::spawn(move || Self::handle_subscriber(stream, event_sender, message_receiver));
        Self {
            message_sender,
            subscriber_thread: Some(subscriber_thread),
        }
    }

    fn handle_subscriber(
        mut stream: TcpStream,
        event_sender: Sender<Event>,
        message_receiver: Receiver<Message>,
    ) {
        log::debug!("Handling subscriber: [{:?}]", stream);
    }
}
