// use gossip::structs::PeerIdentity;
// use std::sync::Arc;
// use tokio::{spawn, sync::RwLock};

// #[tokio::main]
// async fn main() {
//     let peer = Arc::new(RwLock::new(PeerIdentity::new()));
//     let find_peer_clone = Arc::clone(&peer);

//     spawn(async move {
//         find_peer_clone.write().await.find_peers().await.unwrap();
//     })
//     .await
//     .unwrap();
// }
fn main() {}
