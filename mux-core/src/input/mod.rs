pub mod alsa;
pub mod file;
pub mod http;
pub mod silence;

use crossbeam_channel::Sender;
use std::fmt;
use thiserror::Error;

// Define the common audio frame format we'll use throughout the system
// S16LE 44.1 kHz stereo frames
pub type Frame = i16;
pub type AudioBuffer = Vec<Frame>;

#[derive(Debug, Error)]
pub enum InputError {
    #[error("Failed to initialize input: {0}")]
    Initialization(String),

    #[error("Failed to read from input: {0}")]
    Read(String),
}

pub trait AudioInput: Send + fmt::Debug + AudioInputClone {
    fn start(&mut self, sender: Sender<AudioBuffer>) -> Result<(), InputError>;
    fn stop(&mut self) -> Result<(), InputError>;
}

// Trait to enable cloning Box<dyn AudioInput>
pub trait AudioInputClone {
    fn clone_box(&self) -> Box<dyn AudioInput>;
}

impl<T> AudioInputClone for T
where
    T: 'static + AudioInput + Clone,
{
    fn clone_box(&self) -> Box<dyn AudioInput> {
        Box::new(self.clone())
    }
}

// Allow cloning of Box<dyn AudioInput>
impl Clone for Box<dyn AudioInput> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

// Factory function to create an audio input from config
pub fn create_input(config: &crate::config::Input) -> Result<Box<dyn AudioInput>, InputError> {
    match config.kind.as_str() {
        "alsa" => {
            let device = config
                .device
                .clone()
                .unwrap_or_else(|| "default".to_string());
            Ok(Box::new(alsa::AlsaInput::new(&device)?))
        }
        "file" => {
            let path = config
                .path
                .clone()
                .ok_or_else(|| InputError::Initialization("File path not specified".to_string()))?;
            let loop_playback = config.loop_playback.unwrap_or(false);
            Ok(Box::new(file::FileInput::new(&path, loop_playback)?))
        }
        "http" => {
            let url = config
                .url
                .clone()
                .ok_or_else(|| InputError::Initialization("HTTP URL not specified".to_string()))?;
            Ok(Box::new(http::HttpInput::new(&url)?))
        }
        "silence" => Ok(Box::new(silence::SilenceInput::new())),
        _ => Err(InputError::Initialization(format!(
            "Unsupported input kind: {}",
            config.kind
        ))),
    }
}
