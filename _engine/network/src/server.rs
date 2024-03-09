use std::{
    io::{ErrorKind, Read, Write},
    marker::PhantomData,
    net::{TcpListener, TcpStream},
};

use crate::commons::{Connection, NetworkProtocol};

pub struct NetworkServer<P: NetworkProtocol> {
    providers: Vec<Box<dyn ConnectionProvider>>,
    connections: Vec<Box<dyn Connection>>,
    _protocol: P,
}

impl<P: NetworkProtocol> NetworkServer<P> {
    pub fn new(_protocol: P) -> Self {
        Self {
            providers: Vec::new(),
            connections: Vec::new(),
            _protocol,
        }
    }

    pub fn add_provider<T: ConnectionProvider + 'static>(&mut self, provider: T) {
        self.providers.push(Box::new(provider))
    }

    pub fn update(&mut self) {
        for provider in &self.providers {
            while let Some(conn) = provider.poll_conn() {
                println!(
                    "INFO: Network server opened connection with: {}",
                    conn.addr()
                );
                self.connections.push(conn);
            }
        }
    }
}

pub trait ConnectionProvider {
    fn configure(&mut self);
    fn poll_conn(&self) -> Option<Box<dyn Connection>>;
}

// TCP

impl ConnectionProvider for TcpListener {
    fn configure(&mut self) {
        self.set_nonblocking(true).expect("Cannot set non-blocking");
    }

    fn poll_conn(&self) -> Option<Box<dyn Connection>> {
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
