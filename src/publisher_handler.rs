use std::net::TcpStream;
use std::thread::{self, JoinHandle};

use crossbeam::channel::Sender;

use crate::event::Event;
use crate::message::Message;

pub struct PublisherHandler {
    publisher_thread: Option<JoinHandle<()>>,
}

impl PublisherHandler {
    pub fn new(stream: TcpStream, event_sender: Sender<Event>) -> Self {
        let publisher_thread = thread::spawn(move || Self::handle_publisher(stream, event_sender));
        Self {
            publisher_thread: Some(publisher_thread),
        }
    }

    fn handle_publisher(mut stream: TcpStream, event_sender: Sender<Event>) {
        // TODO: Graceful shutdown.
        // TODO: Proper error handling (including disconnection).
        loop {
            match Message::read(&mut stream) {
                Ok(message) => event_sender.send(Event::Publish(message)).unwrap(),
                Err(e) => {
                    log::error!("Error receiving message: [{}]", e);
                    break;
                }
            };
        }
    }
}

impl Drop for PublisherHandler {
    fn drop(&mut self) {
        if let Some(thread) = self.publisher_thread.take() {
            thread.join().unwrap();
        }
    }
}
