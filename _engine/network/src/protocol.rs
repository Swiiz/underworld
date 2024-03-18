use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

use dyn_clone::DynClone;
use serde::{de::DeserializeOwned, Serialize};

use crate::NetworkSide;

pub(crate) type PacketId = u16;

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

    pub(crate) fn id_of(&self, type_id: &TypeId) -> Option<PacketId> {
        self.packets.get(type_id).copied()
    }

    pub(crate) fn size_of(&self, id: PacketId) -> Option<usize> {
        self.packet_sizes.get(id as usize).copied()
    }
}
