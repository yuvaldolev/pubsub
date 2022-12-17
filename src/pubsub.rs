pub struct PubSub {
    pub_port: u16,
    sub_port: u16,
}

impl PubSub {
    pub fn new(pub_port: u16, sub_port: u16) -> Self {
        Self { pub_port, sub_port }
    }

    pub fn run(&self) {
        println!(
            "Running: pub_port={}, sub_port={}",
            self.pub_port, self.sub_port
        );
    }
}
