use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProtocolError {
    #[error("Subscription has expired")]
    SubscriptionExpired,
    
    #[error("Bandwidth limit exceeded")]
    BandwidthExceeded,
    
    #[error("Proxy node is offline or degraded")]
    NodeOffline,
    
    #[error("Invalid signature provided")]
    InvalidSignature,

    #[error("Network failure: {0}")]
    NetworkFailure(String),
}

pub type Result<T> = std::result::Result<T, ProtocolError>;
