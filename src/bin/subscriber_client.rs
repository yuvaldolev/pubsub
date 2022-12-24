use std::net::TcpStream;

use clap::Parser;

use pubsub::Message;
use pubsub::SubscriptionRequest;

#[derive(Parser)]
#[command(author = "ydolev", version = "1.0.0", about = "A pubsub subscriber client written in Rust", long_about = None)]
struct Cli {
    port: u16,
    topics: Vec<String>,
}

fn main() -> anyhow::Result<()> {
    // Initialize the logger according to the environment.
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("debug"));

    // Parse the command line arguments.
    let cli = Cli::parse();

    // Connect to the pubsub server.
    log::info!("Connecting to the pubsub server on port: ({})", cli.port);
    let mut stream = TcpStream::connect(format!("localhost:{}", cli.port))?;

    // Send the subscription request.
    let subscription_request = SubscriptionRequest::new(cli.topics);
    subscription_request.write(&mut stream)?;

    // Receive message from publishers.
    loop {
        let message = Message::read(&mut stream)?;
        let data = String::from_utf8(message.data)?;

        log::info!(
            "Received message from topic [{}]: [{}]",
            message.topic,
            data
        );
    }
}
