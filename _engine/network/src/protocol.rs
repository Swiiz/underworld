use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

use serde::{de::DeserializeOwned, Serialize};

use crate::NetworkSide;

pub(crate) type PacketId = u16;

pub trait Packet: Serialize + DeserializeOwned + Any {
    type Side: NetworkSide;
}

pub struct NetworkProtocol {
    packets: HashMap<TypeId, PacketId>,
}

impl NetworkProtocol {
    pub fn new() -> Self {
        Self {
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
        self
    }

    pub(crate) fn id_of(&self, type_id: &TypeId) -> Option<PacketId> {
        self.packets.get(type_id).copied()
    }
}
