use std::{
    error::Error,
    fmt::{Display, Formatter, Result},
    result,
};

use serde::{Deserialize, Serialize};
use socketioxide::{
    extract::{Data, Extension, SocketRef, State},
    socket::DisconnectReason,
};
use tracing::info;
use validator::Validate;

use crate::AppState;

#[derive(Debug, Clone)]
pub struct UserData {
    user_id: String,
}

#[derive(Debug, Validate, Clone, Copy, Serialize, Deserialize)]
struct RoomCreateRequest {
    #[validate(range(min = 3, max = 50))]
    pub player_number: u8,
}

#[derive(Debug)]
pub struct AuthError;
impl Display for AuthError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "AuthError")
    }
}
impl Error for AuthError {}

pub async fn on_connect(socket: SocketRef, Extension(ext): Extension<UserData>) {
    info!("Connection from user {}", ext.user_id);

    socket.on_disconnect(
        async |_: SocketRef, reason: DisconnectReason, Extension(ext): Extension<UserData>| {
            info!("User {} disconnected. Reason {}", ext.user_id, reason);
        },
    );

    socket.on("room_create_request", on_client_room_create_request);
}

async fn on_client_room_create_request(
    socket: SocketRef,
    Data(data): Data<RoomCreateRequest>,
    State(state): State<AppState>,
    Extension(ext): Extension<UserData>,
) {
    info!("Room create request from user {}", ext.user_id);
    match data.validate() {
        Ok(_) => {}
        Err(e) => {
            socket.emit("error", &e.clone()).unwrap_or_default();
            return;
        }
    }

    if !socket.rooms().is_empty() {
        info!("User {} already in a room", ext.user_id);
        socket
            .broadcast()
            .emit("already_in_room", "You are already in a room")
            .await
            .unwrap_or_default();
        return;
    }

    let id: u128 = state.snowflake.lock().await.next_id().await;

    socket.join([id.to_string()]);

    let room = state
        .session_manager
        .lock()
        .await
        .create_room(id, data.player_number);

    let res = serde_json::to_string(&room).unwrap_or_default();

    info!(
        "Room {} created for user {}. Automatically joining...",
        id, ext.user_id
    );

    socket.emit("room_created", &res).unwrap_or_default();
}

pub async fn verify_auth_header(
    s: SocketRef,
    State(state): State<AppState>,
) -> result::Result<(), AuthError> {
    let headers = &s.req_parts().headers;
    let auth_header = match headers.get("Authorization") {
        Some(v) => v.to_str().unwrap_or_default(),
        None => return Err(AuthError),
    };

    if !auth_header.starts_with("Bearer ") {
        return Err(AuthError);
    }

    let tokens: Vec<&str> = auth_header.split(" ").collect();

    if tokens.len() < 2 {
        return Err(AuthError);
    }

    let jwt = tokens[1];
    let sub = match state.jwt_utils.verify(jwt) {
        Ok(v) => v,
        Err(_) => return Err(AuthError),
    };

    s.extensions.insert(UserData { user_id: sub });

    Ok(())
}
