use anyhow::Result;
use clap::Parser;
use tokio::sync::Mutex;

mod fswatch;
mod socks;
mod web;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long, default_value = "0.0.0.0")]
    socks_host: String,
    #[arg(short, long, default_value_t = 1080)]
    socks_port: u16,
    #[arg(long, default_value_t = false)]
    web: bool,
    #[arg(long, default_value = "0.0.0.0")]
    web_host: String,
    #[arg(long, default_value_t = 8080)]
    web_port: u16,
    #[arg(long)]
    watch: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let args = Args::parse();

    let mut proxies = Mutex::new(Vec::<String>::new());

    socks::spawn_socks_service(format!("{}:{}", args.socks_host, args.socks_port)).await?;

    Ok(())
}
