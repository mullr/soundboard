use std::sync::Arc;

use axum::{
    extract::Path,
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{get, post},
    Extension, Router,
};
use askama::Template;
use tokio::sync::Mutex;

use crate::{model::Library, player::Player};

pub async fn run_server(library: Arc<Library>, player: Arc<Mutex<Player>>) {
    // build our application with a single route
    let app = Router::new()
        .route("/", get(root))
        .route("/collection/:coll_id/clip/:clip_id/play", post(play_clip))
        .route("/collection/:coll_id/clip/:clip_id/stop", post(stop_clip))
        .route("/stop_all", post(stop_all))
        .layer(Extension(library))
        .layer(Extension(player));

    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
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
) -> impl IntoResponse {
    let mut player = player_mutex.lock().await;
    let path = match library.clip_path(coll_id, clip_id) {
        Some(p) => p,
        None => return (StatusCode::NOT_FOUND, "Uknown clip id"),
    };
    player.play_clip(coll_id, clip_id , path);
    println!("Play clip {coll_id}/{clip_id}");

    (StatusCode::ACCEPTED, "Playing")
}

async fn stop_clip(Path((coll_id, clip_id)): Path<(u64, u64)>) {
    println!("Play clip {coll_id}/{clip_id}")
}

async fn stop_all(Extension(player_mutex): Extension<Arc<Mutex<Player>>>) {
    let mut player = player_mutex.lock().await;
    player.stop_all();
    println!("Stop all")
}
