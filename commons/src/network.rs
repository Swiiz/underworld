use network::commons::NetworkProtocol;

pub const SERVER_PORT: u16 = 4467;

pub struct Protocol;

pub enum ClientPacket {}
pub enum ServerPacket {}

impl NetworkProtocol for Protocol {
    type ClientPacket = ClientPacket;
    type ServerPacket = ServerPacket;
}
