use bimap::BiMap;
use serde::{Deserialize, Serialize};
use std::{any::TypeId, u16};

use serde::de::DeserializeOwned;

pub mod proto;

#[derive(Serialize, Deserialize)]
pub struct RawPacket {
    pub id: u16,
    pub data: Box<[u8]>,
}

pub struct AnyPacket {
    type_id: TypeId,
    data: Box<[u8]>,
}

impl AnyPacket {
    pub fn is<T: 'static>(&self) -> bool {
        TypeId::of::<T>() == self.type_id
    }

    pub fn try_decode<T: DeserializeOwned + 'static>(&self) -> Option<T> {
        self.is::<T>()
            .then(|| bincode::deserialize(&self.data).expect("Failed to deserialize data"))
    }

    pub fn type_id(&self) -> TypeId {
        self.type_id
    }
}

type PacketId = u16;

pub struct Protocol {
    packets: BiMap<TypeId, PacketId>,
}

impl Protocol {
    fn new() -> Self {
        Self {
            packets: BiMap::new(),
        }
    }

    fn add_packet<T: 'static>(&mut self) -> &mut Self {
        let tid = TypeId::of::<T>();
        if self.packets.contains_left(&tid) {
            panic!("Packet already exists");
        }

        self.packets.insert(tid, self.packets.len() as u16);
        self
    }

    pub fn id_of<T: 'static>(&self) -> Option<u16> {
        self.packets.get_by_left(&TypeId::of::<T>()).copied()
    }

    pub fn type_id_of(&self, id: PacketId) -> TypeId {
        self.packets
            .get_by_right(&id)
            .unwrap_or_else(|| panic!("Invalid packet id: {}", id))
            .clone()
    }

    pub fn encode<T: Serialize + 'static>(&self, data: &T) -> Box<[u8]> {
        let data_bin = bincode::serialize(data).expect("Failed to serialize data");
        bincode::serialize(&RawPacket {
            id: self.id_of::<T>().unwrap_or_else(|| {
                panic!(
                    "Tried to encode unknown packet: {}, include it in the protocol",
                    std::any::type_name::<T>()
                )
            }),
            data: data_bin.into(),
        })
        .expect("Failed to encode packet")
        .into()
    }

    pub fn decode(&self, bytes: &[u8]) -> Option<AnyPacket> {
        let raw: RawPacket = bincode::deserialize(bytes).ok()?;
        Some(AnyPacket {
            type_id: self.type_id_of(raw.id),
            data: raw.data,
        })
    }
}
