use thiserror::Error;

#[derive(Error, Debug)]
pub enum IdentityErrors {
    #[error("Failed to start Swarm")]
    FailedToStartSwarm,

    #[error("Failed to start listening")]
    FailedToStartListening,
}
