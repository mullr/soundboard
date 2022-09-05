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

    for d in args.drops.into_iter() {
        library.add_collection(Collection::from_dir(d, model::CollectionKind::Drops)?);
    }

    for d in args.music.into_iter() {
        library.add_collection(Collection::from_dir(d, model::CollectionKind::Music)?);
    }

    for d in args.fx.into_iter() {
        library.add_collection(Collection::from_dir(d, model::CollectionKind::Fx)?);
    }

    for d in args.ambience.into_iter() {
        library.add_collection(Collection::from_dir(d, model::CollectionKind::Ambience)?);
    }

    if library.collections.is_empty() {
        println!("Error: At least one kind of library directory must be provided.");
        return Ok(());
    }

    let library = Arc::new(library);

    let player = Player::new()?;
    let player = Arc::new(Mutex::new(player));

    server::run_server(args.address, library, player).await
}

#[derive(clap::Parser)]
struct Args {
    /// A directory with sound drops; relatively short pieces of music.
    #[clap(short, long)]
    drops: Vec<PathBuf>,

    /// A directory with background music
    #[clap(short, long)]
    music: Vec<PathBuf>,

    /// A directory with sound effects
    #[clap(short, long)]
    fx: Vec<PathBuf>,

    /// A directory with ambience recordings
    #[clap(short, long)]
    ambience: Vec<PathBuf>,

    /// What address to listen on.
    #[clap(long, value_parser, default_value = "127.0.0.1:14181")]
    address: SocketAddr,
}
