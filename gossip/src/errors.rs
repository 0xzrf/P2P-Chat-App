use thiserror::Error;

#[derive(Error, Debug)]
pub enum MessagePropogationErrors {
    #[error("Couldn't find peer type")]
    CouldNotFindPeerType,

    #[error("Unable to build swarm")]
    UnableToBuildSwarm,
    #[error("Failed to start listening")]
    FailedToStartListening,

    #[error("Could not send message to topic")]
    UnableToSendMessage,

    #[error("Not Part of Topic")]
    NotPartOfTopic,

    #[error("Unable To Subscribe")]
    UnableToSubscribe,

    #[error("Not Subscribed to topic")]
    UnableToUnsubscribe,
}
