use axum::{response::IntoResponse, routing::get, Router};
use tower::limit::ConcurrencyLimitLayer;
use tower_http::trace::TraceLayer;
use tracing::info;

async fn handle() -> impl IntoResponse {
    "OK"
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let middleware = tower::ServiceBuilder::new()
        .layer(TraceLayer::new_for_http())
        // FIXME: Warp isn't currently designed to handle multiple requests concurrently.
        .layer(ConcurrencyLimitLayer::new(1));

    let app = Router::new()
        .route("/api/v1", get(handle))
        .layer(middleware);

    let listener = tokio::net::TcpListener::bind(ADDRESS).await.unwrap();
    info!("listening on {ADDRESS}");
    axum::serve(listener, app).await.unwrap();
}

const ADDRESS: &str = "localhost:23818";
