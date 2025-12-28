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

    socket.on("room_create_request", on_client_room_create);
    socket.on("room_leave_request", on_client_room_leave);
}

async fn on_client_room_leave(
    socket: SocketRef,
    Extension(ext): Extension<UserData>,
    State(state): State<AppState>,
) {
    info!("User {} leave room request", ext.user_id);
    let socket_rooms = socket.rooms().clone();

    if socket_rooms.is_empty() {
        info!("User {} is not in a room. Cancelling", ext.user_id);
        socket
            .emit("err_not_in_room", "You are currently not in a room")
            .unwrap_or_default();
        return;
    }

    let r = match socket_rooms.get(0) {
        Some(v) => v,
        None => {
            socket
                .emit("err_invalid_room", "Unknown room")
                .unwrap_or_default();
            return;
        }
    };

    let rid: u128 = match r.parse() {
        Ok(v) => v,
        Err(_) => {
            socket
                .emit("err_invalid_room", "Unknown room")
                .unwrap_or_default();
            return;
        }
    };

    socket.leave_all();

    let mut session_manager = state.session_manager.lock().await;

    info!("User {} left room", ext.user_id);
    let room = match session_manager.get_room(rid) {
        Some(v) => v.clone(),
        None => {
            socket
                .emit("err_invalid_room", "Unknown room")
                .unwrap_or_default();
            return;
        }
    };

    if room.player_count.eq(&0) {
        session_manager.delete_room(room.id);
    }

    socket.emit("room_leave", "").unwrap_or_default();
}

async fn on_client_room_create(
    socket: SocketRef,
    Data(data): Data<RoomCreateRequest>,
    State(state): State<AppState>,
    Extension(ext): Extension<UserData>,
) {
    info!("Room create request from user {}", ext.user_id);
    match data.validate() {
        Ok(_) => {}
        Err(e) => {
            socket
                .emit("err_invalid_room_create_data", &e.clone())
                .unwrap_or_default();
            return;
        }
    }

    if !socket.rooms().is_empty() {
        info!("User {} already in a room", ext.user_id);
        socket
            .broadcast()
            .emit("err_already_in_room", "You are already in a room")
            .await
            .unwrap_or_default();
        return;
    }

    let id: u128 = state.snowflake.lock().await.next_id().await;

    socket.join([id.to_string()]);

    let room =
        state
            .session_manager
            .lock()
            .await
            .create_room(id, ext.user_id.clone(), data.player_number);

    let res = serde_json::to_string(&room).unwrap_or_default();

    info!(
        "Room {} created for user {}. Automatically joining...",
        id, ext.user_id
    );

    socket.emit("room_create", &res).unwrap_or_default();
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
