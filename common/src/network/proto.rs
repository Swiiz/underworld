use super::Protocol;
use crate::core::{spatial::Position, EntityKind};
use ecs::serde::EcsState;
use serde::{Deserialize, Serialize};

///////////////////////////////
//   GAME NETWORK PROTOCOL   //
//                           //
// Clientbound = From Server //
// Serverbound = From Client //
///////////////////////////////

pub mod login {

    use super::*;

    #[derive(Serialize, Deserialize)]
    pub struct ServerboundLoginStart {
        pub username: String,
    }

    #[derive(Serialize, Deserialize)]
    pub struct ClientboundLoginSuccess {
        pub ecs_state: EcsState<SyncComponentSelection>,
    }

    pub fn login_protocol(proto: &mut Protocol) {
        proto
            .add_packet::<ServerboundLoginStart>()
            .add_packet::<ClientboundLoginSuccess>();
    }
}

pub mod play {
    use ecs::{serde::EntityState, AliveEntityId};

    use super::*;

    #[derive(Serialize, Deserialize)]
    pub struct ServerboundSetPlayerPos {
        pub pos: Position,
    }

    #[derive(Serialize, Deserialize)]
    pub struct ClientboundSpawnEntity {
        pub entity: AliveEntityId,
        pub state: EntityState<SyncComponentSelection>,
    }

    #[derive(Serialize, Deserialize)]
    pub struct ClientboundRemoveEntity {
        pub entity: AliveEntityId,
    }

    #[derive(Serialize, Deserialize)]
    pub struct ClientboundSetEntityPosition {
        pub entity: AliveEntityId,
        pub pos: Position,
    }

    pub fn play_protocol(proto: &mut Protocol) {
        proto
            .add_packet::<ServerboundSetPlayerPos>()
            .add_packet::<ClientboundSpawnEntity>()
            .add_packet::<ClientboundRemoveEntity>()
            .add_packet::<ClientboundSetEntityPosition>();
    }
}

pub mod extra {
    use std::time::Instant;

    use super::*;

    #[derive(Serialize, Deserialize)]
    pub struct CommonPing {
        #[serde(with = "serde_millis")]
        pub time: Instant,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub enum ServerboundDisconnect {
        GameClosed,
    }

    pub fn extra_protocol(proto: &mut Protocol) {
        proto
            .add_packet::<CommonPing>()
            .add_packet::<ServerboundDisconnect>();
    }
}

pub type SyncComponentSelection = (EntityKind, Position);

pub fn network_protocol() -> Protocol {
    let mut proto = Protocol::new();

    login::login_protocol(&mut proto);
    play::play_protocol(&mut proto);
    extra::extra_protocol(&mut proto);

    proto
}
