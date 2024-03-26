use std::{
    any::{type_name, TypeId},
    fmt::Display,
    io::{self, ErrorKind, Read, Write},
    marker::PhantomData,
    rc::Rc,
};

use platform::warn;

use crate::{
    ctx::{ConnectionHandle, DynConnection, Network},
    protocol::{NetworkProtocol, Packet, PacketId},
    Client, NetworkSide, Server, ServerOnly,
};

pub struct PacketQueue<S: NetworkSide> {
    packets: Vec<(Vec<RawPacket>, ServerOnly<S, Vec<ConnectionHandle<S>>>)>,
    protocol: NetworkProtocol,
    _marker: PhantomData<S>,
}

impl<S: NetworkSide> PacketQueue<S> {
    pub fn new(protocol: NetworkProtocol) -> Self {
        Self {
            packets: Vec::new(),
            protocol,
            _marker: PhantomData,
        }
    }

    pub fn push<T: Packet<Side = S>>(&mut self, packets: &[T], conns: Vec<ConnectionHandle<S>>) {
        self.packets.push((
            packets
                .into_iter()
                .map(|p| RawPacket::new(p, &self.protocol))
                .collect(),
            S::server_only(conns),
        ));
    }
}

impl PacketQueue<Client> {
    pub fn submit(&mut self, network: &mut Network<Client>) {
        let packets = std::mem::replace(&mut self.packets, Vec::new());
        for (packets, _) in packets {
            network.send_raw(packets);
        }
    }
}

impl PacketQueue<Server> {
    pub fn submit(&mut self, network: &mut Network<Server>) {
        let packets = std::mem::replace(&mut self.packets, Vec::new());
        for (packets, conns) in packets {
            network.send_raw(packets, &conns);
        }
    }
}

#[derive(Clone)]
pub struct RawPacket {
    pub id: PacketId,
    pub size: u16,
    pub bytes: Rc<Vec<u8>>,
}

impl RawPacket {
    pub fn new<T: Packet>(packet: &T, protocol: &NetworkProtocol) -> Self {
        let bytes = bincode::serialize(packet)
            .unwrap_or_else(|e| panic!("Could not serialize packet {}, {}", type_name::<T>(), e));

        Self {
            id: protocol
                .id_of(&TypeId::of::<T>())
                .or_else(|| {
                    panic!(
                        "Unknown packet {}, you need to define it in your NetworkProtocol!",
                        type_name::<T>()
                    )
                })
                .unwrap(),
            size: bytes.len() as u16,
            bytes: Rc::new(bytes),
        }
    }

    pub fn decode<'a, T: Packet>(&'a self) -> T {
        bincode::deserialize(&self.bytes)
            .unwrap_or_else(|e| panic!("Could not deserialize packet {}, {}", type_name::<T>(), e))
    }
}

#[derive(Debug)]
pub enum PacketPollError {
    InvalidPacket,
    Io(io::Error),
}
#[derive(PartialEq, Eq)]
pub enum ConnectionState {
    Valid,
    ShouldClose,
}
impl ConnectionState {
    pub fn aggregate(&mut self, other: Self) {
        if other == ConnectionState::ShouldClose {
            *self = other;
        }
    }
}
impl PacketPollError {
    pub fn process<S: NetworkSide>(self, conn: &mut dyn Connection<S>) -> ConnectionState {
        match &self {
            Self::Io(e) => match e.kind() {
                ErrorKind::WouldBlock => {
                    return ConnectionState::Valid;
                }
                ErrorKind::ConnectionAborted
                | ErrorKind::ConnectionRefused
                | ErrorKind::ConnectionReset
                | ErrorKind::Interrupted
                | ErrorKind::Unsupported => {
                    warn!(
                        "Fatal network error occured thus closing the connection: {}",
                        conn.remote_addr()
                    );
                    conn.close();
                    return ConnectionState::ShouldClose;
                }
                _ => (),
            },
            _ => (),
        }
        warn!("{self}");
        ConnectionState::Valid
    }
}
impl Display for PacketPollError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "An error occured polling packet: {}",
            match self {
                Self::InvalidPacket => "Invalid packet!".to_owned(),
                Self::Io(e) => format!("Io error: {}", e),
            }
        )
    }
}
impl std::error::Error for PacketPollError {}

pub trait ConnectionProvider {
    fn configure(&mut self);
    fn poll_conn(&self) -> Option<DynConnection<Server>>;
}

