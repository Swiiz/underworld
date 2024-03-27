use std::{any::TypeId, rc::Rc};

use genmap::GenMap;
use platform::info;

use crate::{
    connection::{
        process_conn, Connection, ConnectionProvider, ConnectionState, RawPacket, ReceivedPackets,
    },
    protocol::{NetworkProtocol, Packet},
    Client, ClientOnly, NetworkSide, Server, ServerOnly,
};

#[derive(Debug, Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct ConnectionHandle<S: NetworkSide>(S::ConnectionHandle);
pub type DynConnection<S> = Box<dyn Connection<S>>;

pub struct Network<S: NetworkSide> {
    protocol: Rc<NetworkProtocol>,
    received: ReceivedPackets<S>,

    // Client only
    client_connection: ClientOnly<S, Option<DynConnection<Client>>>,

    // Server only
    server_connection_providers: ServerOnly<S, Vec<Box<dyn ConnectionProvider>>>,
    server_connections: ServerOnly<S, GenMap<DynConnection<Server>>>,
}

impl<S: NetworkSide> Network<S> {
    pub fn new(protocol: NetworkProtocol) -> Self {
        Self {
            protocol: Rc::new(protocol),
            received: ReceivedPackets::new(),

            client_connection: S::client_only(None),

            server_connection_providers: S::server_only(Vec::new()),
            server_connections: S::server_only(GenMap::<DynConnection<Server>>::with_capacity(50)),
        }
    }

    /// May lead to performance issue if not used carefully
    pub fn on<P: Packet>(
        &mut self,
        mut callback: impl FnMut(&mut Network<S>, P, ConnectionHandle<S>),
    ) {
        assert!(
            TypeId::of::<P::Side>() != TypeId::of::<S>(),
            "Cannot handle packets from own network side!"
        );
        let id = self.protocol.id_of(&TypeId::of::<P>()).expect("Tried to handle packet that isn't in the protocol! Don't forget to add it to your NetworkProtocol!");
        let Some(r) = self.received.bytes_with_id(id) else {
            return;
        };
        r.iter()
            .cloned()
            .collect::<Vec<_>>()
            .into_iter()
            .for_each(|(conn_handle, bytes)| {
                callback(
                    self,
                    RawPacket {
                        id,
                        size: bytes.len() as u16,
                        bytes,
                    }
                    .decode::<P>(),
                    conn_handle,
                )
            })
    }
}
impl Network<Client> {
    pub fn set_connection(&mut self, mut conn: impl Connection<Client> + 'static) {
        conn.configure();
        self.client_connection = Some(Box::new(conn));
        info!(
            "client_connection established with remote: {}",
            self.client_connection.as_ref().unwrap().remote_addr()
        );
    }

    pub fn is_connected(&self) -> bool {
        self.client_connection.is_some()
    }

    pub fn poll(&mut self) {
        self.received.clear();
        if let Some(conn) = &mut self.client_connection {
            let (packets, state) = process_conn(conn.as_mut(), &self.protocol);
            for p in packets {
                self.received.push(ConnectionHandle(()), p);
            }
            if state == ConnectionState::ShouldClose {
                self.client_connection = None;
            }
        }
    }

    pub fn send<T: Packet<Side = Client>>(&mut self, packets: &[T]) {
        let packets = packets
            .iter()
            .map(|p| RawPacket::new(p, &self.protocol))
            .collect();
        self.send_raw(packets);
    }

    pub(crate) fn send_raw(&mut self, packets: Vec<RawPacket>) {
        self.client_connection.as_mut().map(|c| {
            for p in packets {
                c.emit(p)
            }
        });
    }
}

impl Network<Server> {
    pub fn add_provider<T: ConnectionProvider + 'static>(&mut self, mut provider: T) {
        provider.configure();
        self.server_connection_providers.push(Box::new(provider))
    }

    pub fn poll(&mut self) {
        for provider in &self.server_connection_providers {
            while let Some(mut conn) = provider.poll_conn() {
                info!(
                    "Network server opened connection with: {}",
                    conn.remote_addr()
                );
                conn.configure();
                self.server_connections.insert(conn);
            }
        }

        self.received.clear();
        let mut should_close = Vec::new();
        let handles = self.server_connections.iter().collect::<Vec<_>>();
        for conn_handle in handles {
            let (packets, state) = process_conn(
                self.server_connections
                    .get_mut(conn_handle)
                    .unwrap()
                    .as_mut(),
                &self.protocol,
            );
            for p in packets {
                self.received.push(ConnectionHandle(conn_handle), p);
            }
            if state == ConnectionState::ShouldClose {
                should_close.push(conn_handle);
            }
        }
        for id in should_close {
            self.server_connections.remove(id);
        }
    }

    pub fn all_connections(&self) -> Vec<ConnectionHandle<Server>> {
        self.server_connections
            .iter()
            .map(ConnectionHandle::<Server>)
            .collect::<Vec<_>>()
    }

    pub fn send<T: Packet<Side = Server>>(
        &mut self,
        packets: &[T],
        conn_handles: &[ConnectionHandle<Server>],
    ) {
        let packets = packets
            .iter()
            .map(|p| RawPacket::new(p, &self.protocol))
            .collect();
        self.send_raw(packets, conn_handles);
    }

    pub(crate) fn send_raw(
        &mut self,
        packets: Vec<RawPacket>,
        conn_handles: &[ConnectionHandle<Server>],
    ) {
        conn_handles.iter().for_each(|ch| {
            for p in packets.clone() {
                self.server_connections
                    .get_mut(ch.0)
                    .unwrap()
                    .emit(p.clone())
            }
        })
    }
}
