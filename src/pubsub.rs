pub struct PubSub {
    pub_port: u16,
    sub_port: u16,
}

impl PubSub {
    pub fn new(pub_port: u16, sub_port: u16) -> Self {
        Self { pub_port, sub_port }
    }

    pub fn run(&self) {
        log::info!(
            "Running: pub_port={}, sub_port={}",
            self.pub_port,
            self.sub_port
        );
        // let publisher_listener = BackgroundTcpListener::new(format!("0.0.0.0:{}", self.pub_port));
        // publisher_listener.listen(|| self.handle_publisher_connected());

        // Spawn a thread for listening to incoming publisher connections.
        // let subscriber
        // let publisher_thread = thread::spawn(move || self.listen_for_publishers());
        // publisher_thread.join();

        // Spawn a thread for listening to incoming subscriber connections.
    }
}
