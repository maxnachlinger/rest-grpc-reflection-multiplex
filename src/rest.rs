use axum::routing::get;
use axum::Router;
use tower_http::trace::TraceLayer;

async fn web_root() -> &'static str {
    "Echo, World!"
}

pub fn setup_rest() -> Router {
    Router::new()
        .route("/", get(web_root))
        .layer(TraceLayer::new_for_http())
}
