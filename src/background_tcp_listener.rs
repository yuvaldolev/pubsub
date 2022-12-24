use std::io;
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

use crossbeam::channel::Sender;

use crate::connection_kind::ConnectionKind;
use crate::error;
use crate::event::Event;

pub struct BackgroundTcpListener {
    listener_thread: Option<JoinHandle<()>>,
    address: String,
    terminate: Arc<Mutex<bool>>,
}

impl BackgroundTcpListener {
    pub fn new(
        address: String,
        connection_kind: ConnectionKind,
        event_sender: Sender<Event>,
    ) -> Self {
        let terminate = Arc::new(Mutex::new(false));

        Self {
            listener_thread: Some(Self::start_listener_thread(
                address.clone(),
                connection_kind,
                event_sender,
                terminate.clone(),
            )),
            address,
            terminate,
        }
    }

    fn start_listener_thread(
        address: String,
        connection_kind: ConnectionKind,
        event_sender: Sender<Event>,
        terminate: Arc<Mutex<bool>>,
    ) -> JoinHandle<()> {
        thread::spawn(move || Self::listen(address, connection_kind, event_sender, terminate))
    }

    fn listen(
        address: String,
        connection_kind: ConnectionKind,
        event_sender: Sender<Event>,
        terminate: Arc<Mutex<bool>>,
    ) {
        log::info!(
            "Listening for [{}] connections to: [{}]",
            connection_kind,
            address
        );

        // Create the TCP listener.
        let listener = match TcpListener::bind(address.clone()) {
            Ok(listener) => listener,
            Err(e) => {
                log::error!(
                    "Failed to bind to address [{}] for [{}] connections: [{}]",
                    address,
                    connection_kind,
                    e,
                );
                return;
            }
        };

        // Listen for connections.
        for stream in listener.incoming() {
            // Check if listening should terminate.
            if *terminate.lock().unwrap() {
                break;
            }

            // Send a Connection event.
            Self::send_connection_event(&event_sender, connection_kind.clone(), stream);
        }
    }

    fn send_connection_event(
        event_sender: &Sender<Event>,
        connection_kind: ConnectionKind,
        stream: io::Result<TcpStream>,
    ) {
        // Ensure that the stream is valid before sending the Connection event.
        match stream {
            Ok(stream) => {
                if let Err(e) =
                    event_sender.send(Event::Connection(connection_kind.clone(), stream))
                {
                    log::error!(
                        "Failed sending [{}] Connection event: [{}]",
                        connection_kind,
                        e,
                    );
                }
            }
            Err(e) => log::error!("Failed receiving connection: [{}]", e),
        }
    }

    fn unblock_listener_thread(&self) -> error::Result<()> {
        TcpStream::connect(&self.address)?;
        Ok(())
    }
}

impl Drop for BackgroundTcpListener {
    fn drop(&mut self) {
        if let Some(thread) = self.listener_thread.take() {
            // Indicate the listener thread that it should terminate.
            *self.terminate.lock().unwrap() = true;

            // Unblock the listener thread.
            self.unblock_listener_thread().unwrap();

            // Join the listener thread.
            thread.join().unwrap();
        }
    }
}
