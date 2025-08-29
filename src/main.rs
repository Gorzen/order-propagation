use std::{process::exit, time::Duration};

use order_propagation::{
    config::Config,
    network::Network,
    packet::{GossipPacket, PacketId, SerialiedPacket},
    plot,
};
use tokio::sync::mpsc;

#[tokio::main]
async fn main() {
    let config = Config::load().unwrap();
    let network = Network::generate_network(config.num_nodes, config.num_neighbors);
    let num_runs = config.num_runs as usize;
    let threshold = (config.num_nodes as f64 * 0.95).ceil() as usize;
    let mut packet_latencies = Vec::<Duration>::with_capacity(num_runs * threshold);
    let mut elapsed_times = Vec::<Duration>::with_capacity(num_runs);

    let (report_tx, mut report_rx) = mpsc::channel::<PacketId>(config.num_nodes as usize);

    let (start_node_id, start_sender) = network
        .run_network(config.latency(), config.num_peers, &report_tx)
        .expect("Empty network");

    for i in 0..num_runs {
        let packet = GossipPacket::new_with_random_order(
            PacketId::new(i as u64),
            start_node_id.clone(),
            config.time_to_live,
        );

        let (elapsed, latencies, returned_rx) =
            propagate_message(packet, threshold, report_rx, start_sender.clone()).await;

        report_rx = returned_rx;
        elapsed_times.push(elapsed);
        packet_latencies.extend(latencies);
    }

    let (mean, std_dev) = calculate_stats(&elapsed_times);
    println!("Number of runs: {num_runs}\n95% Propagation Time (mean ± σ): {mean:?} ± {std_dev:?}");

    plot::plot_gossip_data(packet_latencies).expect("Failed to plot gossip data");
}

async fn propagate_message(
    packet: GossipPacket,
    threshold: usize,
    mut report_rx: mpsc::Receiver<PacketId>,
    node_sender: mpsc::Sender<SerialiedPacket>,
) -> (Duration, Vec<Duration>, mpsc::Receiver<PacketId>) {
    let now = tokio::time::Instant::now();

    if let Err(e) = node_sender.send(packet.borsh_serialize()).await {
        eprintln!("Failed to send initial message: {e}. Exiting...");
        exit(1)
    }

    println!("Waiting for message to reach {threshold} nodes...");

    let mut received_count = 0;
    let mut latencies = Vec::with_capacity(threshold);

    loop {
        match report_rx.recv().await {
            Some(packet_id) => {
                // Ignore messages that are not for that packet_id (can happen if num_runs is > 1)
                if packet_id == packet.id {
                    received_count += 1;
                    latencies.push(now.elapsed());
                    if received_count >= threshold {
                        println!("Propagation threshold reached!");
                        break;
                    }
                }
            }
            None => {
                eprintln!("Report channel closed. Exiting...");
                exit(1)
            }
        }
    }
    let elapsed = now.elapsed();

    (elapsed, latencies, report_rx)
}

/// Calculates the mean and standard deviation of durations.
fn calculate_stats(data: &[Duration]) -> (Duration, Duration) {
    let count = data.len() as u64;
    if count == 0 {
        return (Duration::ZERO, Duration::ZERO);
    }

    let sum: Duration = data.iter().sum();
    let mean = sum / count as u32;

    let variance_sum: f64 = data
        .iter()
        .map(|&d| (d.as_secs_f64() - mean.as_secs_f64()).powi(2))
        .sum();
    let variance = variance_sum / count as f64;

    let std_dev = Duration::from_secs_f64(variance.sqrt());

    (mean, std_dev)
}
