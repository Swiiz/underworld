use crate::core::spatial::Position;

use super::Protocol;

///////////////////////////////
//   GAME NETWORK PROTOCOL   //
//                           //
// Clientbound = From Server //
// Serverbound = From Client //
///////////////////////////////

pub mod login {
    use ecs::serde::EcsState;
    use serde::{Deserialize, Serialize};

    use crate::{network::Protocol, state::CommonState};

    use super::SyncComponentSelection;

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

pub type SyncComponentSelection = (Position,);

pub fn network_protocol() -> Protocol {
    let mut proto = Protocol::new();

    login::login_protocol(&mut proto);

    proto
}
