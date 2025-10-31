use libp2p::{Multiaddr, swarm::NetworkBehaviour};

pub struct PeerInfo {
    pub multi_addr: Multiaddr,
    pub subscribed_topics: Vec<String>,
}

pub enum Commands<T> {
    Subscribe(T),
    Unsubscribe(T),
    SendMessage((T, T)),
    Help,
    InvalidCommand,
}

#[derive(NetworkBehaviour)]
pub struct MyNetworkBehaviours {
    pub gossip: libp2p::gossipsub::Behaviour,
    pub mdns: libp2p::mdns::tokio::Behaviour,
}
