use std::time::Instant;

use ecs::EntityId;

pub struct NetRemoteClient {
    pub username: String,
    pub last_packet: Instant,
    pub entity: EntityId,
}

impl NetRemoteClient {
    pub fn new(username: String, entity: EntityId) -> Self {
        Self {
            username,
            entity,
            last_packet: Instant::now(),
        }
    }
}
