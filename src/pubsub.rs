use std::io;
use std::net::TcpStream;

use crossbeam::channel::{self, Receiver, Sender};

use crate::background_tcp_listener::BackgroundTcpListener;
use crate::connection_kind::ConnectionKind;
use crate::error;
use crate::event::Event;
use crate::message::Message;
use crate::publisher_handler::PublisherHandler;
use crate::subscriber_handler::SubscriberHandler;

pub struct PubSub {
    subscriber_handlers: Vec<SubscriberHandler>,
    publisher_handlers: Vec<PublisherHandler>,
    _publisher_listener: BackgroundTcpListener,
    _subscriber_listener: BackgroundTcpListener,
    event_sender: Sender<Event>,
    event_receiver: Receiver<Event>,
}

impl PubSub {
    pub fn new(publisher_port: u16, subscriber_port: u16) -> Self {
        log::info!(
            "PubSub: publisher_port=({}), subscriber_port=({})",
            publisher_port,
            subscriber_port
        );

        // Create a channel that will be used for communication between threads.
        log::info!("Creating the communication channel");
        let (event_sender, event_receiver): (Sender<Event>, Receiver<Event>) = channel::unbounded();

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
        Self {
            subscriber_handlers: Vec::new(),
            publisher_handlers: Vec::new(),
            _publisher_listener: publisher_listener,
            _subscriber_listener: subscriber_listener,
            event_sender,
            event_receiver,
        }
    }

    pub fn process_events(&mut self) -> error::Result<()> {
        log::info!("Starting to process incoming events");

        loop {
            // Receive an event from the channel.
            let event = self.event_receiver.recv()?;
            log::info!("Received event: [{}]", event);

            // Handle the event.
            if let Err(e) = self.handle_event(event) {
                log::error!("Error while handling event: [{}]", e);
            }
        }
    }

    fn start_background_tcp_listener(
        port: u16,
        connection_kind: ConnectionKind,
        event_sender: Sender<Event>,
    ) -> BackgroundTcpListener {
        let address = format!("0.0.0.0:{}", port);
        BackgroundTcpListener::new(address, connection_kind, event_sender)
    }

    fn handle_event(&mut self, event: Event) -> error::Result<()> {
        match event {
            Event::Connection(kind, stream) => self.handle_connection(kind, stream)?,
            Event::Publish(message) => self.handle_publish(message),
        }

        Ok(())
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
    }

    fn handle_publisher_connection(&mut self, stream: io::Result<TcpStream>) -> error::Result<()> {
        log::debug!("Publisher connection: {stream:?}");

        let publisher_handler = PublisherHandler::new(stream?, self.event_sender.clone());
        self.publisher_handlers.push(publisher_handler);

        Ok(())
    }

    fn handle_subscriber_connection(&mut self, stream: io::Result<TcpStream>) -> error::Result<()> {
        log::debug!("Subscriber connection: {stream:?}");

        let subscriber_handler = SubscriberHandler::new(stream?, self.event_sender.clone());
        self.subscriber_handlers.push(subscriber_handler);

        Ok(())
    }
}
