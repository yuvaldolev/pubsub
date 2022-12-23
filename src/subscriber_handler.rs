use std::net::TcpStream;
use std::thread::{self, JoinHandle};

use crossbeam::channel::{self, Receiver, Sender};
use uuid::Uuid;

use crate::error;
use crate::event::Event;
use crate::message::Message;
use crate::subscription_request::SubscriptionRequest;

pub struct SubscriberHandler {
    message_sender: Sender<Message>,
    subscriber_thread: Option<JoinHandle<()>>,
}

impl SubscriberHandler {
    pub fn new(id: Uuid, stream: TcpStream, event_sender: Sender<Event>) -> Self {
        let (message_sender, message_receiver): (Sender<Message>, Receiver<Message>) =
            channel::unbounded();

        let subscriber_thread = thread::spawn(move || {
            Self::handle_subscriber(id, stream, event_sender, message_receiver)
        });
        Self {
            message_sender,
            subscriber_thread: Some(subscriber_thread),
        }
    }

    pub fn publish(&self, message: Message) -> error::Result<()> {
        self.message_sender.send(message)?;
        Ok(())
    }

    fn handle_subscriber(
        id: Uuid,
        mut stream: TcpStream,
        event_sender: Sender<Event>,
        message_receiver: Receiver<Message>,
    ) {
        // Receive the subscriber's subscription request.
        // TODO: Proper error handling.
        let subscription_request = SubscriptionRequest::read(&mut stream).unwrap();
        // TODO: Proper error handling.
        event_sender
            .send(Event::SubscriptionRequest(id, subscription_request))
            .unwrap();

        // Send incoming messages to the subscriber.
        // TODO: Proper error handling.
        loop {
            let message = message_receiver.recv().unwrap();

            if let Err(e) = message.write(&mut stream) {
                log::error!("Error writing message: [{}]", e);
                break;
            }
        }
    }
}

impl Drop for SubscriberHandler {
    fn drop(&mut self) {
        if let Some(thread) = self.subscriber_thread.take() {
            thread.join().unwrap();
        }
    }
}
