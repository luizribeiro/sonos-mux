use crossbeam_channel::Sender;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use super::{AudioBuffer, AudioInput, InputError};

// Constants for our audio format
const SAMPLE_RATE: u32 = 44100;
const CHANNELS: u16 = 2;
const BUFFER_SIZE: usize = 1024; // Number of frames per buffer

#[derive(Debug)]
pub struct AlsaInput {
    device_name: String,
    running: Arc<Mutex<bool>>,
    thread_handle: Option<thread::JoinHandle<()>>,
}

impl Clone for AlsaInput {
    fn clone(&self) -> Self {
        // We don't clone the thread handle, just create a new instance
        Self {
            device_name: self.device_name.clone(),
            running: Arc::new(Mutex::new(false)),
            thread_handle: None,
        }
    }
}

impl AlsaInput {
    pub fn new(device_name: &str) -> Result<Self, InputError> {
        Ok(AlsaInput {
            device_name: device_name.to_string(),
            running: Arc::new(Mutex::new(false)),
            thread_handle: None,
        })
    }
}

impl AudioInput for AlsaInput {
    fn start(&mut self, sender: Sender<AudioBuffer>) -> Result<(), InputError> {
        // For Sprint 2, we'll create a mock implementation that generates a sine wave
        // instead of actually capturing from ALSA
        // This allows us to test the pipeline without needing ALSA hardware

        // Set the running flag
        {
            let mut running = self.running.lock().unwrap();
            *running = true;
        }

        let running_clone = Arc::clone(&self.running);
        let device_name = self.device_name.clone();

        // Create a thread that generates a sine wave
        let thread_handle = thread::spawn(move || {
            println!("Started mock ALSA input for device: {}", device_name);

            // Generate a 440Hz sine wave
            let frequency = 440.0; // Hz
            let sample_rate = SAMPLE_RATE as f32;
            let mut phase = 0.0;

            while *running_clone.lock().unwrap() {
                let mut buffer = Vec::with_capacity(BUFFER_SIZE * CHANNELS as usize);

                // Generate one buffer of sine wave samples
                for _ in 0..BUFFER_SIZE {
                    // Calculate the sample value
                    let sample = (phase * 2.0 * std::f32::consts::PI).sin();

                    // Convert to i16 and add to buffer (both channels)
                    let sample_i16 = (sample * 8192.0) as i16; // Not too loud
                    buffer.push(sample_i16); // Left channel
                    buffer.push(sample_i16); // Right channel

                    // Increment phase
                    phase += frequency / sample_rate;
                    if phase >= 1.0 {
                        phase -= 1.0;
                    }
                }

                // Send the buffer
                match sender.send(buffer) {
                    Ok(_) => {}
                    Err(_) => {
                        // Receiver dropped, exit the loop
                        break;
                    }
                }

                // Sleep a bit to simulate real-time audio
                thread::sleep(Duration::from_millis(
                    (1000 * BUFFER_SIZE / SAMPLE_RATE as usize) as u64,
                ));
            }

            println!("Stopped mock ALSA input for device: {}", device_name);
        });

        self.thread_handle = Some(thread_handle);
        Ok(())
    }

    fn stop(&mut self) -> Result<(), InputError> {
        // Set the running flag to false to stop the thread
        {
            let mut running = self.running.lock().unwrap();
            *running = false;
        }

        // Wait for the thread to finish
        if let Some(handle) = self.thread_handle.take() {
            match handle.join() {
                Ok(_) => {}
                Err(_) => {
                    return Err(InputError::Read("Failed to join ALSA thread".to_string()));
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossbeam_channel::unbounded;
    use std::time::Duration;

    #[test]
    fn test_alsa_input() {
        let mut input = AlsaInput::new("test_device").unwrap();
        let (sender, receiver) = unbounded();

        input.start(sender).unwrap();

        // Wait for a few frames to be received
        let mut frame_count = 0;
        let timeout = Duration::from_secs(1);

        // Try to receive 1000 frames (which should be quick with our mock implementation)
        while frame_count < 1000 {
            match receiver.recv_timeout(timeout) {
                Ok(buffer) => {
                    frame_count += buffer.len() / 2; // Divide by 2 for stereo
                    println!(
                        "Received {} frames, total: {}",
                        buffer.len() / 2,
                        frame_count
                    );
                }
                Err(_) => {
                    break;
                }
            }
        }

        input.stop().unwrap();

        assert!(frame_count >= 1000, "Failed to receive 1000 frames");
    }
}
