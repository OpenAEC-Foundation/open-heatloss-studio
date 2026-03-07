//! ISSO 51 REST API server.
//!
//! Axum-based API that wraps isso51-core for web and desktop clients.
//! Supports optional OIDC authentication for user/project management.

mod auth;
mod config;
mod error;
mod handlers;
mod state;

use axum::http::{header, HeaderValue, Method, StatusCode};
use axum::routing::{get, post};
use axum::Router;
use sqlx::sqlite::SqlitePoolOptions;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

use crate::auth::JwksCache;
use crate::config::Config;
use crate::state::AppState;

#[tokio::main]
async fn main() {
    // Load .env file if present (development convenience).
    let _ = dotenvy::dotenv();

    // Initialize tracing (respects RUST_LOG env var).
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("isso51_api=info,tower_http=info")),
        )
        .init();

    let config = Config::from_env();

    // --- Database ---
    let db = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await
        .expect("Failed to connect to database");

    // Run migrations (SQLx executes one statement at a time).
    run_migrations(&db).await;

    tracing::info!("Database initialized");

    // --- OIDC JWKS ---
    let jwks = init_jwks(&config).await;

    // Spawn background JWKS key refresh (every 15 min).
    if let Some(ref cache) = jwks {
        let cache = cache.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(900));
            interval.tick().await; // skip immediate first tick
            loop {
                interval.tick().await;
                if let Err(e) = cache.refresh_keys().await {
                    tracing::warn!("JWKS refresh failed: {e}");
                }
            }
        });
    }

    let app_state = AppState::new(
        db,
        jwks,
        config.reports_api_url.clone(),
        config.reports_api_key.clone(),
    );

    // --- Routes ---
    let public = Router::new()
        .route("/health", get(handlers::health))
        .route("/calculate", post(handlers::calculate))
        .route("/schemas", get(handlers::list_schemas))
        .route("/schemas/{name}", get(handlers::get_schema))
        .route("/report/generate", post(handlers::generate_report));

    let protected = Router::new()
        .route("/me", get(handlers::get_profile))
        .route(
            "/projects",
            get(handlers::list_projects).post(handlers::create_project),
        )
        .route(
            "/projects/{id}",
            get(handlers::get_project)
                .put(handlers::update_project)
                .delete(handlers::delete_project),
        )
        .route(
            "/projects/{id}/calculate",
            post(handlers::calculate_and_save),
        );

    // --- CORS ---
    let cors = CorsLayer::new()
        .allow_origin(
            config
                .cors_origins
                .iter()
                .filter_map(|o| o.parse::<HeaderValue>().ok())
                .collect::<Vec<_>>(),
        )
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION]);

    // --- App ---
    let mut app = Router::new()
        .nest(config::API_PREFIX, public.merge(protected))
        .with_state(app_state)
        .layer(cors)
        .layer(TraceLayer::new_for_http());

    // --- Static file serving (SPA fallback) ---
    // NOTE: We cannot use ServeDir::not_found_service() because tower-http 0.6
    // always overrides the fallback response status to 404.
    // Instead, we wrap ServeDir in a custom service that intercepts 404s and
    // returns index.html with 200 for SPA client-side routing.
    if let Some(ref static_dir) = config.static_dir {
        let index_html: bytes::Bytes = std::fs::read(format!("{static_dir}/index.html"))
            .expect("index.html not found in static_dir")
            .into();

        let serve_dir = ServeDir::new(static_dir.clone());

        let spa_fallback = tower::service_fn(move |req: axum::extract::Request| {
            let serve_dir = serve_dir.clone();
            let index_html = index_html.clone();
            async move {
                use tower::ServiceExt;
                let resp = serve_dir.oneshot(req).await?;
                if resp.status() == StatusCode::NOT_FOUND {
                    Ok(axum::response::Response::builder()
                        .status(StatusCode::OK)
                        .header("content-type", "text/html; charset=utf-8")
                        .body(axum::body::Body::from(index_html))
                        .unwrap())
                } else {
                    Ok(resp.map(axum::body::Body::new))
                }
            }
        });

        app = app.fallback_service(spa_fallback);
        tracing::info!("Serving static files from {static_dir}");
    }

    let addr = format!("0.0.0.0:{}", config.port);
    tracing::info!("ISSO 51 API listening on {addr}");

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

/// Initialize JWKS cache from OIDC issuer if configured.
async fn init_jwks(config: &Config) -> Option<JwksCache> {
    let (Some(issuer), Some(audience)) = (&config.oidc_issuer, &config.oidc_audience) else {
        tracing::warn!(
            "OIDC_ISSUER or OIDC_AUDIENCE not set — authentication disabled. \
             Protected routes will return 401."
        );
        return None;
    };

    match JwksCache::from_issuer(issuer, audience).await {
        Ok(cache) => {
            tracing::info!("OIDC authentication enabled (issuer: {issuer})");
            Some(cache)
        }
        Err(e) => {
            tracing::error!("Failed to initialize OIDC: {e}");
            tracing::warn!("Protected routes will return 401.");
            None
        }
    }
}

/// Run database migrations. Each statement is executed individually because
/// SQLx's `execute` only supports single statements.
async fn run_migrations(db: &sqlx::SqlitePool) {
    let migration = include_str!("../migrations/001_initial.sql");
    for statement in migration.split(';') {
        let trimmed = statement.trim();
        if trimmed.is_empty() {
            continue;
        }
        sqlx::query(trimmed)
            .execute(db)
            .await
            .unwrap_or_else(|e| panic!("Migration failed on: {trimmed}\nError: {e}"));
    }
}
