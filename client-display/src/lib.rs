//! client-display
//!
//! This crate helps display and manage the workflow
//!
//! of the client side of the node, implementing sending and receiving messages over the
//!
//! gossip protocol
use futures::prelude::*;
use gossip::{MessagePropogationErrors, MyNetworkBehavioursEvent, PeerIdentity};
use libp2p::{
    gossipsub::{self, IdentTopic},
    mdns,
    swarm::SwarmEvent,
};
use tokio::{io::AsyncBufReadExt, select};

pub struct Client;

pub enum Commands<T> {
    Subscribe(T),
    Unsubscribe(T),
    SendMessage((T, T)),
    Help,
    InvalidCommand,
}

impl Client {
    /// Handles the entire client code
    pub fn run() -> Result<(), MessagePropogationErrors> {
        Ok(())
    }

    pub async fn build() -> Result<(), MessagePropogationErrors> {
        let mut peer = PeerIdentity::new();

        peer.build_swarm()?;

        let swarm = peer.swarm.as_mut().unwrap();

        swarm
            .listen_on("/ip4/0.0.0.0/udp/0/quic-v1".parse().unwrap())
            .map_err(|_| MessagePropogationErrors::UnableToBuildSwarm)?;

        let mut stdin = tokio::io::BufReader::new(tokio::io::stdin()).lines();

        loop {
            select! {
                Ok(Some(line)) = stdin.next_line() => {
                    let line = line.trim();
                    let (cmd, arg) = line.split_once(" ").unwrap();

                    let command = Self::return_cmd_type(cmd, arg);

                    match command {
                        Commands::Subscribe(topic) => {
                            let to_topic = IdentTopic::new(topic);
                            if let Err(e) = swarm.behaviour_mut().gossip.subscribe(&to_topic) {
                                println!("{e:?}");
                            }
                        },
                        Commands::Unsubscribe(topic) => {
                            let to_topic = IdentTopic::new(topic);
                            if !swarm.behaviour_mut().gossip.unsubscribe(&to_topic) {
                                println!("Not subscribed to topic");
                            }
                        },
                        Commands::SendMessage((topic, message)) => {
                            let to_topic = IdentTopic::new(topic);
                            if let Err(e) = swarm.behaviour_mut().gossip.publish(to_topic, message.as_bytes()) {
                                println!("{e:?}");
                            }
                        },
                        Commands::Help => {},
                        Commands::InvalidCommand => {
                            println!("Invalid Command format, type /help to see manual")
                        }
                    }

                }
                event = swarm.select_next_some() => match event {
                    SwarmEvent::Behaviour(MyNetworkBehavioursEvent::Mdns(mdns::Event::Discovered(list))) => {
                        for (peer_id, _multiaddr) in list {
                            println!("mDNS discovered a new peer: {peer_id}");
                            swarm.behaviour_mut().gossip.add_explicit_peer(&peer_id);
                        }
                    },
                    SwarmEvent::Behaviour(MyNetworkBehavioursEvent::Mdns(mdns::Event::Expired(list))) => {
                        for (peer_id, _multiaddr) in list {
                            println!("mDNS discover peer has expired: {peer_id}");
                            swarm.behaviour_mut().gossip.remove_explicit_peer(&peer_id);
                        }
                    },
                    SwarmEvent::Behaviour(MyNetworkBehavioursEvent::Gossip(gossipsub::Event::Message {
                        propagation_source: peer_id,
                        message_id: id,
                        message,
                    })) => println!(
                            "Got message: '{}' with id: {id} from peer: {peer_id}",
                            String::from_utf8_lossy(&message.data),
                        ),
                    SwarmEvent::NewListenAddr { address, .. } => {
                        println!("Local node is listening on {address}");
                    }
                    _ => {}
                }
            }
        }
    }

    pub fn return_cmd_type(cmd: &str, arg: &str) -> Commands<String> {
        match cmd {
            "/subscribe" => Commands::Subscribe(arg.to_string()),
            "/unsubscribe" => Commands::Unsubscribe(arg.to_string()),
            "/send" => {
                let split_val = arg.split_once(" ");

                if let Some((topic, message)) = split_val {
                    return Commands::SendMessage((topic.to_string(), message.to_string()));
                } else {
                    return Commands::InvalidCommand;
                }
            }
            "/help" => Commands::Help,
            _ => Commands::InvalidCommand,
        }
    }
}
