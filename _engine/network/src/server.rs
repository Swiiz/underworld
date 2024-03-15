use crate::commons::{
    process_conn, Connection, ConnectionState, HandlePacket, NetworkProtocol, RawPacket,
    ReceivedPackets, ServerSide,
};
use std::{io::ErrorKind, net::TcpListener};

pub struct NetworkServer {
    providers: Vec<Box<dyn ConnectionProvider>>,
    connections: Vec<Box<dyn Connection<ServerSide>>>,
    protocol: NetworkProtocol,
    received: ReceivedPackets,
}

impl NetworkServer {
    pub fn new(protocol: NetworkProtocol) -> Self {
        Self {
            providers: Vec::new(),
            connections: Vec::new(),
            protocol,
            received: ReceivedPackets::new(),
        }
    }

    pub fn add_provider<T: ConnectionProvider + 'static>(&mut self, mut provider: T) {
        provider.configure();
        self.providers.push(Box::new(provider))
    }

    pub fn poll(&mut self) {
        for provider in &self.providers {
            while let Some(mut conn) = provider.poll_conn() {
                println!(
                    "INFO: Network server opened connection with: {}",
                    conn.remote_addr()
                );
                conn.configure();
                self.connections.push(conn);
            }
        }

        self.received.clear();
        let mut should_close = Vec::new();
        for (conn_id, conn) in self.connections.iter_mut().enumerate() {
            let (packets, state) = process_conn(conn.as_mut(), &self.protocol);
            for p in packets {
                self.received.push(p);
            }
            if state == ConnectionState::ShouldClose {
                should_close.push(conn_id);
            }
        }
        for id in should_close {
            self.connections.remove(id);
        }
    }
}

pub trait ConnectionProvider {
    fn configure(&mut self);
    fn poll_conn(&self) -> Option<Box<dyn Connection<ServerSide>>>;
}

// TCP

impl ConnectionProvider for TcpListener {
    fn configure(&mut self) {
        self.set_nonblocking(true).expect("Cannot set non-blocking");
    }

    fn poll_conn(&self) -> Option<Box<dyn Connection<ServerSide>>> {
        match self.accept() {
            Ok((conn, _)) => Some(Box::new(conn)),
            Err(e) => {
                if e.kind() != ErrorKind::WouldBlock {
                    println!(
                        "WARNING: Error occured while polling connections from tcp server: {e}"
                    )
                }
                None
            }
        }
    }
}

impl HandlePacket for NetworkServer {
    #[allow(private_interfaces)]
    fn received(&self) -> &ReceivedPackets {
        &self.received
    }
    fn protocol(&self) -> &NetworkProtocol {
        &self.protocol
    }
}
