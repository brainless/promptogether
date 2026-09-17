mod config;
mod db;
mod domain;
mod error;
mod routes;
mod state;

use config::{Config, WorkerConfig};
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

            init_tracing(&config.log_level);

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

            init_tracing(&config.log_level);

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

            init_tracing(&config.log_level);

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
        "worker" => {
            let config = Config::load().unwrap_or_else(|err| {
                eprintln!("error: {err}");
                std::process::exit(1);
            });

            let worker_config = WorkerConfig::load().unwrap_or_else(|err| {
                eprintln!("error: {err}");
                std::process::exit(1);
            });

            init_tracing(&config.log_level);

            let pool = db::create_pool(&config.database_url).await.unwrap_or_else(|err| {
                eprintln!("error: failed to create database pool: {err}");
                std::process::exit(1);
            });

            db::check_migrations(&pool).await.unwrap_or_else(|err| {
                eprintln!("error: database schema is behind: {err}");
                eprintln!("hint: run `backend migrate` to apply pending migrations");
                std::process::exit(1);
            });

            domain::jobs_worker::run(pool, worker_config).await.unwrap_or_else(|err| {
                eprintln!("error: worker failed: {err}");
                std::process::exit(1);
            });
        }
        "enqueue-fixture" => {
            let should_fail = std::env::args().any(|a| a == "--fail");

            let config = Config::load().unwrap_or_else(|err| {
                eprintln!("error: {err}");
                std::process::exit(1);
            });

            init_tracing(&config.log_level);

            let pool = db::create_pool(&config.database_url).await.unwrap_or_else(|err| {
                eprintln!("error: failed to create database pool: {err}");
                std::process::exit(1);
            });

            db::run_migrations(&pool).await.unwrap_or_else(|err| {
                eprintln!("error: failed to run migrations: {err}");
                std::process::exit(1);
            });

            let repo = domain::jobs_repo::JobRepository::new(&pool);
            let new_job = domain::jobs::enqueue(&domain::jobs::FixturePayload { should_fail })
                .unwrap_or_else(|err| {
                    eprintln!("error: failed to serialize payload: {err}");
                    std::process::exit(1);
                });
            let id = repo.enqueue(&new_job).await.unwrap_or_else(|err| {
                eprintln!("error: failed to enqueue job: {err}");
                std::process::exit(1);
            });

            println!("{id}");
        }
        "seed-gallery" => {
            let config = Config::load().unwrap_or_else(|err| {
                eprintln!("error: {err}");
                std::process::exit(1);
            });

            init_tracing(&config.log_level);

            let pool = db::create_pool(&config.database_url).await.unwrap_or_else(|err| {
                eprintln!("error: failed to create database pool: {err}");
                std::process::exit(1);
            });

            db::check_migrations(&pool).await.unwrap_or_else(|err| {
                eprintln!("error: database schema is behind: {err}");
                eprintln!("hint: run `backend migrate` to apply pending migrations");
                std::process::exit(1);
            });

            let summary = domain::gallery_seed::seed(&pool).await.unwrap_or_else(|err| {
                eprintln!("error: failed to seed gallery: {err}");
                std::process::exit(1);
            });

            println!("seed complete: {} project(s) upserted, {} file(s) upserted, {} file(s) removed", summary.projects_upserted, summary.files_upserted, summary.files_removed);
        }
        other => {
            eprintln!("error: unknown command \"{other}\"; expected: serve, migrate, bootstrap, worker, enqueue-fixture, seed-gallery");
            std::process::exit(2);
        }
    }
}

fn init_tracing(log_level: &str) {
    let filter = tracing_subscriber::EnvFilter::try_new(log_level)
        .expect("log filter was validated while loading configuration");
    tracing_subscriber::fmt().with_env_filter(filter).init();
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
