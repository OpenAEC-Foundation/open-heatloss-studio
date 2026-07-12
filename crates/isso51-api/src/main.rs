//! ISSO 51 REST API server.
//!
//! Axum-based API that wraps isso51-core for web and desktop clients.
//! Authentication is delegated to Authentik via the Caddy forward_auth
//! outpost; this service trusts the `X-Authentik-*` headers injected on
//! the internal Docker network. See `crate::auth` for details.

mod auth;
mod config;
mod cors;
mod error;
mod handlers;
mod ratelimit;
mod state;

use std::collections::HashSet;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;

use axum::extract::DefaultBodyLimit;
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::Router;
use sqlx::sqlite::{
    SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous,
};
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

use openaec_cloud::TenantsRegistry;

use crate::config::Config;
use crate::ratelimit::RateLimiter;
use crate::state::AppState;

/// Body-limiet voor de publieke reken-routes. De grootste realistische
/// thermal-import-fixture (`tests/fixtures/thermal-import-v11-geometry.json`) is
/// ~190 KB; 2 MB geeft daar ruime hoofdruimte boven (~10×) en begrenst tegelijk
/// het geheugen dat één ongeauthenticeerd verzoek kan claimen. Voorheen liep dit
/// op de impliciete axum-default van 2 MB; nu expliciet zodat de grens niet
/// meebeweegt met framework-defaults.
const COMPUTE_BODY_LIMIT: usize = 2 * 1024 * 1024;

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
    // WAL + busy_timeout zijn essentieel voor concurrency: met de default
    // (rollback journal, 0 ms busy timeout) geeft een tweede gelijktijdige
    // schrijver direct `SQLITE_BUSY` i.p.v. te wachten, wat als HTTP 500 —
    // een stille save-failure — bij de client belandt. WAL laat lezers en
    // één schrijver parallel toe; de busy_timeout laat concurrente schrijvers
    // tot 5 s wachten op de write-lock i.p.v. meteen te falen.
    // `create_if_missing(true)` is het equivalent van `?mode=rwc` in de URL.
    let connect_options = SqliteConnectOptions::from_str(&config.database_url)
        .expect("Invalid DATABASE_URL")
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .busy_timeout(Duration::from_secs(5))
        .synchronous(SqliteSynchronous::Normal);

    let db = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(connect_options)
        .await
        .expect("Failed to connect to database");

    // Run migrations (SQLx executes one statement at a time).
    run_migrations(&db).await;

    tracing::info!("Database initialized");

    // --- Authentication ---
    // Identity is provided by Authentik via the Caddy forward_auth outpost
    // (`X-Authentik-*` headers). No JWT/JWKS state to initialise here.
    tracing::info!("Auth mode: Authentik forward_auth headers");

    // --- Multi-tenant cloud storage ---
    let tenants = if let Some(ref path) = config.tenants_config {
        TenantsRegistry::load(path).unwrap_or_default()
    } else {
        TenantsRegistry::load_from_env().unwrap_or_default()
    };

    if tenants.is_configured() {
        tracing::info!(
            tenants = ?tenants.slugs(),
            "Cloud storage enabled for {} tenant(s)",
            tenants.slugs().len()
        );
    } else {
        tracing::info!("No tenants configured — cloud storage disabled");
    }

    let app_state = AppState::new(
        db,
        config.reports_api_url.clone(),
        config.reports_api_key.clone(),
        config.reports_api_service_token.clone(),
        config.ifc_tool_path.clone(),
        tenants,
        config.default_tenant.clone(),
    );

    // --- Routes ---
    // Publieke reken-routes: geen auth (bewust — publieke reken-API), maar
    // CPU-intensief. Daarom een eigen, strak begrensd `DefaultBodyLimit` plus
    // een per-IP rate-limit (default 30/min, env `COMPUTE_RATE_LIMIT_PER_MIN`).
    // Zie `crate::ratelimit`. Layer-volgorde: de rate-limit staat als laatste
    // (= buitenste), zodat een geweigerd verzoek al 429 krijgt vóór de body
    // gelezen of de handler geraakt wordt.
    let rate_limiter = RateLimiter::from_env();
    let compute = Router::new()
        .route("/calculate", post(handlers::calculate))
        .route("/calculate_v2", post(handlers::calculate_v2))
        .route("/calculate/ifcx", post(handlers::calculate_ifcx_handler))
        .route("/import/thermal", post(handlers::thermal_import_handler))
        .route("/cooling/simplified", post(handlers::simplified_cooling))
        .route("/tojuli/calculate", post(handlers::tojuli_calculate))
        .route("/beng/calculate", post(handlers::beng_calculate))
        .layer(DefaultBodyLimit::max(COMPUTE_BODY_LIMIT))
        .layer(axum::middleware::from_fn_with_state(
            rate_limiter.clone(),
            ratelimit::rate_limit,
        ));

    let public = Router::new()
        .route("/health", get(handlers::health))
        .route("/schemas", get(handlers::list_schemas))
        .route("/schemas/{name}", get(handlers::get_schema))
        .merge(compute);

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
        )
        .route("/report/generate", post(handlers::generate_report))
        // `project_data` is sinds de envelope-pariteit fix de volledige
        // opslag-envelope (project + result + modeller-geometrie + sidecars).
        // Grote projecten kunnen daarmee de axum-default van 2 MB
        // overschrijden; een 413 zou een stille save-failure zijn. De
        // onderlegger (base64 PDF/afbeelding) blijft bewust uit de envelope —
        // 20 MB is ruime headroom voor puur project-JSON.
        .layer(DefaultBodyLimit::max(20 * 1024 * 1024));

    // Cloud storage routes (authenticated).
    let cloud_routes = Router::new()
        .route("/status", get(handlers::cloud_status))
        .route("/projects", get(handlers::cloud_list_projects))
        .route(
            "/projects/{project}/models",
            get(handlers::cloud_list_models),
        )
        .route(
            "/projects/{project}/calculations",
            get(handlers::cloud_list_calculations),
        )
        .route(
            "/projects/{project}/save",
            post(handlers::cloud_save_calculation),
        );

    // IFC import with 100 MB body limit (default is 2 MB).
    let ifc_routes = Router::new()
        .route("/import", post(handlers::import_ifc))
        .layer(DefaultBodyLimit::max(100 * 1024 * 1024));

    // --- CORS ---
    // Tenant-aware: scan <OPENAEC_TENANTS_ROOT>/<slug>/tenant.yaml voor
    // allowed_origins. Bij ontbrekende config of lege set valt de layer
    // terug op de statische `cors_origins` lijst uit env `CORS_ORIGINS`
    // (backward-compat met pre-B-4 deployments). Zie `crate::cors`.
    let tenants_root = std::env::var("OPENAEC_TENANTS_ROOT")
        .ok()
        .filter(|s| !s.is_empty())
        .map(PathBuf::from);
    let include_dev = std::env::var("OPENAEC_ENV")
        .unwrap_or_else(|_| "development".into())
        != "production";
    let origins = match tenants_root.as_deref() {
        Some(root) if root.exists() => {
            tracing::info!(
                tenants_root = %root.display(),
                include_dev,
                "CORS: laden tenant origins"
            );
            cors::load_tenant_origins(root, include_dev)
        }
        Some(root) => {
            tracing::warn!(
                tenants_root = %root.display(),
                "OPENAEC_TENANTS_ROOT wijst naar niet-bestaande directory — fallback op CORS_ORIGINS env"
            );
            HashSet::new()
        }
        None => {
            tracing::info!(
                "OPENAEC_TENANTS_ROOT niet gezet — fallback op CORS_ORIGINS env lijst"
            );
            HashSet::new()
        }
    };
    let cors = cors::build_cors_layer(origins, config.cors_origins.clone());

    // --- App ---
    let mut app = Router::new()
        .nest(
            config::API_PREFIX,
            public
                .merge(protected)
                .nest("/cloud", cloud_routes)
                .nest("/ifc", ifc_routes),
        )
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
