pub mod alsa;

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

pub trait AudioInput: Send + fmt::Debug {
    fn start(&mut self, sender: Sender<AudioBuffer>) -> Result<(), InputError>;
    fn stop(&mut self) -> Result<(), InputError>;
}

// Factory function to create an audio input from config
pub fn create_input(config: &crate::Input) -> Result<Box<dyn AudioInput>, InputError> {
    match config.kind.as_str() {
        "alsa" => {
            let device = config
                .device
                .clone()
                .unwrap_or_else(|| "default".to_string());
            Ok(Box::new(alsa::AlsaInput::new(&device)?))
        }
        // Will implement other input types in future sprints
        _ => Err(InputError::Initialization(format!(
            "Unsupported input kind: {}",
            config.kind
        ))),
    }
}
