mod config;
mod domain;
mod error;
mod routes;
mod state;

use config::Config;
use state::AppState;

#[tokio::main]
async fn main() {
    let mut args = std::env::args().skip(1);
    match args.next() {
        Some(other) if other != "serve" => {
            eprintln!("error: unknown command \"{other}\"; expected: serve");
            std::process::exit(2);
        }
        _ => {}
    }

    let config = Config::load().unwrap_or_else(|err| {
        eprintln!("error: {err}");
        std::process::exit(1);
    });

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_new(&config.log_level)
                .unwrap_or_else(|e| {
                    eprintln!("warning: invalid log level \"{}\", falling back to info: {e}", config.log_level);
                    "backend=info,tower_http=info".into()
                }),
        )
        .init();

    let state = AppState::new(config.clone());

    let app = routes::router().with_state(state);

    tracing::info!(bind = %config.bind_address, "starting server");

    let listener = tokio::net::TcpListener::bind(config.bind_address)
        .await
        .unwrap_or_else(|err| {
            eprintln!("error: failed to bind to {}: {err}", config.bind_address);
            std::process::exit(1);
        });

    tracing::info!(bind = %config.bind_address, "listening");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("server error");
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to listen for SIGINT");
        tracing::info!("received SIGINT, shutting down");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
        tracing::info!("received SIGTERM, shutting down");
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
