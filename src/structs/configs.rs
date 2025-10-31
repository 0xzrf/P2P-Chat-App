use super::{MyNetworkBehaviours, PeerInfo};
use crate::errors::MessagePropogationErrors;
use libp2p::{
    Multiaddr, PeerId, Swarm, SwarmBuilder, gossipsub, identity::Keypair, mdns, noise, tcp, yamux,
};
use std::{collections::HashMap, time::Duration};

pub struct PeerIdentity {
    pub peer: Keypair,
    pub multi_addr: Option<Multiaddr>,
    pub known_peers: HashMap<PeerId, PeerInfo>,
    pub subscribed_topics: Vec<String>,
    pub swarm: Option<Swarm<MyNetworkBehaviours>>,
}

impl PeerIdentity {
    pub fn build() -> Result<Self, MessagePropogationErrors> {
        let mut peer_key: Keypair = Keypair::generate_ecdsa();
        let swarm = SwarmBuilder::with_new_identity()
            .with_tokio()
            .with_tcp(
                tcp::Config::default(),
                noise::Config::new,
                yamux::Config::default,
            )
            .map_err(|_| MessagePropogationErrors::UnableToBuildSwarm)?
            .with_quic()
            .with_behaviour(|key| {
                // Set a custom gossipsub configuration
                let gossipsub_config = gossipsub::ConfigBuilder::default()
                    .heartbeat_interval(Duration::from_secs(10)) // This is set to aid debugging by not cluttering the log space
                    .validation_mode(gossipsub::ValidationMode::Strict) // This sets the kind of message validation. The default is Strict (enforce message
                    // signing)
                    .build()
                    .map_err(std::io::Error::other)?; // Temporary hack because `build` does not return a proper `std::error::Error`.

                let gossip = gossipsub::Behaviour::new(
                    gossipsub::MessageAuthenticity::Signed(key.clone()),
                    gossipsub_config,
                )?;

                let mdns_config =
                    mdns::Behaviour::new(mdns::Config::default(), key.public().to_peer_id())?;

                peer_key = key.clone();

                Ok(MyNetworkBehaviours {
                    gossip,
                    mdns: mdns_config,
                })
            })
            .map_err(|_| MessagePropogationErrors::UnableToBuildSwarm)?
            .build();

        let new_peer = PeerIdentity {
            swarm: Some(swarm),
            peer: peer_key,
            known_peers: HashMap::new(),
            subscribed_topics: vec![],
            multi_addr: None, //MultiAddr not set to anything till we start listening to it
        };
        Ok(new_peer)
    }
}
