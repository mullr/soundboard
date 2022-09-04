use std::sync::Arc;

use askama::Template;
use axum::{
    extract::Path,
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{get, post},
    Extension, Router,
};
use tokio::sync::Mutex;
use tracing::{error, info, warn};

use crate::{model::Library, player::Player};

pub async fn run_server(
    port: u16,
    library: Arc<Library>,
    player: Arc<Mutex<Player>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // build our application with a single route
    let app = Router::new()
        .route("/", get(root))
        .route("/collection/:coll_id/clip/:clip_id/play", post(play_clip))
        .route("/collection/:coll_id/clip/:clip_id/stop", post(stop_clip))
        .route("/stop_all", post(stop_all))
        .layer(Extension(library))
        .layer(Extension(player));

    info!(%port, "Running http server");

    // run it with hyper on localhost:3000
    Ok(axum::Server::bind(&format!("0.0.0.0:{port}").parse()?)
        .serve(app.into_make_service())
        .await?)
}

struct HtmlTemplate<T>(T);
impl<T> IntoResponse for HtmlTemplate<T>
where
    T: Template,
{
    fn into_response(self) -> axum::response::Response {
        match self.0.render() {
            Ok(html) => Html(html).into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to render template. {}", err),
            )
                .into_response(),
        }
    }
}

#[derive(Template)]
#[template(path = "index.html")]
struct RootTemplate {
    library: Arc<Library>,
}

async fn root(Extension(library): Extension<Arc<Library>>) -> impl IntoResponse {
    let template = RootTemplate { library };
    HtmlTemplate(template)
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

async fn stop_all(Extension(player_mutex): Extension<Arc<Mutex<Player>>>) {
    info!("Stop all");
    let mut player = player_mutex.lock().await;
    player.stop_all();
}
