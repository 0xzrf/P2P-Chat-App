use futures::prelude::*;
use libp2p::{
    Multiaddr, PeerId,
    mdns::{Config as MdnsConfig, Event as MdnsEvents, tokio::Behaviour as TokioBehaviour},
    noise,
    swarm::SwarmEvent,
    tcp, yamux,
};

use std::{collections::HashMap, error::Error};

pub struct PeerIdentity {
    pub peer_id: PeerId,
    pub known_peers: HashMap<PeerId, PeerInfo>,
}

/// Struct containing info of a specific Peer
pub struct PeerInfo {
    pub multi_addr: Multiaddr,
}

impl PeerIdentity {
    /// Initializes a new peer for a node
    pub fn new() -> Self {
        PeerIdentity {
            peer_id: PeerId::random(),
            known_peers: HashMap::new(),
        }
    }

    /// Discovers nodes and adds them to the known peers
    pub async fn find_peers(&mut self) -> Result<(), Box<dyn Error>> {
        let mut swarm = libp2p::SwarmBuilder::with_new_identity()
            .with_tokio()
            .with_tcp(
                tcp::Config::default(),
                noise::Config::new,
                yamux::Config::default,
            )?
            .with_behaviour(|_| {
                TokioBehaviour::new(MdnsConfig::default(), self.peer_id)
                    .expect("Couldn't create a tokio mdns behaviour")
            })?
            .build();

        swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

        loop {
            match swarm.select_next_some().await {
                SwarmEvent::Behaviour(MdnsEvents::Discovered(peers)) => {
                    for peer in peers {
                        println!(
                            "A new Peer joined:\nPeer ID: {}\nMultiAddr: {}",
                            peer.0.to_string(),
                            peer.1.to_string()
                        );

                        self.known_peers
                            .insert(peer.0, PeerInfo { multi_addr: peer.1 });
                    }
                }
                SwarmEvent::Behaviour(MdnsEvents::Expired(peers)) => {
                    for peer in peers {
                        println!("Peer expired: {}", peer.0.to_string());
                    }
                }
                _ => {}
            }
        }
    }
}
