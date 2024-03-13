use network::commons::{ClientSide, NetworkProtocol, Packet};
use serde::{Deserialize, Serialize};

pub const SERVER_PORT: u16 = 4467;

#[derive(Serialize, Deserialize, Clone)]
pub struct ClientPingPacket;
impl Packet for ClientPingPacket {
    type Side = ClientSide;
}

pub fn protocol() -> NetworkProtocol {
    NetworkProtocol::new().with_packet::<ClientPingPacket>()
}
