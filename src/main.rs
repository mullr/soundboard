mod api;
mod model;
mod player;
mod server;

use clap::Parser;
use model::{Collection, Library};
use player::Player;
use std::{net::SocketAddr, path::PathBuf, sync::Arc};
use tokio::sync::Mutex;
use tracing::error;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    if let Err(e) = run().await {
        error!(err = ?e.as_ref());
    }
}

async fn run() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let args = Args::parse();
    let mut library = Library::default();
    for d in args.directory.into_iter() {
        library.add_collection(Collection::from_dir(d)?);
    }

    let library = Arc::new(library);

    let player = Player::new()?;
    let player = Arc::new(Mutex::new(player));

    server::run_server(args.address, library, player).await
}

#[derive(clap::Parser)]
struct Args {
    /// A directory full of audio files, to use for the
    /// soundboard. May be provided more than once.
    #[clap(short, long, required(true))]
    directory: Vec<PathBuf>,

    /// What address to listen on.
    #[clap(short, long, value_parser, default_value = "127.0.0.1:14181")]
    address: SocketAddr,
}
