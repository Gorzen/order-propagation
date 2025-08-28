use std::{
    collections::{HashMap, HashSet},
    process::exit,
    time::Duration,
};

use tokio::sync::mpsc;

use crate::{
    network::NodeId,
    order::Order,
    packet::{GossipPacket, PacketId, SerialiedPacket},
};

use rand::seq::IndexedRandom;

/// Node's async task. It listens for incoming messages and gossips them to its neighbors.
pub async fn node_task(
    node_id: NodeId,
    latency: Duration,
    neighbors: HashSet<NodeId>,
    num_peers: u64,
    mut receiver: mpsc::Receiver<SerialiedPacket>,
    all_senders: HashMap<NodeId, mpsc::Sender<SerialiedPacket>>,
    report_sender: mpsc::Sender<PacketId>,
) {
    // A set to keep track of messages this node has already seen and gossiped.
    // This is crucial to prevent infinite message loops in the network (e.g., A->B->A).
    let mut seen_messages = HashSet::new();

    let mut orders: Vec<Order> = Vec::new();

    // Loop indefinitely, waiting for messages on the receiver channel.
    while let Some(serialized_packet) = receiver.recv().await {
        let packet = serialized_packet.borsh_deserialize();
        let is_new_message = seen_messages.insert(packet.id);

        // If we've already processed this message, ignore it.
        if !is_new_message {
            continue;
        }

        // -- Process the order --
        // We simply record it but the logic could be more complex (match against orders, send match result, allow different order types, allow cancellation, ...)
        orders.push(packet.order.clone());

        // Report back to main
        let _ = report_sender.send(packet.id).await;

        // Don't propagate order if TTL is reached
        if packet.ttl == 0 {
            continue;
        }

        let mut considered_neighbors: HashSet<NodeId> = neighbors.clone();
        considered_neighbors.remove(&packet.source_id);

        let packet_to_send = GossipPacket::new(
            packet.id,
            node_id.clone(),
            packet.ttl.saturating_sub(1),
            packet.order,
        );

        let neighbor_count = considered_neighbors.len().min(num_peers as usize);

        // Iterate over the node's neighbors.
        for neighbor_id in random_neighbors(&considered_neighbors, neighbor_count) {
            // In a real application, would handle the case where the sender is missing
            let sender = all_senders.get(neighbor_id).unwrap();

            // Spawn a new task to send packet to simulate network delay in the send without blocking the node's task.
            tokio::spawn(send_gossip_packet_with_delay(
                sender.clone(),
                node_id.clone(),
                neighbor_id.clone(),
                packet_to_send.borsh_serialize(),
                latency,
            ));
        }
    }
    println!("[{node_id:?}]: Channel closed. Task shutting down.");
}

fn random_neighbors(nodes: &HashSet<NodeId>, count: usize) -> Vec<&NodeId> {
    let mut rng = rand::rng();
    nodes
        .iter()
        .collect::<Vec<&NodeId>>()
        .choose_multiple(&mut rng, count)
        .cloned()
        .collect()
}

async fn send_gossip_packet_with_delay(
    sender: mpsc::Sender<SerialiedPacket>,
    node_id: NodeId,
    neighbor_id: NodeId,
    packet: SerialiedPacket,
    latency: Duration,
) -> () {
    tokio::time::sleep(latency).await;

    // In a real application, you'd want to better handle potential errors sending the packet.
    if let Err(e) = sender.send(packet).await {
        eprintln!("[{node_id:?}]: Failed to send to {neighbor_id:?}: {e}");
        exit(1)
    }
}
