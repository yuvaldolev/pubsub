use std::net::TcpListener;
use std::thread::{self, JoinHandle};

use crossbeam::channel::Sender;

use crate::connection_kind::ConnectionKind;
use crate::event::Event;

pub struct BackgroundTcpListener {
    listener_thread: Option<JoinHandle<()>>,
}

impl BackgroundTcpListener {
    pub fn new(
        address: String,
        connection_kind: ConnectionKind,
        event_sender: Sender<Event>,
    ) -> Self {
        let listener_thread =
            thread::spawn(move || Self::listen(address, connection_kind, event_sender));
        Self {
            listener_thread: Some(listener_thread),
        }
    }

    fn listen(address: String, connection_kind: ConnectionKind, event_sender: Sender<Event>) {
        log::info!("Listening for connections to: [{}]", address);

        // Create the TCP listener.
        // TODO: Proper error handling.
        let listener = TcpListener::bind(address).unwrap();

        // Listen for connections.
        // TODO: Graceful shutdown.
        // TODO: Proper error handling.
        for stream in listener.incoming() {
            event_sender
                .send(Event::Connection(connection_kind.clone(), stream))
                .unwrap();
        }
    }
}

impl Drop for BackgroundTcpListener {
    fn drop(&mut self) {
        if let Some(thread) = self.listener_thread.take() {
            thread.join().unwrap();
        }
    }
}
