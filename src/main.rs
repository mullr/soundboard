mod model;
mod player;
mod server;

use clap::Parser;
use model::{Collection, Library};
use player::Player;
use std::{path::PathBuf, sync::Arc};
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

    let (player, _stream) = Player::new()?;
    let player = Arc::new(Mutex::new(player));

    server::run_server(args.port, library, player).await
}

#[derive(clap::Parser)]
struct Args {
    #[clap(short, long)]
    directory: Vec<PathBuf>,

    #[clap(short, long, value_parser, default_value_t = 14181)]
    port: u16,
}
