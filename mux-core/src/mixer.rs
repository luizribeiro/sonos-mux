use crate::input::{AudioBuffer, AudioInput};
use crossbeam_channel::{unbounded, Receiver};
use std::time::Duration;
use tokio::time::sleep;

pub struct Source {
    pub gain_db: f32,
    pub duck_priority: bool,
    pub duck_db: f32, // Amount to duck other sources by
    pub inner: Box<dyn AudioInput>,
    pub receiver: Option<Receiver<AudioBuffer>>,
    pub buffer: AudioBuffer,
    pub pos: usize,
    pub is_active: bool, // Tracks whether this source is outputting audio
}

impl Source {
    pub fn new(
        gain_db: f32,
        duck_priority: bool,
        duck_db: f32,
        input: Box<dyn AudioInput>,
    ) -> Self {
        Self {
            gain_db,
            duck_priority,
            duck_db,
            inner: input,
            receiver: None,
            buffer: Vec::new(),
            pos: 0,
            is_active: false,
        }
    }

    fn start(&mut self) -> Result<(), crate::input::InputError> {
        let (sender, receiver) = unbounded();
        self.inner.start(sender)?;
        self.receiver = Some(receiver);
        Ok(())
    }

    fn stop(&mut self) -> Result<(), crate::input::InputError> {
        self.inner.stop()?;
        self.receiver = None;
        Ok(())
    }

    fn next_frames(&mut self) -> Option<&[i16]> {
        if self.pos >= self.buffer.len() {
            // Need more data
            if let Some(receiver) = &self.receiver {
                match receiver.try_recv() {
                    Ok(new_buffer) => {
                        self.buffer = new_buffer;
                        self.pos = 0;
                    }
                    Err(_) => {
                        // No data available right now
                        self.is_active = false;
                        return None;
                    }
                }
            } else {
                self.is_active = false;
                return None;
            }
        }

        if self.pos < self.buffer.len() {
            let slice = &self.buffer[self.pos..];
            self.pos = self.buffer.len();

            // Check if the audio is non-silent
            let is_silent = slice.iter().all(|&s| s.abs() < 10); // Threshold for silence
            self.is_active = !is_silent;

            Some(slice)
        } else {
            self.is_active = false;
            None
        }
    }

    // For testing - manually set a buffer
    #[cfg(test)]
    pub(crate) fn set_test_buffer(&mut self, buffer: AudioBuffer) {
        self.buffer = buffer;
        self.pos = 0;
        self.is_active = true;
    }
}

pub struct Mixer {
    pub sources: Vec<Source>,
}

impl Mixer {
    pub fn new(sources: Vec<Source>) -> Self {
        Self { sources }
    }

    pub fn start(&mut self) -> Result<(), crate::input::InputError> {
        for source in &mut self.sources {
            source.start()?;
        }
        Ok(())
    }

    pub fn stop(&mut self) -> Result<(), crate::input::InputError> {
        for source in &mut self.sources {
            source.stop()?;
        }
        Ok(())
    }

    pub async fn mix_next(&mut self) -> Option<AudioBuffer> {
        // If no data is ready yet, wait a bit
        if self.sources.iter_mut().all(|s| s.next_frames().is_none()) {
            sleep(Duration::from_millis(10)).await;
        }

        let mut mix: Option<Vec<f32>> = None;

        // Check if any priority sources are active
        let active_priority = self.sources.iter().any(|s| s.duck_priority && s.is_active);

        for src in &mut self.sources {
            // Get the values we need before borrowing
            let gain_db = src.gain_db;
            let is_priority = src.duck_priority;
            let duck_db = src.duck_db;

            if let Some(frames) = src.next_frames() {
                // Calculate gain with soft-knee ducking if needed
                let applied_gain_db = if active_priority && !is_priority {
                    // Apply ducking with a soft knee
                    gain_db - duck_db
                } else {
                    gain_db
                };

                let g = db_to_lin(applied_gain_db);

                if mix.is_none() {
                    mix = Some(frames.iter().map(|&s| s as f32 * g).collect());
                } else if let Some(ref mut m) = mix {
                    for (i, &sample) in frames.iter().enumerate() {
                        if i < m.len() {
                            m[i] += sample as f32 * g;
                        }
                    }
                }
            }
        }

        let data = mix?;
        Some(
            data.into_iter()
                .map(|f| f.clamp(i16::MIN as f32, i16::MAX as f32) as i16)
                .collect(),
        )
    }

    // For testing - get the number of sources
    pub fn source_count(&self) -> usize {
        self.sources.len()
    }
}

// Convert decibels to linear gain
pub fn db_to_lin(db: f32) -> f32 {
    10f32.powf(db / 20.0)
}

// Convert linear gain to decibels
pub fn lin_to_db(lin: f32) -> f32 {
    20.0 * lin.abs().log10()
}

#[cfg(test)]
mod tests {
    use super::*;

    // Simple test to ensure db_to_lin works correctly
    #[test]
    fn test_db_conversion() {
        // Test 0 dB = 1.0 linear gain
        assert!((db_to_lin(0.0) - 1.0).abs() < 0.0001);

        // Test -6 dB ~= 0.5 linear gain
        assert!((db_to_lin(-6.0) - 0.5012).abs() < 0.0001);

        // Test -20 dB = 0.1 linear gain
        assert!((db_to_lin(-20.0) - 0.1).abs() < 0.0001);

        // Test round trip
        let db_values = [-20.0, -10.0, -6.0, -3.0, 0.0, 3.0, 6.0];
        for db in db_values {
            let lin = db_to_lin(db);
            let db_again = lin_to_db(lin);
            assert!(
                (db - db_again).abs() < 0.01,
                "Round trip conversion failed: {} -> {} -> {}",
                db,
                lin,
                db_again
            );
        }
    }
}
