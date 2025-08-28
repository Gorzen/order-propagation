use std::{
    collections::{HashMap, HashSet},
    time::Duration,
};

use bincode::{Decode, Encode};
use borsh::{BorshDeserialize, BorshSerialize};
use rand::seq::IndexedRandom;
use tokio::sync::mpsc;

use crate::{
    node,
    packet::{PacketId, SerialiedPacket},
};

#[derive(Debug, Eq, Hash, PartialEq, Clone, Encode, Decode, BorshDeserialize, BorshSerialize)]
pub struct NodeId(u64);

impl NodeId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }
}

#[derive(Debug)]
pub struct Network {
    /// Map from NodeId to neighbors
    neighbors: HashMap<NodeId, HashSet<NodeId>>,
}

impl Network {
    fn new(neighbors: HashMap<NodeId, HashSet<NodeId>>) -> Self {
        Self { neighbors }
    }

    fn nodes(&self) -> HashSet<NodeId> {
        self.neighbors.keys().cloned().collect()
    }

    fn neighbors(&self, node_id: NodeId) -> HashSet<NodeId> {
        self.neighbors
            .get(&node_id)
            .cloned()
            .unwrap_or_else(HashSet::new)
    }

    pub fn generate_network(num_nodes: u64, num_neighbors: u64) -> Self {
        assert!(
            num_neighbors < num_nodes,
            "More neighbors wanted than nodes available"
        );

        let mut network: HashMap<NodeId, HashSet<NodeId>> = HashMap::new();
        let mut rng = rand::rng();

        let node_ids: Vec<NodeId> = (0..num_nodes).map(NodeId).collect();

        for node_id in &node_ids {
            let possible_neighbors: Vec<NodeId> =
                node_ids.iter().filter(|n| *n != node_id).cloned().collect();
            let neighbors: HashSet<NodeId> = possible_neighbors
                .choose_multiple(&mut rng, num_neighbors as usize)
                .cloned()
                .collect();
            network.insert(node_id.clone(), neighbors);
        }

        Network::new(network)
    }

    /// Starts each node task and returns one node_id and its sender to propagate messages to the network
    pub fn run_network(
        &self,
        latency: Duration,
        num_peers: u64,
        report_tx: &mpsc::Sender<PacketId>,
    ) -> Option<(NodeId, mpsc::Sender<SerialiedPacket>)> {
        let mut senders = HashMap::new();
        let mut receivers = HashMap::new();

        for id in self.nodes() {
            // Create a channel for each node. 32 is the buffer size.
            let (tx, rx) = mpsc::channel::<SerialiedPacket>(32); // queue of 32
            senders.insert(id.clone(), tx);
            receivers.insert(id.clone(), rx);
        }

        let (start_node_id, start_sender) = senders.iter().next()?;

        // Spawn a task for each node.
        for node_id in self.nodes() {
            let receiver = receivers.remove(&node_id).unwrap();
            let all_senders_clone = senders.clone();

            tokio::spawn(node::node_task(
                node_id.clone(),
                latency,
                self.neighbors(node_id),
                num_peers,
                receiver,
                all_senders_clone,
                report_tx.clone(),
            ));
        }

        Some((start_node_id.clone(), start_sender.clone()))
    }
}

#[cfg(test)]
mod tests {
    use crate::packet::GossipPacket;

    use super::*;

    #[tokio::test]
    /// Start a simple network with 3 nodes and check a message can be propagated to the network
    async fn test_network() {
        let network = Network::new(HashMap::from([
            (NodeId(0), HashSet::from([NodeId(1)])),
            (NodeId(1), HashSet::from([NodeId(2)])),
            (NodeId(2), HashSet::from([NodeId(0)])),
        ]));

        let (report_tx, mut report_rx) = mpsc::channel::<PacketId>(3);

        // Run network and send start packet
        let (start_id, start_sender) = network.run_network(Duration::ZERO, 1, &report_tx).unwrap();
        let packet = GossipPacket::new_with_random_order(PacketId::new(1), start_id, 3);
        let _ = start_sender.send(packet.borsh_serialize()).await;

        // Wait for packet to be propagated
        let mut received_count = 0;
        loop {
            if let Some(_) = report_rx.recv().await {
                received_count += 1;
                if received_count == 3 {
                    break;
                }
            }
        }
    }
}
