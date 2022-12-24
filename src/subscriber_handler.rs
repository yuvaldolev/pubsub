use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crossbeam::channel::{self, Receiver, Sender};
use uuid::Uuid;

use crate::error;
use crate::event::Event;
use crate::message::Message;
use crate::subscription_request::SubscriptionRequest;

const RECEIVE_MESSAGE_TIMEOUT_MS: u64 = 300;

pub struct SubscriberHandler {
    message_sender: Sender<Message>,
    handler_thread: Option<JoinHandle<()>>,
    terminate: Arc<Mutex<bool>>,
}

impl SubscriberHandler {
    pub fn new(id: Uuid, stream: TcpStream, event_sender: Sender<Event>) -> Self {
        let (message_sender, message_receiver): (Sender<Message>, Receiver<Message>) =
            channel::unbounded();

        let terminate = Arc::new(Mutex::new(false));

        Self {
            message_sender,
            handler_thread: Some(Self::start_handler_thread(
                id,
                stream,
                event_sender,
                message_receiver,
                terminate.clone(),
            )),
            terminate,
        }
    }

    pub fn publish(&self, message: Message) -> error::Result<()> {
        self.message_sender.send(message)?;
        Ok(())
    }

    fn start_handler_thread(
        id: Uuid,
        stream: TcpStream,
        event_sender: Sender<Event>,
        message_receiver: Receiver<Message>,
        terminate: Arc<Mutex<bool>>,
    ) -> JoinHandle<()> {
        thread::spawn(move || {
            Self::handle_subscriber(id, stream, event_sender, message_receiver, terminate)
        })
    }

    fn handle_subscriber(
        id: Uuid,
        mut stream: TcpStream,
        event_sender: Sender<Event>,
        message_receiver: Receiver<Message>,
        terminate: Arc<Mutex<bool>>,
    ) {
        log::info!(
            "Handling subscriber: id=[{}], address=[{}]",
            id,
            stream.peer_addr().unwrap()
        );

        // Receive the subscriber's subscription request.
        let subscription_request = match SubscriptionRequest::read(&mut stream) {
            Ok(request) => request,
            Err(e) => {
                log::error!(
                    "Failed receiving subscription request from [{}]: [{}]",
                    id,
                    e,
                );
                return;
            }
        };

        // Send a SubscriptionRequest event.
        if let Err(e) = event_sender.send(Event::SubscriptionRequest(id, subscription_request)) {
            log::error!(
                "Failed sending SubscriptionRequest event from [{}]: [{}]",
                id,
                e,
            );
            return;
        }

        // Send incoming messages to the subscriber.
        log::info!("Publishing incoming messages to: [{}]", id);
        while !(*terminate.lock().unwrap()) {
            // Receive a message from the messages channel.
            let message = match message_receiver
                .recv_timeout(Duration::from_millis(RECEIVE_MESSAGE_TIMEOUT_MS))
            {
                Ok(message) => message,
                Err(_) => {
                    continue;
                }
            };

            // Send the message to the subscriber.
            if let Err(e) = message.write(&mut stream) {
                log::error!("Error writing message to [{}]: [{}]", id, e);
                break;
            }
        }
    }
}

impl Drop for SubscriberHandler {
    fn drop(&mut self) {
        if let Some(thread) = self.handler_thread.take() {
            // Indicate the handler thread that it should terminate.
            *self.terminate.lock().unwrap() = true;

            // Join the handler thread.
            thread.join().unwrap();
        }
    }
}
