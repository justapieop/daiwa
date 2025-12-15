mod config;
mod events;
mod jwt_utils;

use std::{
    error::Error,
    sync::Arc,
    time::Duration,
};

use axum::{
    Router,
    body::Body,
    http::{HeaderValue, Request, Response},
};
use reqwest::StatusCode;
use socketioxide::SocketIo;
use tokio::net::TcpListener;
use tokio::sync::OnceCell;
use tower_http::{
    auth::AsyncRequireAuthorizationLayer, compression::CompressionLayer, cors::CorsLayer,
    trace::TraceLayer,
};
use tracing::info;

use crate::jwt_utils::JwtUtils;

static CONFIG: OnceCell<config::Config> = OnceCell::const_new();
static JWT_UTILS: OnceCell<Arc<JwtUtils>> = OnceCell::const_new();

pub fn get_config() -> &'static config::Config {
    CONFIG.get().expect("config should be initialized")
}

#[derive(Clone)]
struct AppState {
    jwt: Arc<JwtUtils>,
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenvy::dotenv().unwrap_or_default();
    CONFIG.set(config::Config::new()).unwrap();
    let config = get_config();
    tracing_subscriber::fmt::init();
    info!("Initializing daiwa WS server. Populating configuration");

    info!("Verifying JWKS");

    let jwt_utils = JwtUtils::new(config.jwks_url.clone()).await;
    if let Err(_) = JWT_UTILS.set(Arc::new(jwt_utils)) {
        panic!("JWT_UTILS already initialized");
    }

    let (layer, io) = SocketIo::new_layer();

    io.ns("/", events::on_connect);

    let app: Router<()> = axum::Router::new()
        .layer(layer)
        .layer(CorsLayer::new().max_age(Duration::from_secs(604800)))
        .layer(AsyncRequireAuthorizationLayer::new(
            |mut req: Request<Body>| async {
                let jwt_utils = JWT_UTILS.get().unwrap().clone();
                let headers = req.headers_mut();

                let err_res = Response::builder()
                    .status(StatusCode::UNAUTHORIZED)
                    .body(Body::new(String::from("Unauthorized")))
                    .unwrap_or_default();

                let auth_header = match headers.get("Authorization") {
                    Some(v) => v.to_str().unwrap_or_default(),
                    None => return Err(err_res),
                };

                if !auth_header.starts_with("Bearer ") {
                    return Err(err_res);
                }

                let tokens: Vec<&str> = auth_header.split(" ").collect();

                if tokens.len() < 2 {
                    return Err(err_res);
                }

                let jwt = tokens[1];

                let sub = match jwt_utils.verify(jwt) {
                    Ok(v) => v,
                    Err(_) => return Err(err_res),
                };

                let x_user_id_value = match HeaderValue::from_str(&sub) {
                    Ok(v) => v,
                    Err(_) => return Err(err_res),
                };

                headers.append("X-User-ID", x_user_id_value);

                Ok(req)
            },
        ))
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http());

    info!("Starting WS server");

    let listener = TcpListener::bind(config.listen_address.clone())
        .await
        .expect("address should be bindable");

    info!("Server is listening on {}", config.listen_address);

    axum::serve(listener, app)
        .await
        .expect("listener should start");

    Ok(())
}
