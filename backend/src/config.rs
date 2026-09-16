use std::env;
use std::net::SocketAddr;

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
    InvalidLogFilter {
        setting: &'static str,
        value: String,
        reason: String,
    },
    NonUnicodeValue(&'static str),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::InvalidBindAddress(val) => {
                write!(f, "invalid BIND_ADDRESS: \"{val}\" — expected a socket address like 127.0.0.1:3000")
            }
            ConfigError::InvalidLogFilter {
                setting,
                value,
                reason,
            } => write!(f, "invalid {setting}: \"{value}\": {reason}"),
            ConfigError::NonUnicodeValue(setting) => {
                write!(f, "invalid {setting}: value is not valid Unicode")
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

        let (log_setting, log_level) = match env::var("LOG_LEVEL") {
            Ok(value) => ("LOG_LEVEL", value),
            Err(env::VarError::NotPresent) => match env::var("RUST_LOG") {
                Ok(value) => ("RUST_LOG", value),
                Err(env::VarError::NotPresent) => {
                    ("LOG_LEVEL", DEFAULT_LOG_LEVEL.to_string())
                }
                Err(env::VarError::NotUnicode(_)) => {
                    return Err(ConfigError::NonUnicodeValue("RUST_LOG"));
                }
            },
            Err(env::VarError::NotUnicode(_)) => {
                return Err(ConfigError::NonUnicodeValue("LOG_LEVEL"));
            }
        };

        tracing_subscriber::EnvFilter::try_new(&log_level).map_err(|err| {
            ConfigError::InvalidLogFilter {
                setting: log_setting,
                value: log_level.clone(),
                reason: err.to_string(),
            }
        })?;

        Ok(Self {
            bind_address,
            database_url,
            log_level,
        })
    }
}
