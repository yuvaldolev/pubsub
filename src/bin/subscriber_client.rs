use std::net::TcpStream;

use clap::Parser;

#[derive(Parser)]
#[command(author = "ydolev", version = "1.0.0", about = "A pubsub subscriber client written in Rust", long_about = None)]
struct Cli {
    port: u16,
}

fn main() -> anyhow::Result<()> {
    // Initialize the logger according to the environment.
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("debug"));

    // Parse the command line arguments.
    let cli = Cli::parse();

    // Connect to the pubsub server.
    log::info!("Connecting to the pubserver server on port: ({})", cli.port);
    let mut stream = TcpStream::connect(format!("localhost:{}", cli.port))?;

    Ok(())
}
