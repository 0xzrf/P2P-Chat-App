use crate::{
    Commands, MessagePropogationErrors, MessageType, MyNetworkBehavioursEvent, PeerIdentity,
    PeerInfo,
    constants::{BROADCAST_SUBSCRIBE, BROADCAST_UNSUBSCRIBE},
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
        Self::print_minimal_welcome();
        let mut peer = PeerIdentity::build()?;

        let swarm = peer.swarm.as_mut().unwrap();

        swarm
            .listen_on("/ip4/0.0.0.0/udp/0/quic-v1".parse().unwrap())
            .map_err(|_| MessagePropogationErrors::UnableToBuildSwarm)?;

        // Making peer subscribe to global topic to hear to broadcasts
        let global_topic = IdentTopic::new("global");
        swarm
            .behaviour_mut()
            .gossip
            .subscribe(&global_topic)
            .map_err(|_| MessagePropogationErrors::UnableToSubscribe)?;
        let mut stdin = tokio::io::BufReader::new(tokio::io::stdin()).lines();

        loop {
            select! {
                Ok(Some(line)) = stdin.next_line() => {
                    let line = line.trim();
                    let (cmd, arg) = line.split_once(" ").unwrap_or((&line, ""));

                    let command = Self::return_cmd_type(cmd, arg);

                    match command {
                        Commands::Subscribe(topic) => {
                            let to_topic = IdentTopic::new(&topic);
                            if let Err(e) = swarm.behaviour_mut().gossip.subscribe(&to_topic) {
                                println!("{e:?}");
                            } else {
                                let global_topic = IdentTopic::new("global");
                                print_center(format!("Subscribed to {topic} sucessfully"));
                                if let Err(_) = swarm.behaviour_mut().gossip.publish(global_topic, format!("{BROADCAST_SUBSCRIBE} {topic}").as_bytes()) {
                                    println!("Unable to send subscription broadcast to peer")
                                }
                                peer.subscribed_topics.push(topic);
                            }
                        },
                        Commands::Unsubscribe(topic) => {
                            let to_topic = IdentTopic::new(&topic);
                            if !swarm.behaviour_mut().gossip.unsubscribe(&to_topic) {
                                println!("Not subscribed to topic");
                            } else {
                                let global_topic = IdentTopic::new("global");

                                print_center(format!("Unsubscribed from {topic} succesfully"));
                                if let Err(_) = swarm.behaviour_mut().gossip.publish(global_topic, format!("{BROADCAST_UNSUBSCRIBE} {topic}").as_bytes()) {
                                    println!("Unable to send subscription broadcast to peer")
                                }
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
                        Commands::Help => {
                            Self::print_help();
                        },
                        Commands::PrintKnownNodes => {
                            for (peer, peer_info) in &peer.known_peers {
                                println!("----------------------");
                                println!("PeerId: {}", peer.to_string());
                                println!("MultiAddr: {}", peer_info.multi_addr.to_string());
                                println!("Subscribed to:");
                                if peer_info.subscribed_topics.len() == 0 {
                                    println!("\tNot yet subscribed to any topic");
                                } else {
                                    for topic in &peer_info.subscribed_topics {
                                        println!("\t{topic}");
                                    }
                                }
                            }
                        },
                        Commands::PrintInfo => {
                            println!("PeerId: {}", peer.peer.public().to_peer_id().to_string());
                            println!("Multiaddr: {}", peer.multi_addr.take().unwrap().to_string());
                            if peer.subscribed_topics.len() == 0 {
                                println!("\tNot yet subscribed to any topic");
                            } else {
                                for topic in &peer.subscribed_topics {
                                    println!("\t{topic}");
                                }
                            };
                        },
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
                            peer.known_peers.remove_entry(&peer_id);
                        }
                    },
                    SwarmEvent::Behaviour(MyNetworkBehavioursEvent::Gossip(gossipsub::Event::Message {
                        propagation_source: peer_id,
                        message_id: _,
                        message,
                    })) => {
                        let broadcast_type = Self::return_msg_type(&String::from_utf8_lossy(&message.data).to_string());

                        match broadcast_type {
                            MessageType::PeerSubscribed(topic) => {
                                let peer_value = peer.known_peers.get_mut(&peer_id);
                                if let Some(peer) = peer_value {
                                    peer.subscribed_topics.push(topic);
                                } else {
                                    println!("Broadcast message received from an unknown peer");
                                }
                            },
                            MessageType::PeerUnsubscribed(topic) => {
                                let peer_value = peer.known_peers.get_mut(&peer_id);
                                if let Some(peer) = peer_value {
                                    peer.subscribed_topics.retain(|item| item.ne(&topic));
                                } else {
                                    println!("Broadcast message received from an unknown peer");
                                }
                            },
                            MessageType::TopicMessage => {
                                println!(
                                    "{peer_id}: {}",
                                    String::from_utf8_lossy(&message.data),
                                );
                            },
                            MessageType::InvalidMessage => {},
                        };


                    },
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
            "/known" => Commands::PrintKnownNodes,
            "/info" => Commands::PrintInfo,
            _ => Commands::InvalidCommand,
        }
    }

    pub fn return_msg_type(msg: &str) -> MessageType {
        let (broadcast_type, topic) = msg.split_once(" ").unwrap_or((msg, ""));

        match broadcast_type {
            BROADCAST_SUBSCRIBE => {
                if topic.is_empty() {
                    return MessageType::InvalidMessage;
                } else {
                    return MessageType::PeerSubscribed(topic.to_string());
                }
            }
            BROADCAST_UNSUBSCRIBE => {
                if topic.is_empty() {
                    return MessageType::InvalidMessage;
                } else {
                    return MessageType::PeerUnsubscribed(topic.to_string());
                }
            }
            _ => MessageType::TopicMessage,
        }
    }

    pub fn print_minimal_welcome() {
        const BOLD: &str = "\x1b[1m";
        const CYAN: &str = "\x1b[36m";
        const GREEN: &str = "\x1b[32m";
        const RESET: &str = "\x1b[0m";

        println!(
            "{}{}🎯 P2P Chat app{} - Ready to connect!",
            BOLD, CYAN, RESET
        );
        println!(
            "{}Type /help for commands or /subscribe <topic> to get started{}",
            GREEN, RESET
        );
        println!(
            "{}─────────────────────────────────────────────────{}",
            CYAN, RESET
        );
        println!();
        Self::print_help();
    }

    pub fn print_help() {
        const BOLD: &str = "\x1b[1m";
        const YELLOW: &str = "\x1b[33m";
        const RESET: &str = "\x1b[0m";
        const BLUE: &str = "\x1b[34m";

        println!("{BOLD}{BLUE}📋 Quick Commands:{RESET}");

        println!("{YELLOW}   /subscribe <topic>   - Subscribe to a topic");
        println!("   /unsubscribe         - Unsubscribe from topic{RESET}");
        println!("{BLUE}   /send global <msg>   - Send message to all peers");
        println!(
            "   /send <topic> <msg>  - Send message to all peers subscribed to a topic{RESET}"
        );
        println!("   /known               - List known peer and their info.");
        println!("   /help                - Show all commands");
        println!("   /quit                - Exit the application{RESET}");
    }
}
