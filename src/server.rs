use std::{net::SocketAddr, sync::Arc};

use axum::{
    extract::Path,
    http::StatusCode,
    routing::{get, post},
    Extension, Router, Json,
};
use axum_static_macro::static_file;
use tokio::sync::Mutex;
use tracing::{error, info, warn};

use crate::{model::Library, player::Player, api};

pub async fn run_server(
    address: SocketAddr,
    library: Arc<Library>,
    player: Arc<Mutex<Player>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    static_file!(index_html, "public/index.html", "text/html");
    static_file!(index_js, "public/index.js", "application/javascript");

    // build our application with a single route
    let app = Router::new()
        // .route("/", get(root))
        .route("/", get(index_html))
        .route("/index.js", get(index_js))
        .route("/collection", get(collections))
        .route("/collection/:coll_id/clip/:clip_id/play", post(play_clip))
        .route("/collection/:coll_id/clip/:clip_id/stop", post(stop_clip))
        .route("/stop_all", post(stop_all))
        .layer(Extension(library))
        .layer(Extension(player));

    info!("Running http server on http://{address}");


    Ok(axum::Server::bind(&address)
        .serve(app.into_make_service())
        .await?)
}

async fn collections(Extension(library): Extension<Arc<Library>>) -> Json<Vec<api::Collection>>{
    let api_lib: api::Library = (*library).clone().into();
    Json(api_lib.collections)
}


async fn play_clip(
    Path((coll_id, clip_id)): Path<(u64, u64)>,
    Extension(library): Extension<Arc<Library>>,
    Extension(player_mutex): Extension<Arc<Mutex<Player>>>,
) -> Result<String, StatusCode> {
    info!("Play clip {coll_id}/{clip_id}");

    let mut player = player_mutex.lock().await;
    let path = library
        .clip_path(coll_id, clip_id)
        .ok_or(StatusCode::NOT_FOUND)?;

    player.play_clip(coll_id, clip_id, path).map_err(|e| {
        error!(err = %&e as &dyn std::error::Error, "Error playing clip");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok("Playing".to_string())
}

async fn stop_clip(Path((coll_id, clip_id)): Path<(u64, u64)>) {
    warn!("Stop clip {coll_id}/{clip_id} UNIMPLEMENTED")
}

async fn stop_all(
    Extension(player_mutex): Extension<Arc<Mutex<Player>>>,
) -> Result<String, StatusCode> {
    info!("Stop all");
    let mut player = player_mutex.lock().await;
    player.stop_all().map_err(|e| {
        error!(err = %&e as &dyn std::error::Error, "Error stopping clips");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok("Stopped".to_string())
}
