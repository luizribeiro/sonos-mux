use crate::input::Frame;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EncoderError {
    #[error("Failed to initialize encoder: {0}")]
    Initialization(String),

    #[error("Failed to encode: {0}")]
    Encode(String),
}

// For Sprint 2, we'll create a mock implementation of the MP3 encoder
// This allows us to test the pipeline without requiring the LAME library
pub struct Lame {
    bitrate: i32,
    bytes_encoded: usize,
}

impl Lame {
    pub fn new(bitrate: i32) -> Result<Self, EncoderError> {
        Ok(Lame {
            bitrate,
            bytes_encoded: 0,
        })
    }

    // Encode interleaved stereo samples to a mock MP3 format
    pub fn encode(&mut self, pcm: &[Frame]) -> Result<Vec<u8>, EncoderError> {
        // In a real implementation, this would use LAME to encode the PCM data to MP3
        // For now, we'll just create a mock MP3 frame with a simple header

        // Calculate how many bytes we'd expect from real MP3 encoding
        // MP3 bitrate is in kbps, so bytes = (bitrate * 1000 / 8) * (num_samples / sample_rate)
        let num_samples = pcm.len() / 2; // Divide by 2 because we have interleaved stereo
        let sample_rate = 44100;
        let expected_bytes = (self.bitrate as usize * 1000 / 8) * num_samples / sample_rate;

        // Create a mock MP3 frame
        let mut buffer = Vec::with_capacity(expected_bytes);

        // Add a fake MP3 header (0xFF 0xE0 is the MP3 frame sync)
        buffer.push(0xFF);
        buffer.push(0xE0);

        // Add some dummy data to make up the expected size
        buffer.resize(expected_bytes, 0x00);

        self.bytes_encoded += buffer.len();

        Ok(buffer)
    }

    // Flush the encoder to get any remaining MP3 frames
    pub fn flush(&mut self) -> Result<Vec<u8>, EncoderError> {
        // In a real implementation, this would flush the LAME encoder
        // For now, we'll just return an empty buffer
        Ok(Vec::new())
    }

    // Get the total number of bytes encoded
    pub fn bytes_encoded(&self) -> usize {
        self.bytes_encoded
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn test_encode_sine_wave() {
        // Create a 1-second sine wave at 440Hz
        let sample_rate = 44100;
        let duration = 1.0; // seconds
        let frequency = 440.0; // Hz
        let num_samples = (sample_rate as f32 * duration) as usize;

        let mut pcm = Vec::with_capacity(num_samples * 2); // *2 for stereo

        for i in 0..num_samples {
            let t = i as f32 / sample_rate as f32;
            let sample = (2.0 * PI * frequency * t).sin() * 0.5;

            // Convert to i16 and add to both channels (interleaved)
            let sample_i16 = (sample * 32767.0) as i16;
            pcm.push(sample_i16); // Left channel
            pcm.push(sample_i16); // Right channel
        }

        // Create encoder and encode the sine wave
        let mut encoder = Lame::new(128).unwrap(); // 128kbps
        let mp3_data = encoder.encode(&pcm).unwrap();
        let flush_data = encoder.flush().unwrap();

        // Combine the encoded data and flush data
        let mut full_mp3 = mp3_data;
        full_mp3.extend_from_slice(&flush_data);

        // Check that we have some MP3 data
        assert!(!full_mp3.is_empty());

        // Check for MP3 frame sync word (0xFF 0xE0) in the first few bytes
        let has_mp3_header = full_mp3
            .windows(2)
            .any(|window| (window[0] == 0xFF) && ((window[1] & 0xE0) == 0xE0));

        assert!(has_mp3_header, "MP3 frame sync word not found");

        // Verify the size is reasonable for a 1-second MP3 at 128kbps
        // Expected size is approximately (128000 / 8) bytes = 16000 bytes
        let expected_size = 16000;
        let size_margin = 5000; // Allow for +/- 5KB

        assert!(
            full_mp3.len() > expected_size - size_margin
                && full_mp3.len() < expected_size + size_margin,
            "MP3 size is not within expected range: {} bytes (expected ~{})",
            full_mp3.len(),
            expected_size
        );
    }
}
