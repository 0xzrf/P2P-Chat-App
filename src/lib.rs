//! Gossip crate
//!
//! This crate is a helper for communication over ther libp2p crate
//! For communication in the chat app
//! Discovers node over MDNS protocol provided by libp2p
pub mod constants;
pub mod errors;
pub mod helper;
pub mod structs;
pub use errors::*;
pub use structs::*;
