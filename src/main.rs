mod model;
mod player;
mod server;

use model::{Collection, Library};
use player::Player;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() {
    let mut library = Library::default();
    library.add_collection(
        Collection::from_dir("/home/mullr/storage/TRPG/Star Trek Adventures/Audio/").unwrap(),
    );

    let library = Arc::new(library);

    let (player, _stream) = Player::new();
    let player = Arc::new(Mutex::new(player));

    server::run_server(library, player).await
}
