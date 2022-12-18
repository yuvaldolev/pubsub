use crossbeam::channel::{self, Receiver, Sender};

use crate::background_tcp_listener::BackgroundTcpListener;
use crate::connection_kind::ConnectionKind;
use crate::event::Event;

pub struct PubSub {
    publisher_port: u16,
    subscriber_port: u16,
}

impl PubSub {
    pub fn new(publisher_port: u16, subscriber_port: u16) -> Self {
        Self {
            publisher_port,
            subscriber_port,
        }
    }

    pub fn run(&self) {
        log::info!(
            "Running: publisher_port=({}), subscriber_port=({})",
            self.publisher_port,
            self.subscriber_port
        );

        // Create a channel for sending and receiving events.
        log::debug!("Creating events channel");
        let (event_sender, event_receiver): (Sender<Event>, Receiver<Event>) = channel::unbounded();

        // Start the publisher TCP listener.
        log::info!(
            "Starting the publisher TCP listener on port: ({})",
            self.publisher_port
        );
        let _publisher_listener = Self::start_background_tcp_listener(
            self.publisher_port,
            ConnectionKind::Publisher,
            event_sender.clone(),
        );

        // Start the subscriber TCP listener.
        log::info!(
            "Starting the subscriber TCP listener on port: ({})",
            self.subscriber_port
        );
        let _subscriber_listener = Self::start_background_tcp_listener(
            self.subscriber_port,
            ConnectionKind::Subscriber,
            event_sender.clone(),
        );

        // Handle the incoming events.
        log::info!("Starting to handle incoming events");
        Self::process_events(event_receiver);
    }

    fn start_background_tcp_listener(
        port: u16,
        connection_kind: ConnectionKind,
        event_sender: Sender<Event>,
    ) -> BackgroundTcpListener {
        let address = format!("0.0.0.0:{}", port);
        BackgroundTcpListener::new(address, connection_kind, event_sender)
    }

    fn process_events(receiver: Receiver<Event>) {
        loop {
            let event = receiver.recv();
            log::debug!("Received event: {:?}", event.unwrap());
        }
    }
}
