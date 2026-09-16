mod config;
mod db;
mod domain;
mod error;
mod routes;
mod state;

use config::Config;
use state::AppState;

#[tokio::main]
async fn main() {
    let command = std::env::args().nth(1).unwrap_or_else(|| "serve".to_string());

    match command.as_str() {
        "bootstrap" => {
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

            let pool = db::create_pool(&config.database_url).await.unwrap_or_else(|err| {
                eprintln!("error: failed to create database pool: {err}");
                std::process::exit(1);
            });

            let count = db::run_migrations(&pool).await.unwrap_or_else(|err| {
                eprintln!("error: failed to run migrations: {err}");
                std::process::exit(1);
            });

            tracing::info!("database bootstrapped at {}", config.database_url);
            tracing::info!("{count} migration(s) applied");
        }
        "migrate" => {
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

            let pool = db::create_pool(&config.database_url).await.unwrap_or_else(|err| {
                eprintln!("error: failed to create database pool: {err}");
                std::process::exit(1);
            });

            let count = db::run_migrations(&pool).await.unwrap_or_else(|err| {
                eprintln!("error: failed to run migrations: {err}");
                std::process::exit(1);
            });

            tracing::info!("{count} migration(s) applied");
        }
        "serve" => {
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

            let pool = db::create_pool(&config.database_url).await.unwrap_or_else(|err| {
                eprintln!("error: failed to create database pool: {err}");
                std::process::exit(1);
            });

            db::check_migrations(&pool).await.unwrap_or_else(|err| {
                eprintln!("error: database schema is behind: {err}");
                eprintln!("hint: run `backend migrate` to apply pending migrations");
                std::process::exit(1);
            });

            let state = AppState::new(config.clone(), pool);

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
        other => {
            eprintln!("error: unknown command \"{other}\"; expected: serve, migrate, bootstrap");
            std::process::exit(2);
        }
    }
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
