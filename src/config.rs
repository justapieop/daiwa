use serde::{Deserialize, Serialize};
use std::env;

static DEFAULT_LISTEN_ADDRESS: &str = "0.0.0.0:3000";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub listen_address: String,
    pub jwks_url: String,
    pub jwks_iss: String,
}

impl Config {
    pub fn new() -> Self {
        Self {
            listen_address: env::var("LISTEN_ADDRESS")
                .unwrap_or(String::from(DEFAULT_LISTEN_ADDRESS)),
            jwks_url: env::var("JWKS_URL").expect("JWKS_URL should be set"),
            jwks_iss: env::var("JWKS_ISS").expect("JWKS_ISS should be set"),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}
