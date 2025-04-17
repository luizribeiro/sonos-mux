// Output module for sonos-mux
pub mod sonos;

use async_trait::async_trait;
use std::error::Error;
use std::fmt::Debug;

/// Common trait for all output destinations
#[async_trait]
pub trait AudioOutput: Debug + Send + Sync {
    /// Initialize the output
    async fn initialize(&mut self) -> Result<(), Box<dyn Error + Send + Sync>>;

    /// Set the stream URL for this output
    async fn set_stream(&mut self, url: &str) -> Result<(), Box<dyn Error + Send + Sync>>;

    /// Verify connection and re-establish if needed
    async fn keep_alive(&mut self) -> Result<(), Box<dyn Error + Send + Sync>>;

    /// Get the health status of this output
    async fn health_check(&self) -> bool;
}

/// Output error types
#[derive(Debug, thiserror::Error)]
pub enum OutputError {
    #[error("Device discovery error: {0}")]
    Discovery(String),

    #[error("Connection error: {0}")]
    Connection(String),

    #[error("Stream setup error: {0}")]
    StreamSetup(String),

    #[error("Device not found: {0}")]
    DeviceNotFound(String),
}
