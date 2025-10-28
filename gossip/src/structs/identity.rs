use crate::{constants::MAX_KNOWN_PEER, errors::IdentityErrors};
use futures::prelude::*;
use libp2p::{
    Multiaddr, PeerId,
    mdns::{Config as MdnsConfig, Event as MdnsEvents, tokio::Behaviour as TokioBehaviour},
    noise,
    swarm::SwarmEvent,
    tcp, yamux,
};
use rand::Rng;
use std::collections::HashMap;
pub struct PeerIdentity {
    peer_id: PeerId,
    known_peers: HashMap<PeerId, PeerInfo>, // Hashmap for easier PeerInfo
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
    pub async fn find_peers(&mut self) -> Result<(), IdentityErrors> {
        let mut swarm = libp2p::SwarmBuilder::with_new_identity()
            .with_tokio()
            .with_tcp(
                tcp::Config::default(),
                noise::Config::new,
                yamux::Config::default,
            )
            .map_err(|_| IdentityErrors::FailedToStartSwarm)?
            .with_behaviour(|_| {
                TokioBehaviour::new(MdnsConfig::default(), self.peer_id)
                    .expect("Couldn't create a tokio mdns behaviour")
            })
            .map_err(|_| IdentityErrors::FailedToStartSwarm)?
            .build();

        swarm
            .listen_on("/ip4/0.0.0.0/tcp/0".parse().unwrap()) // Okay since we're parsing a constant
            .map_err(|_| IdentityErrors::FailedToStartListening)?;

        loop {
            match swarm.select_next_some().await {
                SwarmEvent::Behaviour(MdnsEvents::Discovered(peers)) => {
                    for peer in peers {
                        println!(
                            "A new Peer joined:\nPeer ID: {}\nMultiAddr: {}",
                            peer.0.to_string(),
                            peer.1.to_string()
                        );

                        let mut rng = rand::rng();

                        // Limiting the no. of nodes known by a peer, so to avoid stack overflow as the network grows bigger
                        // also randomly adding nodes for the random p2p topology
                        if rng.random::<bool>() && self.known_peers.len() <= MAX_KNOWN_PEER {
                            self.known_peers
                                .insert(peer.0, PeerInfo { multi_addr: peer.1 });
                        }
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

    // GETTERS
    pub fn get_peer(&self) -> &PeerId {
        &self.peer_id
    }

    pub fn get_known_peers(&self) -> &HashMap<PeerId, PeerInfo> {
        &self.known_peers
    }
}
