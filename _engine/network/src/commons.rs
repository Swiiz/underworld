use dyn_clone::DynClone;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    any::{type_name, Any, TypeId},
    collections::HashMap,
    fmt::Display,
    io::{self, ErrorKind, Read, Write},
    marker::PhantomData,
    mem,
    net::{Shutdown, TcpStream},
};

pub trait NetworkSide: 'static {
    type Context;
    type OppositeSide;
}
pub struct ClientSide;
pub struct ServerSide;
impl NetworkSide for ClientSide {
    type Context = crate::client::NetworkClient;
    type OppositeSide = ServerSide;
}
impl NetworkSide for ServerSide {
    type Context = crate::server::NetworkServer;
    type OppositeSide = ClientSide;
}

type PacketId = u16;

pub trait Packet: Serialize + DeserializeOwned + DynClone + Any {
    type Side: NetworkSide;
}

pub struct NetworkProtocol {
    packets: HashMap<TypeId, PacketId>,
    packet_sizes: Vec<usize>,
}

impl NetworkProtocol {
    pub fn new() -> Self {
        Self {
            packet_sizes: Vec::new(),
            packets: HashMap::new(),
        }
    }

    pub fn with_packet<P: Packet>(mut self) -> Self {
        let tid = TypeId::of::<P>();
        assert!(
            !self.packets.contains_key(&tid),
            "Cannot register the same packet twice!"
        );
        let id = self
            .packets
            .len()
            .try_into()
            .expect("Could not create packet id from packet type id!");
        self.packets.insert(tid, id);
        self.packet_sizes.push(std::mem::size_of::<P>());
        self
    }

    fn id_of(&self, type_id: &TypeId) -> Option<PacketId> {
        self.packets.get(type_id).copied()
    }

    fn size_of(&self, id: PacketId) -> Option<usize> {
        self.packet_sizes.get(id as usize).copied()
    }
}

pub struct RawPacket {
    pub id: PacketId,
    pub bytes: Vec<u8>,
}

impl RawPacket {
    pub fn new<T: Packet>(packet: T, protocol: &NetworkProtocol) -> Self {
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
            bytes: bincode::serialize(&packet).unwrap_or_else(|e| {
                panic!("Could not serialize packet {}, {}", type_name::<T>(), e)
            }),
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
                    println!(
                        "WARNING: Fatal network error occured thus closing the connection: {}",
                        conn.remote_addr()
                    );
                    conn.close();
                    return ConnectionState::ShouldClose;
                }
                _ => (),
            },
            _ => (),
        }
        println!("WARNING: {self}");
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

pub trait Connection<S: NetworkSide>: Read + Write {
    fn configure(&mut self);
    fn remote_addr(&self) -> String;
    fn next_packet(&mut self, protocol: &NetworkProtocol) -> Result<RawPacket, PacketPollError> {
        let mut idbuf = [0; 2];

        self.read(&mut idbuf).map_err(PacketPollError::Io)?;
        let id = u16::from_be_bytes(idbuf);

        let size = protocol.size_of(id).ok_or(PacketPollError::InvalidPacket)?;

        if size == 0 {
            return Ok(RawPacket {
                id,
                bytes: Vec::new(),
            });
        }

        let mut bytes = vec![0u8; size];
        self.read(&mut bytes).map_or_else(
            |e| Err(PacketPollError::Io(e)),
            |r| {
                if size != r {
                    return Err(PacketPollError::InvalidPacket);
                }
                Ok(RawPacket { id, bytes })
            },
        )
    }
    fn emit(&mut self, packet: RawPacket, protocol: &NetworkProtocol) {
        self.write(&packet.id.to_be_bytes()).map_or_else(
            |e| println!("WARNING: Could not send packet id: {e}"),
            |_: usize| (),
        );
        self.write(&packet.bytes).map_or_else(
            |e| println!("WARNING: Could not send packet bytes: {e}"),
            |w| {
                if w != protocol
                    .size_of(packet.id)
                    .expect("Emitting invalid packet!")
                {
                    panic!("Emitting packet with wrong size!")
                }
            },
        );
        self.flush()
            .unwrap_or_else(|e| println!("WARNING: Could not flush network: {e}"));
    }
    fn close(&mut self);
}

pub(crate) fn process_conn<S: NetworkSide>(
    conn: &mut dyn Connection<S>,
    protocol: &NetworkProtocol,
) -> (Vec<RawPacket>, ConnectionState) {
    let mut packets = Vec::new();
    let mut state = ConnectionState::Valid;
    loop {
        let poll_result = conn.next_packet(protocol);
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

// TCP
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

pub(crate) struct ReceivedPackets {
    inner: Vec<Box<dyn PacketVec>>,
}

struct TypedPacketVec<T: Packet> {
    inner: Vec<T>,
}
trait PacketVec {
    fn push(&mut self, p: RawPacket);
}
impl<T: Packet> TypedPacketVec<T> {
    fn new() -> Self
    where
        Self: Sized,
    {
        Self { inner: Vec::new() }
    }
}
impl<T: Packet> PacketVec for TypedPacketVec<T> {
    fn push(&mut self, p: RawPacket) {
        self.inner.push(p.decode())
    }
}

impl ReceivedPackets {
    pub fn new() -> Self {
        Self { inner: Vec::new() }
    }

    pub fn push(&mut self, p: RawPacket) {
        let minlen = p.id as usize + 1;
        if self.inner.len() < minlen {
            self.inner
                .resize_with(minlen, || Box::new(TypedPacketVec::new()))
        }
        self.inner[p.id as usize].push(p);
    }

    pub fn clear(&mut self) {
        for v in &mut self.inner {
            v.clear();
        }
    }

    fn with_id(&self, id: PacketId) -> &Vec<RawPacket> {
        &self.inner[id as usize]
    }
}
