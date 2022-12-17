use crate::background_tcp_listener::BackgroundTcpListener;

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

        // Start the publisher TCP listener.
        let _publisher_listener = Self::start_background_tcp_listener(self.publisher_port);

        // Start the subscriber TCP listener.
        let _subscriber_listener = Self::start_background_tcp_listener(self.subscriber_port);

        loop {
            log::debug!("In pubsub thread");
            std::thread::sleep(std::time::Duration::from_secs(2));
        }

        // Spawn a thread for listening to incoming publisher connections.
        // let subscriber
        // let publisher_thread = thread::spawn(move || self.listen_for_publishers());
        // publisher_thread.join();

        // Spawn a thread for listening to incoming subscriber connections.
    }

    fn start_background_tcp_listener(port: u16) -> BackgroundTcpListener {
        let address = format!("0.0.0.0:{}", port);
        BackgroundTcpListener::new(address)
    }
}
