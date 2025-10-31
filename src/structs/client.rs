use crate::{
    Commands, MessagePropogationErrors, MyNetworkBehavioursEvent, PeerIdentity, PeerInfo,
    helper::print_center,
};
use futures::prelude::*;
use libp2p::{
    gossipsub::{self, IdentTopic},
    mdns,
    swarm::SwarmEvent,
};
use tokio::{io::AsyncBufReadExt, select};

pub struct Client;

impl Client {
    pub async fn build_and_run() -> Result<(), MessagePropogationErrors> {
        let mut peer = PeerIdentity::build()?;

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
                            let to_topic = IdentTopic::new(&topic);
                            if let Err(e) = swarm.behaviour_mut().gossip.subscribe(&to_topic) {
                                println!("{e:?}");
                            } else {
                                print_center(format!("Subscribed to {topic} sucessfully"));
                                peer.subscribed_topics.push(topic);
                            }
                        },
                        Commands::Unsubscribe(topic) => {
                            let to_topic = IdentTopic::new(&topic);
                            if !swarm.behaviour_mut().gossip.unsubscribe(&to_topic) {
                                println!("Not subscribed to topic");
                            } else {
                                print_center(format!("Unsubscribed from {topic} succesfully"));
                                peer.subscribed_topics.retain(|val| val.ne(&topic));
                            }
                        },
                        Commands::SendMessage((topic, message)) => {
                            let to_topic = IdentTopic::new(&topic);
                            if let Err(e) = swarm.behaviour_mut().gossip.publish(to_topic, message.as_bytes()) {
                                print_center(format!("Unable to send message in topic: {topic}"));
                                print_center(format!("Reason: {e:?}"));
                            }
                        },
                        Commands::Help => {},
                        Commands::InvalidCommand => {
                            print_center("Invalid Command format, type /help to see manual".to_string());
                        }
                    }
                }
                event = swarm.select_next_some() => match event {
                    SwarmEvent::Behaviour(MyNetworkBehavioursEvent::Mdns(mdns::Event::Discovered(list))) => {
                        for (peer_id, multiaddr) in list {
                            println!("mDNS discovered a new peer: {peer_id}");
                            swarm.behaviour_mut().gossip.add_explicit_peer(&peer_id);
                            peer.known_peers.insert(peer_id, PeerInfo {multi_addr: multiaddr, subscribed_topics: vec![]});
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
                        message_id: _,
                        message,
                    })) => println!(
                            "{peer_id}: {}",
                            String::from_utf8_lossy(&message.data),
                        ),
                    SwarmEvent::NewListenAddr { address, .. } => {
                        println!("Local node is listening on {address}");
                        peer.multi_addr = Some(address);
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
