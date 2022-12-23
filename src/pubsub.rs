use std::collections::HashMap;
use std::io;
use std::net::TcpStream;

use crossbeam::channel::{self, Receiver, Sender};
use uuid::Uuid;

use crate::background_tcp_listener::BackgroundTcpListener;
use crate::connection_kind::ConnectionKind;
use crate::error;
use crate::event::Event;
use crate::message::Message;
use crate::publisher_handler::PublisherHandler;
use crate::subscriber_handler::SubscriberHandler;
use crate::subscription_request::SubscriptionRequest;

pub struct PubSub {
    subscriber_to_handler: HashMap<Uuid, SubscriberHandler>,
    topic_to_subscribers: HashMap<String, Vec<Uuid>>,
    publisher_handlers: Vec<PublisherHandler>,
    _publisher_listener: BackgroundTcpListener,
    _subscriber_listener: BackgroundTcpListener,
    event_sender: Sender<Event>,
    event_receiver: Receiver<Event>,
}

impl PubSub {
    pub fn new(publisher_port: u16, subscriber_port: u16) -> error::Result<Self> {
        log::info!(
            "PubSub: publisher_port=({}), subscriber_port=({})",
            publisher_port,
            subscriber_port
        );

        // Create a channel that will be used for communication between threads.
        log::info!("Creating the communication channel");
        let (event_sender, event_receiver): (Sender<Event>, Receiver<Event>) = channel::unbounded();

        // Register a handler for ctrl-c.
        Self::register_ctrlc_handler(event_sender.clone())?;

        // Start the publisher TCP listener.
        log::info!(
            "Starting the publisher TCP listener on port: ({})",
            publisher_port
        );
        let publisher_listener = Self::start_background_tcp_listener(
            publisher_port,
            ConnectionKind::Publisher,
            event_sender.clone(),
        );

        // Start the subscriber TCP listener.
        log::info!(
            "Starting the subscriber TCP listener on port: ({})",
            subscriber_port
        );
        let subscriber_listener = Self::start_background_tcp_listener(
            subscriber_port,
            ConnectionKind::Subscriber,
            event_sender.clone(),
        );

        // Create the PubSub instance.
        Ok(Self {
            subscriber_to_handler: HashMap::new(),
            topic_to_subscribers: HashMap::new(),
            publisher_handlers: Vec::new(),
            _publisher_listener: publisher_listener,
            _subscriber_listener: subscriber_listener,
            event_sender,
            event_receiver,
        })
    }

    pub fn process_events(&mut self) -> error::Result<()> {
        log::info!("Starting to process incoming events");

        let mut running = true;
        while running {
            // Receive an event from the channel.
            let event = self.event_receiver.recv()?;
            log::info!("Received event: [{}]", event);

            // Handle the event.
            running = match self.handle_event(event) {
                Ok(keep_running) => keep_running,
                Err(e) => {
                    log::error!("Error handling event: [{}]", e);
                    false
                }
            }
        }

        Ok(())
    }

    fn register_ctrlc_handler(event_sender: Sender<Event>) -> error::Result<()> {
        ctrlc::set_handler(move || event_sender.send(Event::Termination).unwrap())?;
        Ok(())
    }

    fn start_background_tcp_listener(
        port: u16,
        connection_kind: ConnectionKind,
        event_sender: Sender<Event>,
    ) -> BackgroundTcpListener {
        let address = format!("0.0.0.0:{}", port);
        BackgroundTcpListener::new(address, connection_kind, event_sender)
    }

    fn handle_event(&mut self, event: Event) -> error::Result<bool> {
        match event {
            Event::Connection(kind, stream) => self.handle_connection(kind, stream)?,
            Event::Publish(message) => self.handle_publish(message),
            Event::SubscriptionRequest(id, request) => {
                self.handle_subscription_request(id, request)
            }
            Event::Termination => return Ok(false),
        }

        Ok(true)
    }

    fn handle_connection(
        &mut self,
        kind: ConnectionKind,
        stream: io::Result<TcpStream>,
    ) -> error::Result<()> {
        match kind {
            ConnectionKind::Publisher => self.handle_publisher_connection(stream)?,
            ConnectionKind::Subscriber => self.handle_subscriber_connection(stream)?,
        }

        Ok(())
    }

    fn handle_publish(&self, message: Message) {
        log::debug!("Publishing message: [{:?}]", message);

        match self.topic_to_subscribers.get(&message.topic) {
            Some(subscribers) => {
                self.publish_message_to_subscribers(message, subscribers);
            }
            None => log::warn!("No subscribers registered to topic: [{}]", message.topic),
        }
    }

    fn handle_subscription_request(&mut self, id: Uuid, request: SubscriptionRequest) {
        log::info!("Subscription request from: [{}]", id);

        // Register the subscriber to all requested topics.
        for topic in request.topics.into_iter() {
            self.topic_to_subscribers
                .entry(topic)
                .or_insert(Vec::new())
                .push(id);
        }
    }

    fn handle_publisher_connection(&mut self, stream: io::Result<TcpStream>) -> error::Result<()> {
        log::debug!("Publisher connection: {stream:?}");

        let publisher_handler = PublisherHandler::new(stream?, self.event_sender.clone());
        self.publisher_handlers.push(publisher_handler);

        Ok(())
    }

    fn handle_subscriber_connection(&mut self, stream: io::Result<TcpStream>) -> error::Result<()> {
        log::debug!("Subscriber connection: {stream:?}");

        // Generate a unique ID for the subscriber.
        let subscriber_id = Uuid::new_v4();

        // Create a new handler for the subscriber.
        let subscriber_handler =
            SubscriberHandler::new(subscriber_id, stream?, self.event_sender.clone());

        // Add the subscriber to the handlers map.
        self.subscriber_to_handler
            .insert(subscriber_id, subscriber_handler);

        Ok(())
    }

    fn publish_message_to_subscribers(&self, message: Message, subscribers: &Vec<Uuid>) {
        for subscriber in subscribers {
            match self.subscriber_to_handler.get(subscriber) {
                Some(handler) => {
                    self.publish_message_to_subscriber(message.clone(), subscriber, handler)
                }
                None => log::error!("No handler for subscriber: [{}]", subscriber),
            }
        }
    }

    fn publish_message_to_subscriber(
        &self,
        message: Message,
        id: &Uuid,
        handler: &SubscriberHandler,
    ) {
        if let Err(e) = handler.publish(message.clone()) {
            log::error!("Error publishing message to subscriber [{}]: [{}]", id, e);
        }
    }
}
