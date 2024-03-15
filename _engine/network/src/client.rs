use std::any::TypeId;

use crate::commons::{
    process_conn, ClientSide, Connection, ConnectionState, HandlePacket, NetworkProtocol, Packet,
    RawPacket, ReceivedPackets,
};

pub struct NetworkClient {
    connection: Option<Box<dyn Connection<ClientSide>>>,
    protocol: NetworkProtocol,
    received: ReceivedPackets,
}

impl NetworkClient {
    pub fn new(protocol: NetworkProtocol) -> Self {
        Self {
            protocol,
            connection: None,
            received: ReceivedPackets::new(),
        }
    }

    pub fn set_connection(&mut self, mut conn: impl Connection<ClientSide> + 'static) {
        conn.configure();
        self.connection = Some(Box::new(conn));
        println!(
            "INFO: Connection established with remote: {}",
            self.connection.as_ref().unwrap().remote_addr()
        );
    }

    pub fn is_connected(&self) -> bool {
        self.connection.is_some()
    }

    pub fn emit<P: Packet<Side = ClientSide>>(&mut self, packet: P) {
        self.connection
            .as_mut()
            .map(|c| c.emit(RawPacket::new(packet, &self.protocol), &self.protocol));
    }

    pub fn update(&mut self) {
        self.received.clear();
        if let Some(conn) = &mut self.connection {
            let (packets, state) = process_conn(conn.as_mut(), &self.protocol);
            for p in packets {
                self.received.push(p);
            }
            if state == ConnectionState::ShouldClose {
                self.connection = None;
            }
        }
    }
}

impl HandlePacket for NetworkClient {
    #[allow(private_interfaces)]
    fn received(&self) -> &ReceivedPackets {
        &self.received
    }
    fn protocol(&self) -> &NetworkProtocol {
        &self.protocol
    }
}
