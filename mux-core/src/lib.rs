// Basic scaffold for the mux-core library
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MuxError {
    #[error("Configuration error: {0}")]
    Config(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub version: String,
}

impl Config {
    pub fn new(version: &str) -> Self {
        Self {
            version: version.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_new() {
        let config = Config::new("0.0.1");
        assert_eq!(config.version, "0.0.1");
    }
}
