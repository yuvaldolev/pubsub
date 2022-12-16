use clap::Parser;

#[derive(Parser)]
#[command(author = "ydolev", version = "1.0.0", about = "A pubsub system written in Rust", long_about = None)]
struct Cli {
    #[arg(long)]
    pub_port: u16,

    #[arg(long)]
    sub_port: u16,
}

fn main() {
    let _cli = Cli::parse();
}
