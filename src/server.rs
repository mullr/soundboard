use std::{net::SocketAddr, sync::Arc};

use axum::{
    extract::Path,
    http::StatusCode,
    response::sse::{Event, KeepAlive, Sse},
    routing::{get, post},
    Extension, Json, Router,
};
use axum_static_macro::static_file;
use futures::{stream::Stream, StreamExt};
use tokio::sync::{broadcast::Sender, Mutex};
use tracing::{error, info};

use crate::{
    api,
    model::Library,
    player::{Player, PlayerEvent},
};

pub async fn run_server(
    address: SocketAddr,
    library: Arc<Library>,
    player: Arc<Mutex<Player>>,
    player_event_broadcast: Sender<PlayerEvent>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    static_file!(index_html, "public/index.html", "text/html");
    static_file!(index_js, "public/index.js", "application/javascript");
    static_file!(
        preact_preact_mjs,
        "public/preact/dist/preact.mjs",
        "application/javascript"
    );
    static_file!(
        preact_hooks_mjs,
        "public/preact/hooks/dist/hooks.mjs",
        "application/javascript"
    );
    static_file!(
        preact_debug_mjs,
        "public/preact/debug/dist/debug.mjs",
        "application/javascript"
    );
    static_file!(
        preact_devtools_mjs,
        "public/preact/devtools/dist/devtools.mjs",
        "application/javascript"
    );

    // build our application with a single route
    let app = Router::new()
        // .route("/", get(root))
        .route("/", get(index_html))
        .route("/index.js", get(index_js))
        .route("/collection", get(collections))
        .route("/playing", get(playing))
        .route("/collection/:coll_id/stop", post(stop_coll))
        .route("/collection/:coll_id/clip/:clip_id/play", post(play_clip))
        .route("/collection/:coll_id/clip/:clip_id/stop", post(stop_clip))
        .route("/stop_all", post(stop_all))
        .route("/events", get(events))
        .route("/preact/preact.mjs", get(preact_preact_mjs))
        .route("/preact/hooks.mjs", get(preact_hooks_mjs))
        .route("/preact/debug.mjs", get(preact_debug_mjs))
        .route("/preact/devtools.mjs", get(preact_devtools_mjs))
        .layer(Extension(library))
        .layer(Extension(player))
        .layer(Extension(player_event_broadcast));

    info!("Running http server on http://{address}");

    Ok(axum::Server::bind(&address)
        .serve(app.into_make_service())
        .await?)
}

async fn collections(Extension(library): Extension<Arc<Library>>) -> Json<Vec<api::Collection>> {
    let api_lib: api::Library = (*library).clone().into();

    Json(api_lib.collections)
}

async fn playing(
    Extension(player_mutex): Extension<Arc<Mutex<Player>>>,
) -> Json<Vec<(String, String)>> {
    let playing = {
        let player = player_mutex.lock().await;
        player
            .playing_clips()
            .into_iter()
            .map(|(coll_id, clip_id)| (coll_id.to_string(), clip_id.to_string()))
            .collect::<Vec<_>>()
    };

    Json(playing)
}

async fn play_clip(
    Path((coll_id, clip_id)): Path<(u64, u64)>,
    Extension(library): Extension<Arc<Library>>,
    Extension(player_mutex): Extension<Arc<Mutex<Player>>>,
) -> Result<String, StatusCode> {
    info!("Play clip {coll_id}/{clip_id}");

    let mut player = player_mutex.lock().await;
    let coll = library.collection(coll_id).ok_or(StatusCode::NOT_FOUND)?;
    let clip = coll.clip(clip_id).ok_or(StatusCode::NOT_FOUND)?;

    player
        .play_clip(coll_id, clip_id, clip.path.to_owned(), coll.kind)
        .map_err(|e| {
            error!(err = %&e as &dyn std::error::Error, "Error playing clip");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok("Playing".to_string())
}

async fn stop_clip(
    Path((coll_id, clip_id)): Path<(u64, u64)>,
    Extension(player_mutex): Extension<Arc<Mutex<Player>>>,
) -> Result<String, StatusCode> {
    info!("Stop clip {coll_id}/{clip_id}");
    let mut player = player_mutex.lock().await;

    player.stop_clip(coll_id, clip_id).map_err(|e| {
        error!(err = %&e as &dyn std::error::Error, "Error stopping clip");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok("Stopped".to_string())
}

async fn stop_coll(
    Path(coll_id): Path<u64>,
    Extension(player_mutex): Extension<Arc<Mutex<Player>>>,
) -> Result<String, StatusCode> {
    info!("Stop collection {coll_id}");
    let mut player = player_mutex.lock().await;

    player.stop_coll(coll_id).map_err(|e| {
        error!(err = %&e as &dyn std::error::Error, "Error stopping collection");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok("Stopped".to_string())
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

async fn events(
    Extension(player_event_broadcast): Extension<Sender<PlayerEvent>>,
) -> Sse<impl Stream<Item = Result<Event, Box<dyn std::error::Error + Send + Sync>>>> {
    // use async_stream::stream;

    let rx = player_event_broadcast.subscribe();
    let s = tokio_stream::wrappers::BroadcastStream::new(rx).map(|ev_res| match ev_res {
        Ok(ev) => match Event::default().json_data(api::PlayerEvent::from(ev)) {
            Ok(e) => Ok(e),
            Err(e) => {
                let e: Box<dyn std::error::Error + Send + Sync> = Box::new(e);
                Err(e)
            }
        },
        Err(e) => {
            let e: Box<dyn std::error::Error + Send + Sync> = Box::new(e);
            Err(e)
        }
    });

    Sse::new(s).keep_alive(KeepAlive::default())
}
