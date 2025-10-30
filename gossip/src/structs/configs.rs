use super::{MyNetworkBehaviours, PeerInfo};
use crate::errors::MessagePropogationErrors;
use libp2p::{
    PeerId, Swarm, SwarmBuilder,
    gossipsub::{self, IdentTopic},
    identity::Keypair,
    mdns, noise, tcp, yamux,
};
use std::{
    collections::HashMap,
    hash::{DefaultHasher, Hash, Hasher},
    time::Duration,
};

pub struct PeerIdentity {
    pub peer: Keypair,
    pub known_peers: HashMap<PeerId, PeerInfo>,
    pub subscribed_topics: Vec<String>,
    pub swarm: Option<Swarm<MyNetworkBehaviours>>,
}

impl PeerIdentity {
    pub fn new() -> Self {
        PeerIdentity {
            peer: Keypair::generate_ecdsa(),
            known_peers: HashMap::new(),
            subscribed_topics: vec![],
            swarm: None,
        }
    }

    pub fn build_swarm(&mut self) -> Result<(), MessagePropogationErrors> {
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
                // self.peer_id = key.
                let message_id_fn = |message: &gossipsub::Message| {
                    let mut s = DefaultHasher::new();
                    message.data.hash(&mut s);
                    gossipsub::MessageId::from(s.finish().to_string())
                };

                // Set a custom gossipsub configuration
                let gossipsub_config = gossipsub::ConfigBuilder::default()
                    .heartbeat_interval(Duration::from_secs(10)) // This is set to aid debugging by not cluttering the log space
                    .validation_mode(gossipsub::ValidationMode::Strict) // This sets the kind of message validation. The default is Strict (enforce message
                    // signing)
                    .message_id_fn(message_id_fn) // content-address messages. No two messages of the same content will be propagated.
                    .build()
                    .map_err(std::io::Error::other)?; // Temporary hack because `build` does not return a proper `std::error::Error`.

                let gossip = gossipsub::Behaviour::new(
                    gossipsub::MessageAuthenticity::Signed(key.clone()),
                    gossipsub_config,
                )?;

                let mdns_config =
                    mdns::Behaviour::new(mdns::Config::default(), key.public().to_peer_id())?;

                self.peer = key.clone();

                Ok(MyNetworkBehaviours {
                    gossip,
                    mdns: mdns_config,
                })
            })
            .map_err(|_| MessagePropogationErrors::UnableToBuildSwarm)?
            .build();

        self.swarm = Some(swarm);

        Ok(())
    }

    /// Send message to a specific topic
    pub fn send_message(
        &mut self,
        topic: &str,
        message: &str,
    ) -> Result<(), MessagePropogationErrors> {
        if self.is_subscribed_to(topic) {
            let to_topic = IdentTopic::new(topic);

            if let Err(e) = self
                .swarm
                .as_mut()
                .unwrap()
                .behaviour_mut()
                .gossip
                .publish(to_topic, message.as_bytes())
            {
                return Err(MessagePropogationErrors::UnableToSendMessage);
            }

            Ok(())
        } else {
            return Err(MessagePropogationErrors::NotPartOfTopic);
        }
    }

    /// Subscribe to a specific topic in the p2p network
    #[inline(always)]
    pub fn subscribe(&mut self, topic: &str) -> Result<(), MessagePropogationErrors> {
        let to_topic = IdentTopic::new(topic);

        if let Err(e) = self
            .swarm
            .as_mut()
            .unwrap()
            .behaviour_mut()
            .gossip
            .subscribe(&to_topic)
        {
            println!("{e:?}");

            return Err(MessagePropogationErrors::UnableToSubscribe);
        }

        self.subscribed_topics.push(topic.to_string());

        Ok(())
    }

    /// Subscribe from a specific topic in the p2p network
    pub fn unsubscribe(&mut self, topic: &str) -> Result<(), MessagePropogationErrors> {
        let to_topic = IdentTopic::new(topic);

        // Return true if the peer was already subscribed to the topic
        if !self
            .swarm
            .as_mut()
            .unwrap()
            .behaviour_mut()
            .gossip
            .unsubscribe(&to_topic)
        {
            return Err(MessagePropogationErrors::UnableToUnsubscribe);
        }

        self.subscribed_topics.retain(|item| item.ne(topic));
        Ok(())
    }

    #[inline(always)]
    pub fn is_subscribed_to(&self, topic: &str) -> bool {
        for subscribed_topic in &self.subscribed_topics {
            if subscribed_topic.eq(topic) {
                return true;
            }
        }
        false
    }
}
