mod config;
mod events;
mod jwt_utils;
mod logging;
mod session;
mod snowflake;

use std::sync::Arc;
use std::{error::Error, time::Duration};

use axum::Router;
use socketioxide::SocketIoBuilder;
use socketioxide::handler::ConnectHandler;
use tokio::net::TcpListener;
use tokio::sync::{Mutex, OnceCell};
use tower_http::{compression::CompressionLayer, cors::CorsLayer, trace::TraceLayer};
use tracing::info;

use crate::session::SessionManager;
use crate::{jwt_utils::JwtUtils, snowflake::Snowflake};

#[derive(Clone)]
struct AppState {
    snowflake: Arc<Mutex<Snowflake>>,
    jwt_utils: Arc<JwtUtils>,
    session_manager: Arc<Mutex<SessionManager>>,
}

static CONFIG: OnceCell<config::Config> = OnceCell::const_new();

pub fn get_config() -> &'static config::Config {
    CONFIG.get().expect("config should be initialized")
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenvy::dotenv().unwrap_or_default();
    CONFIG
        .set(config::Config::new())
        .expect("config should be set");
    let config = get_config();
    logging::setup(
        &std::env::var("LOG_LEVEL").unwrap_or(String::from("info")),
        Some("%Y-%m-%d_%H-%M-%S.log"),
    )
    .unwrap();
    info!("Initializing daiwa WS server. Populating configuration");

    info!("Verifying JWKS");

    let state = AppState {
        snowflake: Arc::new(Mutex::const_new(Snowflake::new())),
        jwt_utils: Arc::new(JwtUtils::new(config.jwks_url.clone()).await),
        session_manager: Arc::new(Mutex::const_new(SessionManager::new())),
    };

    info!("JWK verified successfully. Setting up Socket.IO layer");

    let (layer, io) = SocketIoBuilder::new()
        .with_state(state.clone())
        .build_layer();

    io.ns("/", events::on_connect.with(events::verify_auth_header));

    info!("Initializing axum server");

    let app: Router<()> = axum::Router::new()
        .layer(layer)
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new())
        .layer(CorsLayer::new().max_age(Duration::from_secs(604800)));

    info!("Starting server");

    let listener = TcpListener::bind(config.listen_address.clone())
        .await
        .expect("address should be bindable");

    info!("Server is listening on {}", config.listen_address);

    axum::serve(listener, app)
        .await
        .expect("listener should start");

    Ok(())
}
