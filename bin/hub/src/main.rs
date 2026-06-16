use std::sync::Arc;

use hub_core::{state::AppState, types::GatewayConfig};
use tokio::signal::unix::SignalKind;
use tower_http::trace::{DefaultMakeSpan, TraceLayer};
use tracing::{Level, info};

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let log_level = std::env::var("RUST_LOG")
        .ok()
        .and_then(|level| level.parse::<Level>().ok())
        .unwrap_or(Level::WARN);

    tracing_subscriber::fmt().json().with_max_level(log_level).init();

    info!("Starting Hub LLM Gateway...");

    // Initialize ecdysis for graceful restart (SIGUSR1 triggers upgrade)
    let mut builder = ecdysis::tokio_ecdysis::TokioEcdysisBuilder::new(SignalKind::user_defined1())
        .map_err(|e| eyre::eyre!("Failed to initialize ecdysis: {e}"))?;

    // Register stop signals for graceful shutdown
    builder
        .stop_on_signal(SignalKind::interrupt())
        .map_err(|e| eyre::eyre!("Failed to register SIGINT handler: {e}"))?;
    builder
        .stop_on_signal(SignalKind::terminate())
        .map_err(|e| eyre::eyre!("Failed to register SIGTERM handler: {e}"))?;

    // Signal readiness and get the ecdysis instance
    let (ecdysis, upgrade_future) =
        builder.ready().map_err(|e| eyre::eyre!("Failed to signal readiness: {e}"))?;

    // Load configuration
    let config_path =
        std::env::var("CONFIG_FILE_PATH").unwrap_or_else(|_| "config.yaml".to_string());

    let config = if std::path::Path::new(&config_path).exists() {
        hub_core::config::load_config(std::path::Path::new(&config_path))?
    } else {
        info!("No config file found, using default configuration");
        GatewayConfig::default()
    };

    let app_state = Arc::new(AppState::new(config)?);

    // Create gateway router
    let gateway_router = hub_gateway::routes::create_router(app_state.clone());

    // Apply tracing layer
    let app = gateway_router.layer(
        TraceLayer::new_for_http().make_span_with(DefaultMakeSpan::default().include_headers(true)),
    );

    // Check if database mode is enabled
    let hub_mode = std::env::var("HUB_MODE").unwrap_or_else(|_| "yaml".to_string());

    if hub_mode == "database" {
        // Database mode: start both gateway and management API
        let database_url = std::env::var("DATABASE_URL")
            .map_err(|_| eyre::eyre!("DATABASE_URL must be set in database mode"))?;

        let pool = sqlx::PgPool::connect(&database_url).await?;

        // Run migrations
        sqlx::migrate!("../../migrations").run(&pool).await?;

        // Create management router
        let management_router = hub_management::state::create_management_router(pool);

        // Build management listener via ecdysis (inherits socket on upgrade)
        let management_port =
            std::env::var("MANAGEMENT_PORT").unwrap_or_else(|_| "8080".to_string());
        let management_addr: std::net::SocketAddr = format!("0.0.0.0:{management_port}")
            .parse()
            .map_err(|e| eyre::eyre!("Invalid management address: {e}"))?;

        let management_std = ecdysis
            .std_ecdysis()
            .listen_tcp(management_addr)
            .map_err(|e| eyre::eyre!("Failed to create management listener: {e}"))?;
        management_std.set_nonblocking(true)?;
        let management_listener = tokio::net::TcpListener::from_std(management_std)?;

        info!("Starting management API server on {management_addr}");
        let management_handle = tokio::spawn(async move {
            axum::serve(management_listener, management_router)
                .await
                .expect("Management API server failed");
        });

        // Build gateway listener via ecdysis
        let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
        let addr: std::net::SocketAddr =
            format!("0.0.0.0:{port}").parse().map_err(|e| eyre::eyre!("Invalid address: {e}"))?;

        let std_listener = ecdysis
            .std_ecdysis()
            .listen_tcp(addr)
            .map_err(|e| eyre::eyre!("Failed to create listener: {e}"))?;
        std_listener.set_nonblocking(true)?;
        let listener = tokio::net::TcpListener::from_std(std_listener)?;

        info!("Starting gateway server on {addr}");
        let server_handle = tokio::spawn(async move {
            axum::serve(listener, app).await.expect("Gateway server failed");
        });

        // Wait for upgrade or shutdown signal
        match upgrade_future.await {
            Ok((_, reason)) => {
                info!("Ecdysis exit: {reason:?}");
            }
            Err(e) => {
                tracing::error!("Ecdysis error: {e}");
            }
        }

        let _ = server_handle.await;
        let _ = management_handle.await;
    } else {
        // YAML mode: start only gateway
        let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
        let addr: std::net::SocketAddr =
            format!("0.0.0.0:{port}").parse().map_err(|e| eyre::eyre!("Invalid address: {e}"))?;

        let std_listener = ecdysis
            .std_ecdysis()
            .listen_tcp(addr)
            .map_err(|e| eyre::eyre!("Failed to create listener: {e}"))?;
        std_listener.set_nonblocking(true)?;
        let listener = tokio::net::TcpListener::from_std(std_listener)?;

        info!("Starting gateway server on {addr}");
        let server_handle = tokio::spawn(async move {
            axum::serve(listener, app).await.expect("Gateway server failed");
        });

        // Wait for upgrade or shutdown signal
        match upgrade_future.await {
            Ok((_, reason)) => {
                info!("Ecdysis exit: {reason:?}");
            }
            Err(e) => {
                tracing::error!("Ecdysis error: {e}");
            }
        }

        let _ = server_handle.await;
    }

    info!("Server shut down gracefully");
    Ok(())
}
