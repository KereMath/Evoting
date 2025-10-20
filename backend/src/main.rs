use axum::{
    routing::{get, post, delete},
    Router,
    Json,
    extract::State,
    http::StatusCode,
};
use serde::Serialize;
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::{CorsLayer, Any};
use tracing::info;

mod handlers;
mod models;
mod db;

#[derive(Clone)]
pub struct AppState {
    db: sqlx::PgPool,
}

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    version: String,
    database: String,
}

async fn health_check(State(state): State<Arc<AppState>>) -> Result<Json<HealthResponse>, StatusCode> {
    // Check database connection
    let db_status = match sqlx::query("SELECT 1").fetch_one(&state.db).await {
        Ok(_) => "connected",
        Err(_) => "disconnected",
    };

    Ok(Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        database: db_status.to_string(),
    }))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .init();

    info!("🚀 Starting E-Voting Server...");

    // Load environment variables
    dotenv::dotenv().ok();

    // Database connection
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://evoting:evoting@postgres:5432/evoting_db".to_string());

    info!("📦 Connecting to database...");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    info!("✅ Database connected successfully");

    // Note: Migrations handled by init.sql in PostgreSQL
    info!("✅ Database schema loaded from init.sql");

    let app_state = Arc::new(AppState {
        db: pool,
    });

    // CORS configuration
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Build router
    let app = Router::new()
        // Health & Auth
        .route("/health", get(health_check))
        .route("/api/auth/login", post(handlers::auth::login))

        // Elections
        .route("/api/elections", get(handlers::elections::list_elections))
        .route("/api/elections", post(handlers::elections::create_election))
        .route("/api/elections/:id", get(handlers::elections::get_election))
        .route("/api/elections/:id", delete(handlers::elections::delete_election))

        // Trustees
        .route("/api/trustees", get(handlers::trustees::list_trustees))
        .route("/api/trustees", post(handlers::trustees::create_trustee))

        // Voters
        .route("/api/voters", get(handlers::voters::list_voters))
        .route("/api/voters", post(handlers::voters::create_voter))
        .route("/api/voters/:id", delete(handlers::voters::delete_voter))
        .route("/api/voters/upload-csv", post(handlers::voters::upload_voters_csv))

        // Events (for logging/monitoring)
        .route("/api/events", get(handlers::events::list_events))

        // Monitoring & Orchestration
        .route("/api/monitoring/topology", get(handlers::monitoring::get_network_topology))
        .route("/api/elections/:id/setup", post(handlers::orchestration::setup_election))

        // Crypto Setup
        .route("/api/elections/:id/crypto-setup", post(handlers::crypto_setup::crypto_setup))
        .route("/api/crypto/parameters/:id", get(handlers::crypto_setup::get_crypto_parameters))

        // Key Generation (DKG)
        .route("/api/elections/:id/keygen/start", post(handlers::keygen::start_keygen))
        .route("/api/elections/:id/keygen/trustee-ready", post(handlers::keygen::trustee_ready))
        .route("/api/elections/:id/keygen/status", get(handlers::keygen::get_keygen_status))
        .route("/api/elections/:id/keygen/progress", post(handlers::keygen::report_progress))

        .layer(cors)
        .with_state(app_state);

    // Start server
    let host = std::env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = std::env::var("SERVER_PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()?;

    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    info!("🌐 Server listening on http://{}:{}", host, port);
    info!("📊 Health check: http://{}:{}/health", host, port);
    info!("🎯 API endpoint: http://{}:{}/api", host, port);

    // Create TCP listener
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    // Serve the app
    axum::serve(listener, app).await?;

    Ok(())
}
