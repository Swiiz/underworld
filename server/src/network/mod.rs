use std::{
    collections::HashMap,
    io::{ErrorKind, Write},
    iter,
    net::{SocketAddr, TcpListener, TcpStream},
    rc::Rc,
    time::{Duration, Instant},
};

use common::{
    logger::{info, warn},
    network::{proto::network_protocol, AnyPacket, Protocol},
};
use remote::NetRemoteClient;
use serde::Serialize;

pub mod remote;
pub struct NetworkServer {
    protocol: Protocol,
    timeout: Duration,
    server: TcpListener,
    connecting_clients: HashMap<SocketAddr, TcpStream>,
    disconnected_clients: HashMap<SocketAddr, NetRemoteClient>,
    clients: HashMap<SocketAddr, (TcpStream, NetRemoteClient)>,
}

impl NetworkServer {
    pub fn new() -> Self {
        let server_address = "127.0.0.1:8888";
        let timeout = Duration::from_secs(5);

        let server = TcpListener::bind(server_address).expect("Failed to bind/configure listener");
        server
            .set_nonblocking(true)
            .expect("Failed to set listener to non-blocking");
        info!("Server listening on {server_address}");

        let connecting_clients = HashMap::new();
        let disconnected_clients = HashMap::new();
        let clients = HashMap::new();

        Self {
            protocol: network_protocol(),
            timeout,
            server,
            connecting_clients,
            disconnected_clients,
            clients,
        }
    }

    pub fn listen_for_connections(&mut self) {
        loop {
            match self.server.accept() {
                Ok((socket, addr)) => {
                    if !self.clients.contains_key(&addr) {
                        self.connecting_clients.insert(addr, socket);
                    } else {
                        warn!(
                            "Client {addr} tried to connect while already waiting for a connection"
                        );
                        self.disconnect(&addr);
                    }
                }
                Err(error) => {
                    if error.kind() == std::io::ErrorKind::WouldBlock {
                        return; // Finish handling packets
                    }

                    warn!("Client disconnected with error: {:?}", error);
                }
            }
        }
    }

    pub fn handle_packets(&mut self, mut handler: impl FnMut(&mut Self, SocketAddr, AnyPacket)) {
        let incoming = self
            .connecting_clients
            .iter()
            .chain(
                self.clients
                    .iter()
                    .map(|(addr, (socket, _))| (addr, socket)),
            )
            .map(|(addr, socket)| {
                (
                    iter::from_fn(|| match self.protocol.decode(socket) {
                        Err(common::network::ErrorKind::Io(e))
                            if e.kind() == ErrorKind::WouldBlock =>
                        {
                            None
                        }
                        r => Some(r),
                    })
                    .collect::<Result<Box<_>, common::network::ErrorKind>>(),
                    *addr,
                )
            })
            .collect::<Box<_>>();

        for (packets, addr) in incoming {
            match packets {
                Err(common::network::ErrorKind::Io(e))
                    if [
                        ErrorKind::ConnectionAborted,
                        ErrorKind::ConnectionReset,
                        ErrorKind::ConnectionRefused,
                        ErrorKind::UnexpectedEof,
                    ]
                    .contains(&e.kind()) =>
                {
                    warn!("Client disconnected: {:?}", addr);
                    self.disconnect(&addr);
                }
                Err(e) => {
                    warn!("Failed to decode packets from client: {e:?}");
                }
                Ok(packets) => {
                    for packet in packets {
                        handler(self, addr, packet);
                    }
                }
            }
        }
    }

    pub fn handle_disconnections(
        &mut self,
        mut handler: impl FnMut(&mut Self, SocketAddr, NetRemoteClient),
    ) {
        //TODO: Maybe need to be fixed and also no need to run this every update
        for (addr, last_packet) in &self
            .clients
            .iter()
            .map(|(&addr, &(_, NetRemoteClient { last_packet, .. }))| (addr, last_packet))
            .collect::<Box<_>>()
        {
            if last_packet.elapsed() > self.timeout {
                info!("Client {addr:?} timed out");
                self.disconnect(addr);
            }
        }

        for (addr, client) in std::mem::take(&mut self.disconnected_clients) {
            handler(self, addr, client);
        }
    }

    pub fn is_connecting(&self, addr: &SocketAddr) -> bool {
        self.connecting_clients.contains_key(addr)
    }

    pub fn accept_connection(&mut self, addr: SocketAddr, profile: NetRemoteClient) {
        if let Some(socket) = self.connecting_clients.remove(&addr) {
            self.clients.insert(addr, (socket, profile));
        }
    }

    fn send_data_to(&mut self, addrs: impl IntoIterator<Item = SocketAddr>, data: Rc<[u8]>) {
        for addr in addrs {
            self.clients
                .get_mut(&addr)
                .unwrap()
                .0
                .write_all(&data)
                .unwrap_or_else(|e| {
                    warn!("Failed to write data to {addr:?} client: {e:?}");
                })
        }
    }

    pub fn send_to<'a, T: Serialize + 'static>(
        &mut self,
        addrs: impl IntoIterator<Item = SocketAddr>,
        packet: &T,
    ) {
        let data = self.protocol.encode(packet);
        self.send_data_to(addrs, data.into());
    }

    pub fn broadcast<T: Serialize + 'static>(&mut self, packet: &T) {
        let data = self.protocol.encode(packet);
        let addrs = self.clients.keys().cloned().collect::<Box<_>>();
        self.send_data_to(addrs, data.into());
    }

    pub fn broadcast_except<T: Serialize + 'static>(&mut self, except: &SocketAddr, packet: &T) {
        let data = self.protocol.encode(packet);
        let addrs = self
            .clients
            .keys()
            .cloned()
            .filter(|a| a != except)
            .collect::<Box<_>>();
        self.send_data_to(addrs, data.into());
    }

    pub fn get_remote(&self, addr: &SocketAddr) -> Option<&NetRemoteClient> {
        self.clients.get(addr).map(|(_, client)| client)
    }

    pub fn get_remote_mut(&mut self, addr: &SocketAddr) -> Option<&mut NetRemoteClient> {
        self.clients.get_mut(addr).map(|(_, client)| client)
    }

    pub fn disconnect(&mut self, addr: &SocketAddr) {
        self.connecting_clients.remove(addr);
        if let Some((_, client)) = self.clients.remove(addr) {
            self.disconnected_clients.insert(*addr, client);
        }
    }

    pub fn flush(&mut self) {
        for (socket, _) in self.clients.values_mut() {
            socket
                .flush()
                .unwrap_or_else(|e| warn!("Failed to flush stream to client: {e:?}"));
        }
    }
}
