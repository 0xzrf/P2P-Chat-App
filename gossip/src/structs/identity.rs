use libp2p::{SwarmBuilder, mdns, noise, tcp, yamux};

pub struct PeerIdentity {
    pub peer_id: String,
}

impl PeerIdentity {
    /// Initializes a new peer for a node
    pub fn new() -> Self {
        PeerIdentity {
            peer_id: String::from(""),
        }
    }

    pub async fn find_peers() {
        let mut swarm = SwarmBuilder::with_new_identity().with_tokio().with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        );
    }
}
