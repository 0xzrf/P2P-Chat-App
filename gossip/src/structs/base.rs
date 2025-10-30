use libp2p::{Multiaddr, swarm::NetworkBehaviour};

pub struct PeerInfo {
    multi_addr: Multiaddr,
    subscribed_topics: Vec<String>,
}

#[derive(NetworkBehaviour)]
pub struct MyNetworkBehaviours {
    pub gossip: libp2p::gossipsub::Behaviour,
    pub mdns: libp2p::mdns::tokio::Behaviour,
}
