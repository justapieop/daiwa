use serde::{Deserialize, Serialize};
use std::env;

static DEFAULT_LISTEN_ADDRESS: &str = "0.0.0.0:3000";
static DEFAULT_LOG_LEVEL: &str = "info";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub listen_address: String,
    pub jwks_url: String,
    pub jwks_iss: String,
    pub machine_id: u32,
    pub postgres_url: String,
    pub log_level: String,
}

impl Config {
    pub fn new() -> Self {
        Self {
            listen_address: env::var("LISTEN_ADDRESS")
                .unwrap_or(String::from(DEFAULT_LISTEN_ADDRESS)),
            jwks_url: env::var("JWKS_URL").expect("JWKS_URL should be set"),
            jwks_iss: env::var("JWKS_ISS").expect("JWKS_ISS should be set"),
            machine_id: env::var("MACHINE_ID")
                .expect("MACHINE_ID should be set")
                .parse()
                .expect("MACHINE_ID should be a 32-bit unsigned integer"),
            postgres_url: env::var("POSTGRES_URL").expect("POSTGRES_URL should be set"),
            log_level: env::var("LOG_LEVEL").unwrap_or(String::from(DEFAULT_LOG_LEVEL)),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}
