use bincode::config::Configuration;
use criterion::{Criterion, criterion_group, criterion_main};
use order_propagation::{
    network::NodeId,
    packet::{GossipPacket, PacketId},
};
use std::hint::black_box;

fn bincode_codec(packet: GossipPacket, config: Configuration) -> GossipPacket {
    let serialized = packet.bincode_serialize(config);
    serialized.bincode_deserialize(config)
}

fn borsh_codec(packet: GossipPacket) -> GossipPacket {
    let serialized = packet.borsh_serialize();
    serialized.borsh_deserialize()
}

fn criterion_benchmark(c: &mut Criterion) {
    let packet = GossipPacket::new_with_random_order(PacketId::new(1), NodeId::new(1), 1);
    let bincode_config: Configuration = bincode::config::standard();

    c.bench_function("bincode_codec", |b| {
        b.iter(|| bincode_codec(black_box(packet.clone()), bincode_config))
    });
    c.bench_function("borsh_codec", |b| {
        b.iter(|| borsh_codec(black_box(packet.clone())))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
