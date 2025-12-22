use std::collections::HashMap;

use serde::{Deserialize, Serialize};

pub struct SessionManager {
    rooms: HashMap<u128, Room>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Room {
    player_count: u8,
    slots: u8,
    id: u128,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            rooms: HashMap::new(),
        }
    }

    pub fn create_room(&mut self, id: u128, slots: u8) -> Room {
        let room = &Room {
            player_count: 1,
            slots,
            id,
        };

        self.rooms.insert(id, *room);

        *room
    }
}
