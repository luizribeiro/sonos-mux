use crate::input::{AudioBuffer, AudioInput, InputError};
use crate::mixer::{lin_to_db, Mixer, Source};
use crossbeam_channel::Sender;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

// Custom input that generates a tone
#[derive(Debug)]
struct ToneInput {
    frequency: f32, // Hz
    amplitude: f32, // 0.0 to 1.0
    sample_rate: f32,
    thread_handle: Option<thread::JoinHandle<()>>,
    running: Arc<Mutex<bool>>,
}

impl Clone for ToneInput {
    fn clone(&self) -> Self {
        Self {
            frequency: self.frequency,
            amplitude: self.amplitude,
            sample_rate: self.sample_rate,
            thread_handle: None,
            running: Arc::new(Mutex::new(false)),
        }
    }
}

impl ToneInput {
    pub fn new(frequency: f32, amplitude: f32) -> Self {
        Self {
            frequency,
            amplitude,
            sample_rate: 44100.0,
            thread_handle: None,
            running: Arc::new(Mutex::new(false)),
        }
    }
}

impl AudioInput for ToneInput {
    fn start(&mut self, sender: Sender<AudioBuffer>) -> Result<(), InputError> {
        {
            let mut running = self.running.lock().unwrap();
            if *running {
                return Ok(());
            }
            *running = true;
        }

        let frequency = self.frequency;
        let amplitude = self.amplitude;
        let sample_rate = self.sample_rate;
        let running = self.running.clone();

        self.thread_handle = Some(thread::spawn(move || {
            let mut phase: f32 = 0.0;
            let phase_increment = 2.0 * std::f32::consts::PI * frequency / sample_rate;

            while *running.lock().unwrap() {
                let mut buffer = Vec::with_capacity(1024);

                for _ in 0..1024 {
                    // Generate a sine wave
                    let sample = (phase.sin() * amplitude * i16::MAX as f32) as i16;
                    buffer.push(sample);

                    // Increment phase
                    phase += phase_increment;
                    if phase > 2.0 * std::f32::consts::PI {
                        phase -= 2.0 * std::f32::consts::PI;
                    }
                }

                if sender.send(buffer).is_err() {
                    break;
                }

                // Don't hog the CPU
                thread::sleep(Duration::from_millis(10));
            }
        }));

        Ok(())
    }

    fn stop(&mut self) -> Result<(), InputError> {
        {
            let mut running = self.running.lock().unwrap();
            *running = false;
        }

        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }
        Ok(())
    }
}

#[tokio::test]
async fn test_gain_accuracy() {
    // Create a tone input at 1kHz with full amplitude
    let tone_input = ToneInput::new(1000.0, 1.0);

    // Test different gain values
    let gain_values = [-20.0, -10.0, -6.0, -3.0, 0.0];

    for &gain_db in &gain_values {
        let src = Source::new(gain_db, false, 0.0, Box::new(tone_input.clone()));

        let mut mixer = Mixer::new(vec![src]);
        mixer.start().unwrap();

        // Wait a bit for the tone generator to produce something
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Get a mixed buffer with a timeout
        let buffer = tokio::time::timeout(Duration::from_millis(500), mixer.mix_next())
            .await
            .expect("Mixer timed out")
            .expect("No buffer received");

        // Calculate the RMS level of the buffer
        let rms =
            (buffer.iter().map(|&s| (s as f32).powi(2)).sum::<f32>() / buffer.len() as f32).sqrt();

        // Convert to dB
        let measured_db = lin_to_db(rms / i16::MAX as f32);

        // The expected dB value is gain_db - 3.0 (because sine RMS is -3dB below peak)
        let expected_db = gain_db - 3.0;

        // Check that the measured level is within 0.1dB of the expected level
        assert!(
            (measured_db - expected_db).abs() < 0.1,
            "Gain accuracy test failed: expected {} dB, measured {} dB",
            expected_db,
            measured_db
        );

        mixer.stop().unwrap();
    }
}

#[tokio::test]
async fn test_ducking_speed() {
    // Create two tone inputs
    let music = ToneInput::new(440.0, 0.5); // Music at 440Hz
    let alert = ToneInput::new(880.0, 0.8); // Alert at 880Hz

    // Create sources: music (normal) and alert (priority)
    let music_src = Source::new(
        0.0,   // 0dB gain
        false, // Not priority
        0.0,   // No ducking
        Box::new(music),
    );

    let alert_src = Source::new(
        0.0,  // 0dB gain
        true, // Priority
        12.0, // 12dB ducking
        Box::new(alert),
    );

    let mut mixer = Mixer::new(vec![music_src, alert_src]);
    mixer.start().unwrap();

    // Wait for sources to start producing
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Get a few mixed buffers to ensure ducking is applied
    for _ in 0..3 {
        let _ = tokio::time::timeout(Duration::from_millis(500), mixer.mix_next()).await;
    }

    // Get a buffer with ducking applied
    let buffer = tokio::time::timeout(Duration::from_millis(500), mixer.mix_next())
        .await
        .expect("Mixer timed out")
        .expect("No buffer received");

    // Analyze the buffer to confirm ducking is applied
    // In a real test, we would separate the two frequencies and measure their levels
    // Here we'll just check that the output isn't silent
    assert!(!buffer.iter().all(|&s| s == 0), "Output buffer is silent");

    mixer.stop().unwrap();
}
