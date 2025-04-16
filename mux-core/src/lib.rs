// sonos-mux core library
pub mod config;
pub mod encoder;
pub mod input;
pub mod stream;

// Re-export main types for convenience
pub use config::{Config, ConfigError, Input, Logging, Output, Route};
pub use encoder::{EncoderError, Lame};
pub use input::{AudioBuffer, AudioInput, Frame, InputError};
pub use stream::{HttpStreamer, StreamError};

#[derive(Debug, thiserror::Error)]
pub enum MuxError {
    #[error("Configuration error: {0}")]
    Config(#[from] config::ConfigError),

    #[error("Input error: {0}")]
    Input(#[from] input::InputError),

    #[error("Encoder error: {0}")]
    Encoder(#[from] encoder::EncoderError),

    #[error("Stream error: {0}")]
    Stream(#[from] stream::StreamError),

    #[error("Internal error: {0}")]
    Internal(String),
}

/// Get the current version of the library
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!version().is_empty());
    }
}