pub trait Connection<S: NetworkSide>: Read + Write {
    fn configure(&mut self);
    fn remote_addr(&self) -> String;
    fn next_packet(&mut self) -> Result<RawPacket, PacketPollError> {
        let mut idbuf = [0; 2];
        let mut sizebuf = [0; 2];

        self.read(&mut idbuf).map_err(PacketPollError::Io)?;
        let id = u16::from_be_bytes(idbuf);
        self.read(&mut sizebuf).map_err(PacketPollError::Io)?;
        let size = u16::from_be_bytes(sizebuf);

        if size == 0 {
            return Ok(RawPacket {
                id,
                size,
                bytes: Rc::new(Vec::new()),
            });
        }

        let mut bytes = vec![0u8; size as usize];
        self.read(&mut bytes).map_or_else(
            |e| Err(PacketPollError::Io(e)),
            |r| {
                if size != r as u16 {
                    return Err(PacketPollError::InvalidPacket);
                }
                Ok(RawPacket {
                    id,
                    size,
                    bytes: Rc::new(bytes),
                })
            },
        )
    }
    fn emit(&mut self, packet: RawPacket) {
        self.write(&packet.id.to_be_bytes())
            .map_or_else(|e| warn!("Could not send packet id: {e}"), |_: usize| ());
        self.write(&packet.size.to_be_bytes())
            .map_or_else(|e| warn!("Could not send packet size: {e}"), |_: usize| ());
        self.write(&packet.bytes).map_or_else(
            |e| warn!("Could not send packet bytes: {e}"),
            |w| {
                if w != packet.size as usize {
                    panic!("Emitting packet with wrong size!")
                }
            },
        );
        self.flush()
            .unwrap_or_else(|e| warn!("Could not flush network: {e}"));
    }
    fn close(&mut self);
}

pub(crate) fn process_conn<S: NetworkSide>(
    conn: &mut dyn Connection<S>,
    _protocol: &NetworkProtocol,
) -> (Vec<RawPacket>, ConnectionState) {
    let mut packets = Vec::new();
    let mut state = ConnectionState::Valid;
    loop {
        let poll_result = conn.next_packet();
        match poll_result {
            Err(e) => {
                state.aggregate(e.process(conn));
                break;
            }
            Ok(p) => packets.push(p),
        };
    }
    (packets, state)
}

mod tcp {
    use std::{
        io::ErrorKind,
        net::{Shutdown, TcpListener, TcpStream},
    };

    use platform::warn;

    use crate::{ctx::DynConnection, NetworkSide, Server};

    use super::{Connection, ConnectionProvider};

    impl<S: NetworkSide> Connection<S> for TcpStream {
        fn configure(&mut self) {
            self.set_nonblocking(true)
                .expect("Could not configure tcp stream to be non blocking!");
        }

        fn remote_addr(&self) -> String {
            self.peer_addr()
                .expect("Could not resolve tcp connection remote address!")
                .to_string()
        }

        fn close(&mut self) {
            let _ = self.shutdown(Shutdown::Both);
        }
    }

    impl ConnectionProvider for TcpListener {
        fn configure(&mut self) {
            self.set_nonblocking(true).expect("Cannot set non-blocking");
        }

        fn poll_conn(&self) -> Option<DynConnection<Server>> {
            match self.accept() {
                Ok((conn, _)) => Some(Box::new(conn)),
                Err(e) => {
                    if e.kind() != ErrorKind::WouldBlock {
                        warn!("Error occured while polling connections from tcp server: {e}")
                    }
                    None
                }
            }
        }
    }
}

pub(crate) struct ReceivedPackets<S: NetworkSide> {
    inner: Vec<Vec<(ConnectionHandle<S>, Rc<Vec<u8>>)>>,
}

impl<S: NetworkSide> ReceivedPackets<S> {
    pub fn new() -> Self {
        Self { inner: Vec::new() }
    }

    pub fn push(&mut self, conn_handle: ConnectionHandle<S>, p: RawPacket) {
        let minlen = p.id as usize + 1;
        if self.inner.len() < minlen {
            self.inner.resize_with(minlen, || Vec::new())
        }
        self.inner[p.id as usize].push((conn_handle, p.bytes));
    }

    pub fn clear(&mut self) {
        for v in &mut self.inner {
            v.clear();
        }
    }

    pub(crate) fn bytes_with_id(
        &self,
        id: PacketId,
    ) -> Option<&Vec<(ConnectionHandle<S>, Rc<Vec<u8>>)>> {
        if self.inner.len() <= id as usize {
            return None;
        }
        Some(&self.inner[id as usize])
    }
}
