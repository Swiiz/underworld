use network::{
    protocol::{NetworkProtocol, Packet},
    Client, Server,
};
use serde::{Deserialize, Serialize};

use crate::world::{ClientLoadWorldPacket, ServerWorldSendChunkPacket, ServerWorldSetTilePacket};

pub const SERVER_PORT: u16 = 4467;

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct ClientPingPacket;
impl Packet for ClientPingPacket {
    type Side = Client;
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct ServerPongPacket;
impl Packet for ServerPongPacket {
    type Side = Server;
}

pub fn protocol() -> NetworkProtocol {
    NetworkProtocol::new()
        .with_packet::<ClientPingPacket>()
        .with_packet::<ServerPongPacket>()
        .with_packet::<ClientLoadWorldPacket>()
        .with_packet::<ServerWorldSendChunkPacket>()
        .with_packet::<ServerWorldSetTilePacket>()
}
