//! ISSO 51 REST API server.
//!
//! Axum-based API that wraps isso51-core for web and desktop clients.

mod config;
mod error;
mod handlers;

use axum::http::HeaderValue;
use axum::routing::{get, post};
use axum::Router;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    // Initialize tracing (respects RUST_LOG env var).
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("isso51_api=info,tower_http=info")),
        )
        .init();

    let api_routes = Router::new()
        .route("/health", get(handlers::health))
        .route("/calculate", post(handlers::calculate))
        .route("/schemas/{name}", get(handlers::get_schema));

    let cors = CorsLayer::new()
        .allow_origin(
            config::CORS_ORIGINS
                .iter()
                .filter_map(|o| o.parse::<HeaderValue>().ok())
                .collect::<Vec<_>>(),
        )
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::OPTIONS,
        ])
        .allow_headers([axum::http::header::CONTENT_TYPE]);

    let app = Router::new()
        .nest(config::API_PREFIX, api_routes)
        .layer(cors)
        .layer(TraceLayer::new_for_http());

    let addr = format!("0.0.0.0:{}", config::PORT);
    tracing::info!("ISSO 51 API listening on {addr}");

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
