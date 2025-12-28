use std::collections::HashMap;

use serde::{Deserialize, Serialize};

pub struct SessionManager {
    rooms: HashMap<u128, Room>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Room {
    pub player_count: u8,
    pub slots: u8,
    pub id: u128,
    pub host: String,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            rooms: HashMap::new(),
        }
    }

    pub fn create_room(&mut self, id: u128, host: String, slots: u8) -> Room {
        let room = &Room {
            player_count: 1,
            slots,
            id,
            host,
        };

        self.rooms.insert(id, room.clone());

        room.clone()
    }

    pub fn get_room(&self, id: u128) -> Option<&Room> {
        self.rooms.get(&id)
    }

    pub fn delete_room(&mut self, id: u128) {
        match self.rooms.remove(&id) {
            Some(_) => {}
            None => {}
        };
    }
}
