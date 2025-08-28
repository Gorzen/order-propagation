use bincode::{Decode, Encode, config::Configuration};
use borsh::{BorshDeserialize, BorshSerialize};

use crate::{network::NodeId, order::Order};

#[derive(
    Debug, Clone, Copy, Hash, PartialEq, Eq, Decode, Encode, BorshDeserialize, BorshSerialize,
)]
/// Unique identifier for the packet, could be a UUID or a sequence number
pub struct PacketId(u64);

impl PacketId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }
}

#[derive(Debug, Clone)]
pub struct SerialiedPacket(Vec<u8>);

impl SerialiedPacket {
    pub fn bincode_deserialize(&self, config: Configuration) -> GossipPacket {
        bincode::decode_from_slice(&self.0, config).unwrap().0
    }

    pub fn borsh_deserialize(&self) -> GossipPacket {
        borsh::from_slice(&self.0).unwrap()
    }
}

#[derive(Debug, Clone, Encode, Decode, PartialEq, BorshDeserialize, BorshSerialize)]
pub struct GossipPacket {
    pub id: PacketId,
    pub source_id: NodeId, // ID of the node that sent the packet
    pub ttl: u64,
    pub order: Order,
}

impl GossipPacket {
    pub fn new(id: PacketId, source_id: NodeId, ttl: u64, order: Order) -> Self {
        Self {
            id,
            source_id,
            ttl,
            order,
        }
    }

    pub fn new_with_random_order(id: PacketId, source_id: NodeId, ttl: u64) -> Self {
        GossipPacket::new(id, source_id, ttl, Order::random_order())
    }

    pub fn bincode_serialize(&self, config: Configuration) -> SerialiedPacket {
        SerialiedPacket(bincode::encode_to_vec(self, config).unwrap())
    }

    pub fn borsh_serialize(&self) -> SerialiedPacket {
        SerialiedPacket(borsh::to_vec(self).unwrap())
    }
}

#[cfg(test)]
mod tests {
    use bincode::config;

    use super::*;

    /// Num runs for test
    const NUM_RUNS: u64 = 100;

    #[test]
    /// Property test for bincode codec
    fn test_bincode_codec() {
        let config = config::standard();

        for i in 0..NUM_RUNS {
            let packet = GossipPacket::new_with_random_order(PacketId::new(i), NodeId::new(i), i);
            let serialized = packet.bincode_serialize(config);
            let deserialized = serialized.bincode_deserialize(config);
            assert_eq!(packet, deserialized);
        }
    }

    #[test]
    /// Property test for borsh codec
    fn test_borsch_codec() {
        for i in 0..NUM_RUNS {
            let packet = GossipPacket::new_with_random_order(PacketId::new(i), NodeId::new(i), i);
            let serialized = packet.borsh_serialize();
            let deserialized = serialized.borsh_deserialize();
            assert_eq!(packet, deserialized);
        }
    }
}
