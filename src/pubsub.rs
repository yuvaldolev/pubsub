use crossbeam::channel::{self, Receiver, Sender};

use crate::background_tcp_listener::BackgroundTcpListener;
use crate::connection_kind::ConnectionKind;
use crate::event::Event;

pub struct PubSub {
    publisher_port: u16,
    subscriber_port: u16,
    event_sender: Sender<Event>,
    event_receiver: Receiver<Event>,
}

impl PubSub {
    pub fn new(publisher_port: u16, subscriber_port: u16) -> Self {
        let (event_sender, event_receiver): (Sender<Event>, Receiver<Event>) = channel::unbounded();

        Self {
            publisher_port,
            subscriber_port,
            event_sender,
            event_receiver,
        }
    }

    pub fn run(&self) {
        log::info!(
            "Running: publisher_port=({}), subscriber_port=({})",
            self.publisher_port,
            self.subscriber_port
        );

        // Create a channel for sending and receiving events.
        // TODO: Should this be moved to a member???
        log::debug!("Creating events channel");

        // Start the publisher TCP listener.
        log::info!(
            "Starting the publisher TCP listener on port: ({})",
            self.publisher_port
        );
        let _publisher_listener = Self::start_background_tcp_listener(
            self.publisher_port,
            ConnectionKind::Publisher,
            self.event_sender.clone(),
        );

        // Start the subscriber TCP listener.
        log::info!(
            "Starting the subscriber TCP listener on port: ({})",
            self.subscriber_port
        );
        let _subscriber_listener = Self::start_background_tcp_listener(
            self.subscriber_port,
            ConnectionKind::Subscriber,
            self.event_sender.clone(),
        );

        // Process the incoming events.
        log::info!("Starting to process incoming events");
        self.process_events();
    }

    fn start_background_tcp_listener(
        port: u16,
        connection_kind: ConnectionKind,
        event_sender: Sender<Event>,
    ) -> BackgroundTcpListener {
        let address = format!("0.0.0.0:{}", port);
        BackgroundTcpListener::new(address, connection_kind, event_sender)
    }

    fn process_events(&self) {
        loop {
            let event = self.event_receiver.recv();
            log::debug!("Received event: {:?}", event.unwrap());
        }
    }
}
