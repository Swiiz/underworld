use std::any::TypeId;

use crate::{
    connection::{
        process_conn, Connection, ConnectionProvider, ConnectionState, RawPacket, ReceivedPackets,
    },
    protocol::{NetworkProtocol, Packet},
    Client, ClientOnly, NetworkSide, Server, ServerOnly,
};

pub type DynConnection<S> = Box<dyn Connection<S>>;

pub struct Network<S: NetworkSide> {
    protocol: NetworkProtocol,
    received: ReceivedPackets,

    // Client only
    client_connection: ClientOnly<S, Option<DynConnection<S>>>,

    // Server only
    server_connection_providers: ServerOnly<S, Vec<Box<dyn ConnectionProvider>>>,
    server_connections: ServerOnly<S, Vec<DynConnection<S>>>,
}

impl<S: NetworkSide> Network<S> {
    pub fn new(protocol: NetworkProtocol) -> Self {
        Self {
            protocol,
            received: ReceivedPackets::new(),

            client_connection: S::client_only(None),

            server_connection_providers: S::server_only(Vec::new()),
            server_connections: S::server_only(Vec::new()),
        }
    }

    /// May lead to performance issue if not used carefully
    #[track_caller]
    pub fn on<P: Packet, F: FnMut(&mut Network<S>, P)>(&mut self, mut callback: F) {
        assert!(
            TypeId::of::<P::Side>() != TypeId::of::<S>(),
            "Cannot handle packets from own network side!"
        );
        let id = self.protocol.id_of(&TypeId::of::<P>()).unwrap();
        self.received
            .bytes_with_id(id)
            .iter()
            .cloned()
            .collect::<Vec<_>>()
            .into_iter()
            .for_each(|bytes| callback(self, RawPacket { id, bytes }.decode::<P>()))
    }
}
impl Network<Client> {
    pub fn set_connection(&mut self, mut conn: impl Connection<Client> + 'static) {
        conn.configure();
        self.client_connection = Some(Box::new(conn));
        println!(
            "INFO: client_connection established with remote: {}",
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
                self.received.push(p);
            }
            if state == ConnectionState::ShouldClose {
                self.client_connection = None;
            }
        }
    }

    pub fn emit<T: Packet<Side = Client>>(&mut self, packet: &T) {
        self.client_connection
            .as_mut()
            .map(|c| c.emit(RawPacket::new(packet, &self.protocol), &self.protocol));
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
                println!(
                    "INFO: Network server opened connection with: {}",
                    conn.remote_addr()
                );
                conn.configure();
                self.server_connections.push(conn);
            }
        }

        self.received.clear();
        let mut should_close = Vec::new();
        for (conn_id, conn) in self.server_connections.iter_mut().enumerate() {
            let (packets, state) = process_conn(conn.as_mut(), &self.protocol);
            for p in packets {
                self.received.push(p);
            }
            if state == ConnectionState::ShouldClose {
                should_close.push(conn_id);
            }
        }
        for id in should_close {
            self.server_connections.remove(id);
        }
    }

    pub fn broadcast<T: Packet<Side = Server>>(&mut self, packet: &T) {
        self.server_connections
            .iter_mut()
            .for_each(|c| c.emit(RawPacket::new(packet, &self.protocol), &self.protocol));
    }

    pub fn send<T: Packet<Side = Server>>(&mut self, packet: &T) {
        todo!()
    }
}
