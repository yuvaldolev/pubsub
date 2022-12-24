use std::net::TcpStream;
use std::thread;
use std::time::Duration;

use clap::Parser;
use rand::Rng;

use pubsub::Message;

#[derive(Parser)]
#[command(author = "ydolev", version = "1.0.0", about = "A pubsub publisher client written in Rust", long_about = None)]
struct Cli {
    port: u16,
}

fn main() -> anyhow::Result<()> {
    // Initialize the logger according to the environment.
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("debug"));

    // Parse the command line arguments.
    let cli = Cli::parse();

    // Connect to the pubsub server.
    log::info!("Connecting to the pubsub server on port: ({})", cli.port);
    let mut stream = TcpStream::connect(format!("localhost:{}", cli.port))?;

    // Create a random number generator.
    let mut rng = rand::thread_rng();

    // Send 10 messages to the different topics.
    let topics = vec!["hello", "bye", "test"];
    for message_number in 0..10 {
        // Send a message to a random topic.
        let topic_index: usize = rng.gen_range(0..topics.len());
        let message = Message::new(
            topics[topic_index].to_owned(),
            format!("message #{}", message_number).as_bytes().to_vec(),
        );
        log::info!(
            "Publishing message #{} to topic: [{}]",
            message_number,
            message.topic,
        );
        message.write(&mut stream)?;

        // Sleep for a random about of time between 0ms and 2000ms.
        let sleep_time_ms: u64 = rng.gen_range(0..=2000);
        thread::sleep(Duration::from_millis(sleep_time_ms));
    }

    Ok(())
}
