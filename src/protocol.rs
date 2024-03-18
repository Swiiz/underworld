use network::{
    protocol::{NetworkProtocol, Packet},
    Client,
};
use serde::{Deserialize, Serialize};

pub const SERVER_PORT: u16 = 4467;

#[derive(Serialize, Deserialize, Clone)]
pub struct ClientPingPacket;
impl Packet for ClientPingPacket {
    type Side = Client;
}

pub fn protocol() -> NetworkProtocol {
    NetworkProtocol::new().with_packet::<ClientPingPacket>()
}
