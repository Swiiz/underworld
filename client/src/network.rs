use std::{
    io::{ErrorKind, Write},
    net::{TcpStream, ToSocketAddrs},
};

use common::{
    logger::{error, warn},
    network::{proto::network_protocol, AnyPacket, Protocol},
};
use serde::Serialize;

pub struct NetworkClient {
    protocol: Protocol,
    socket: TcpStream,
    packet_queue: Vec<Box<[u8]>>,
}

impl NetworkClient {
    pub fn connect_to(address: impl ToSocketAddrs) -> Self {
        let socket = TcpStream::connect(address).expect("Invalid address");
        socket
            .set_nonblocking(true)
            .expect("Failed to set socket to non-blocking");

        Self {
            socket,
            protocol: network_protocol(),
            packet_queue: Vec::new(),
        }
    }

    pub fn handle_packets(&mut self, mut handler: impl FnMut(&mut Self, AnyPacket)) {
        loop {
            match self.protocol.decode(&self.socket) {
                Ok(packet) => {
                    handler(self, packet);
                }
                Err(common::network::ErrorKind::Io(e))
                    if [ErrorKind::WouldBlock].contains(&e.kind()) =>
                {
                    break;
                }
                Err(common::network::ErrorKind::Io(e))
                    if [
                        ErrorKind::UnexpectedEof,
                        ErrorKind::ConnectionReset,
                        ErrorKind::ConnectionAborted,
                        ErrorKind::ConnectionRefused,
                    ]
                    .contains(&e.kind()) =>
                {
                    error!("Client disconnected: {}", self.socket.peer_addr().unwrap());
                    panic!("");
                }
                Err(e) => {
                    warn!("Failed to decode packet from client: {e:?}");
                    continue;
                }
            }
        }
    }

    pub fn send<T: Serialize + 'static>(&mut self, packet: &T) {
        let data = self.protocol.encode(packet);
        self.socket
            .write_all(&data)
            .unwrap_or_else(|e| warn!("Failed to write packet to client: {e:?}"));
    }

    pub fn flush(&mut self) {
        self.socket
            .flush()
            .unwrap_or_else(|e| warn!("Failed to flush socket to server: {e:?}"));
    }
}
