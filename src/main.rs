mod args;
mod conventional;
mod gitee;

use args::Args;
use clap::Parser;
use gitee::create_release;
use std::error::Error;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Init tracing subscriber
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO) // Set the log level to INFO
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Setting default subscriber failed");

    let args = Args::parse();

    create_release(args, None).await?;

    Ok(())
}
