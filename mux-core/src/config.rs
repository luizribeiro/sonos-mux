use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("TOML parsing error: {0}")]
    Toml(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Unknown input kind: {0}")]
    UnknownInputKind(String),

    #[error("Unknown output kind: {0}")]
    UnknownOutputKind(String),

    #[error("Duplicate ID: {0}")]
    DuplicateId(String),

    #[error("Referenced ID not found: {0}")]
    IdNotFound(String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    #[serde(default)]
    pub inputs: Vec<Input>,

    #[serde(default)]
    pub outputs: Vec<Output>,

    #[serde(default)]
    pub routes: Vec<Route>,

    #[serde(default)]
    pub logging: Option<Logging>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Input {
    pub id: String,
    pub kind: String,

    // ALSA specific
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device: Option<String>,

    // File specific
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,

    // Loop option for file input
    #[serde(skip_serializing_if = "Option::is_none")]
    pub loop_playback: Option<bool>,

    // HTTP specific
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Output {
    pub id: String,
    pub kind: String,

    // Sonos specific
    #[serde(skip_serializing_if = "Option::is_none")]
    pub room: Option<String>,

    // Buffer size in seconds (primarily for Sonos)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub buffer_sec: Option<u32>,

    // HTTP specific
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Route {
    pub input: String,
    pub outputs: Vec<String>,

    #[serde(default)]
    pub gain_db: f32,

    #[serde(default)]
    pub duck_db: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Logging {
    #[serde(default = "default_log_level")]
    pub level: String,

    #[serde(default)]
    pub file: Option<String>,
}

fn default_log_level() -> String {
    "info".to_string()
}

impl Config {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let content = fs::read_to_string(path)?;
        Self::from_reader(content.as_bytes())
    }

    pub fn from_reader<R: std::io::Read>(mut reader: R) -> Result<Self, ConfigError> {
        let mut content = String::new();
        reader
            .read_to_string(&mut content)
            .map_err(ConfigError::Io)?;

        let config: Config =
            toml::from_str(&content).map_err(|e| ConfigError::Toml(e.to_string()))?;

        config.validate()?;

        Ok(config)
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        // Check for valid input kinds
        for input in &self.inputs {
            match input.kind.as_str() {
                "alsa" | "file" | "http" | "silence" => {}
                _ => return Err(ConfigError::UnknownInputKind(input.kind.clone())),
            }
        }

        // Check for valid output kinds
        for output in &self.outputs {
            match output.kind.as_str() {
                "sonos" | "http" => {}
                _ => return Err(ConfigError::UnknownOutputKind(output.kind.clone())),
            }
        }

        // Check for duplicate IDs
        let mut input_ids = HashSet::new();
        let mut output_ids = HashSet::new();

        for input in &self.inputs {
            if !input_ids.insert(input.id.clone()) {
                return Err(ConfigError::DuplicateId(input.id.clone()));
            }
        }

        for output in &self.outputs {
            if !output_ids.insert(output.id.clone()) {
                return Err(ConfigError::DuplicateId(output.id.clone()));
            }
        }

        // Check that referenced IDs exist
        for route in &self.routes {
            if !input_ids.contains(&route.input) {
                return Err(ConfigError::IdNotFound(route.input.clone()));
            }

            for output_id in &route.outputs {
                if !output_ids.contains(output_id) {
                    return Err(ConfigError::IdNotFound(output_id.clone()));
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_temp_config(content: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(content.as_bytes()).unwrap();
        file
    }

    #[test]
    fn test_happy_path_parse() {
        let content = r#"
[[inputs]]
id = "roon_main"
kind = "alsa"
device = "hw:Loopback,1"

[[outputs]]
id = "living_room"
kind = "sonos"
room = "Living Room"
buffer_sec = 5

[[routes]]
input = "roon_main"
outputs = ["living_room"]
"#;

        let file = create_temp_config(content);
        let config = Config::load(file.path()).unwrap();

        assert_eq!(config.inputs.len(), 1);
        assert_eq!(config.inputs[0].id, "roon_main");
        assert_eq!(config.inputs[0].kind, "alsa");
        assert_eq!(config.inputs[0].device, Some("hw:Loopback,1".to_string()));

        assert_eq!(config.outputs.len(), 1);
        assert_eq!(config.outputs[0].id, "living_room");
        assert_eq!(config.outputs[0].kind, "sonos");
        assert_eq!(config.outputs[0].room, Some("Living Room".to_string()));
        assert_eq!(config.outputs[0].buffer_sec, Some(5));

        assert_eq!(config.routes.len(), 1);
        assert_eq!(config.routes[0].input, "roon_main");
        assert_eq!(config.routes[0].outputs, vec!["living_room"]);
    }

    #[test]
    fn test_unknown_kind() {
        let content = r#"
[[inputs]]
id = "invalid"
kind = "invalid_kind"
"#;

        let file = create_temp_config(content);
        let result = Config::load(file.path());

        assert!(result.is_err());
        match result {
            Err(ConfigError::UnknownInputKind(kind)) => assert_eq!(kind, "invalid_kind"),
            _ => panic!("Expected UnknownInputKind error"),
        }
    }

    #[test]
    fn test_duplicate_id() {
        let content = r#"
[[inputs]]
id = "duplicate"
kind = "alsa"

[[inputs]]
id = "duplicate"
kind = "alsa"
"#;

        let file = create_temp_config(content);
        let result = Config::load(file.path());

        assert!(result.is_err());
        match result {
            Err(ConfigError::DuplicateId(id)) => assert_eq!(id, "duplicate"),
            _ => panic!("Expected DuplicateId error"),
        }
    }

    #[test]
    fn test_referenced_id_not_found() {
        let content = r#"
[[inputs]]
id = "input1"
kind = "alsa"

[[outputs]]
id = "output1"
kind = "sonos"

[[routes]]
input = "non_existent"
outputs = ["output1"]
"#;

        let file = create_temp_config(content);
        let result = Config::load(file.path());

        assert!(result.is_err());
        match result {
            Err(ConfigError::IdNotFound(id)) => assert_eq!(id, "non_existent"),
            _ => panic!("Expected IdNotFound error"),
        }
    }
}
