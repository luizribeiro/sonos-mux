// sonos-mux core library
pub mod config;

// Re-export main types for convenience
pub use config::{Config, ConfigError, Input, Logging, Output, Route};

#[derive(Debug, thiserror::Error)]
pub enum MuxError {
    #[error("Configuration error: {0}")]
    Config(#[from] config::ConfigError),

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
