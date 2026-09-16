use std::net::SocketAddr;
use std::env;

const DEFAULT_BIND_ADDRESS: &str = "127.0.0.1:3000";
const DEFAULT_DATABASE_URL: &str = "sqlite:promptogether.db?mode=rwc";
const DEFAULT_LOG_LEVEL: &str = "backend=info,tower_http=info";

#[derive(Debug, Clone)]
pub struct Config {
    pub bind_address: SocketAddr,
    pub database_url: String,
    pub log_level: String,
}

#[derive(Debug)]
pub enum ConfigError {
    InvalidBindAddress(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::InvalidBindAddress(val) => {
                write!(f, "invalid BIND_ADDRESS: \"{val}\" — expected a socket address like 127.0.0.1:3000")
            }
        }
    }
}

impl std::error::Error for ConfigError {}

impl Config {
    pub fn load() -> Result<Self, ConfigError> {
        let bind_address = match env::var("BIND_ADDRESS") {
            Ok(val) => val.parse::<SocketAddr>().map_err(|_| ConfigError::InvalidBindAddress(val))?,
            Err(_) => DEFAULT_BIND_ADDRESS.parse().expect("DEFAULT_BIND_ADDRESS is valid"),
        };

        let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| DEFAULT_DATABASE_URL.to_string());

        let log_level = env::var("LOG_LEVEL")
            .or_else(|_| env::var("RUST_LOG"))
            .unwrap_or_else(|_| DEFAULT_LOG_LEVEL.to_string());

        Ok(Self {
            bind_address,
            database_url,
            log_level,
        })
    }
}
