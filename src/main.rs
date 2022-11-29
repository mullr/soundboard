#![allow(unused)]

mod api;
mod discord;
mod model;
mod player;
mod server;

use clap::Parser;
use hyper::body::Buf;
use model::{Collection, Library};
use player::{Player, PlayerEvent};
use ringbuf::HeapRb;
use std::{
    collections::HashMap,
    io::{Read, Seek},
    net::SocketAddr,
    path::PathBuf,
    sync::Arc,
    time::Duration,
};
use tokio::sync::{Mutex, RwLock};
use tokio_stream::StreamExt;
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

    for d in args.fx.into_iter() {
        library.add_collection(Collection::from_dir(d, model::CollectionKind::Fx)?);
    }

    for d in args.drops.into_iter() {
        library.add_collection(Collection::from_dir(d, model::CollectionKind::Drops)?);
    }

    for d in args.battle_music.into_iter() {
        library.add_collection(Collection::from_dir(d, model::CollectionKind::BattleMusic)?);
    }

    for d in args.ambience.into_iter() {
        library.add_collection(Collection::from_dir(d, model::CollectionKind::Ambience)?);
    }

    for d in args.bgm.into_iter() {
        library.add_collection(Collection::from_dir(
            d,
            model::CollectionKind::BackgroundMusic,
        )?);
    }

    if library.collections.is_empty() {
        println!("Error: At least one kind of library directory must be provided.");
        return Ok(());
    }

    let library = Arc::new(library);

    let (player, maybe_discord_conn) = match args.discord_token {
        Some(token) => {
            let buffer_size = 1920 * 8;
            let buffer = ringbuf::HeapRb::new(buffer_size);
            let (audio_producer, audio_consumer) = buffer.split();
            let player = Player::new(Some(audio_producer))?;
            let conn = discord::DiscordConnection::connect(audio_consumer, token).await?;
            (player, Some(conn))
        }
        None => (Player::new(None)?, None),
    };

    let player = Arc::new(Mutex::new(player));
    let (player_event_tx, _) = tokio::sync::broadcast::channel::<PlayerEvent>(16);
    let player_for_poller = player.clone();
    let tx_for_poller = player_event_tx.clone();
    tokio::spawn(async move {
        loop {
            if let Err(e) =
                player::poll_events(player_for_poller.clone(), tx_for_poller.clone()).await
            {
                error!(err = &e as &dyn std::error::Error);
            }

            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    });

    server::run_server(args.address, library, player, player_event_tx).await
}

#[derive(clap::Parser)]
struct Args {
    /// A directory with sound drops; relatively short pieces of music.
    #[clap(long)]
    drops: Vec<PathBuf>,

    /// A directory with background music
    #[clap(long)]
    bgm: Vec<PathBuf>,

    /// A directory with battle music
    #[clap(long)]
    battle_music: Vec<PathBuf>,

    /// A directory with sound effects
    #[clap(long)]
    fx: Vec<PathBuf>,

    /// A directory with ambience recordings
    #[clap(long)]
    ambience: Vec<PathBuf>,

    /// What address to listen on.
    #[clap(long, value_parser, default_value = "127.0.0.1:14181")]
    address: SocketAddr,

    /// The bot's api token, for discord. If not given, plays back the
    /// sounds locally.
    #[clap(env)]
    discord_token: Option<String>,
}
