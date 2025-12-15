use std::{env, sync::OnceLock};

pub struct Config {
    pub listen_address: String,
    pub jwks_url: String,
}

impl Config {
    pub fn initialize() -> Self {
        Self {
            listen_address: env::var("LISTEN_ADDRESS")
                .unwrap_or(String::from(DEFAULT_LISTEN_ADDRESS)),
            jwks_url: env::var("JWKS_URL").expect("Missing JWKS_URL environment variable"),
        }
    }
}

static DEFAULT_LISTEN_ADDRESS: &str = "0.0.0.0:3000";
static INSTANCE: OnceLock<Config> = OnceLock::new();

pub fn get_cell() -> &'static OnceLock<Config> {
    &INSTANCE
}

pub fn get() -> &'static Config {
    // config is already initialized at launch
    &INSTANCE.get().expect("config should be initialized")
}
