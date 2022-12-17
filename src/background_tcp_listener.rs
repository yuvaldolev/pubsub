use std::net::TcpListener;
use std::thread::{self, JoinHandle};

pub struct BackgroundTcpListener {
    listener_thread: Option<JoinHandle<()>>,
}

impl BackgroundTcpListener {
    pub fn new(address: String) -> Self {
        let listener_thread = thread::spawn(|| Self::listen(address));
        Self {
            listener_thread: Some(listener_thread),
        }
    }

    fn listen(address: String) {
        log::info!("Listening for connections to: [{}]", address);

        // Create the TCP listener.
        // TODO: Proper error handling.
        let listener = TcpListener::bind(address).unwrap();

        // Listen for connections.
        // TODO: Proper error handling.
        for stream in listener.incoming() {
            log::debug!(
                "Got connection from: {}",
                stream.unwrap().peer_addr().unwrap()
            );
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
