use clap::Parser;

use pubsub::PubSub;

#[derive(Parser)]
#[command(author = "ydolev", version = "1.0.0", about = "A pubsub system written in Rust", long_about = None)]
struct Cli {
    #[arg(long)]
    pub_port: u16,

    #[arg(long)]
    sub_port: u16,
}

fn main() -> anyhow::Result<()> {
    // Initialize the logger according to the environment.
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("debug"));

    // Parse the command line arguments.
    let cli = Cli::parse();

    // Initial and run the pubsub system.
    let pub_sub = PubSub::new(cli.pub_port, cli.sub_port);
    pub_sub.process_events()?;

    Ok(())
}
