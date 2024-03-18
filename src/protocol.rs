use network::{
    protocol::{NetworkProtocol, Packet},
    Client, Server,
};
use serde::{Deserialize, Serialize};

pub const SERVER_PORT: u16 = 4467;

#[derive(Serialize, Deserialize, Clone)]
pub struct ClientPingPacket;
impl Packet for ClientPingPacket {
    type Side = Client;
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ServerPongPacket;
impl Packet for ServerPongPacket {
    type Side = Server;
}

pub fn protocol() -> NetworkProtocol {
    NetworkProtocol::new()
        .with_packet::<ClientPingPacket>()
        .with_packet::<ServerPongPacket>()
}
