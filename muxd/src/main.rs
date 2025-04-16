use clap::Parser;
use crossbeam_channel::bounded;
use log::{error, info};
use mux_core::{AudioBuffer, Config, HttpStreamer, Lame, MuxError};
use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;
use std::time::{Duration, Instant};
use tokio::runtime::Runtime;

#[derive(Parser)]
#[command(author, version, about = "Sonos audio multiplexer daemon")]
struct Args {
    /// Path to the configuration file
    #[arg(short, long)]
    config: PathBuf,
}

// Calculate RMS loudness of audio buffer
fn calculate_loudness(buffer: &[i16]) -> f32 {
    if buffer.is_empty() {
        return -60.0; // Silent
    }

    let sum_squares: f64 = buffer
        .iter()
        .map(|&sample| {
            let sample_f64 = f64::from(sample) / 32768.0;
            sample_f64 * sample_f64
        })
        .sum();

    let rms = (sum_squares / buffer.len() as f64).sqrt();

    // Convert to dB, with floor of -60 dB
    if rms > 0.0 {
        (20.0 * rms.log10()) as f32
    } else {
        -60.0
    }
}

fn main() -> Result<(), MuxError> {
    // Initialize logging
    env_logger::init();

    // Parse command line arguments
    let args = Args::parse();

    info!(
        "muxd v{} - Sonos audio multiplexer daemon",
        mux_core::version()
    );
    info!("Loading configuration from: {}", args.config.display());

    // Load and validate configuration
    let config = Config::load(&args.config)?;

    info!("Configuration loaded successfully!");
    info!("Inputs: {}", config.inputs.len());
    info!("Outputs: {}", config.outputs.len());
    info!("Routes: {}", config.routes.len());

    // For Sprint 2, we'll implement a simple pipeline:
    // 1. Create an ALSA input from the first input in the config
    // 2. Create an MP3 encoder
    // 3. Create an HTTP streamer
    // 4. Wire them together

    // Check if we have at least one input and one output
    if config.inputs.is_empty() {
        return Err(MuxError::Internal(
            "No inputs defined in config".to_string(),
        ));
    }

    if config.outputs.is_empty() {
        return Err(MuxError::Internal(
            "No outputs defined in config".to_string(),
        ));
    }

    // For Sprint 2, we'll just use the first input and output
    let input_config = &config.inputs[0];
    let output_config = &config.outputs[0];

    info!("Using input: {} ({})", input_config.id, input_config.kind);
    info!(
        "Using output: {} ({})",
        output_config.id, output_config.kind
    );

    // Create the audio input
    let mut audio_input = mux_core::input::create_input(input_config)?;

    // Create the MP3 encoder
    let mut encoder = Lame::new(128)?; // 128 kbps

    // Create the HTTP streamer
    let port = output_config.port.unwrap_or(8000);
    let streamer = HttpStreamer::new(port);

    // Create tokio runtime for the HTTP streamer
    let rt = Runtime::new()
        .map_err(|e| MuxError::Internal(format!("Failed to create runtime: {}", e)))?;
    rt.block_on(async {
        streamer.start().await.map_err(MuxError::Stream)?;
        Ok::<_, MuxError>(())
    })?;

    info!("HTTP streamer started on port {}", port);
    info!("Stream available at http://localhost:{}/stream.mp3", port);

    // Create channels for the pipeline
    let (audio_sender, audio_receiver) = bounded::<AudioBuffer>(10);

    // Flag to signal shutdown
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();

    // Start the audio input
    audio_input.start(audio_sender)?;

    // Create a thread to process audio and send to the streamer
    let processor_thread = thread::spawn(move || {
        let mut last_stats = Instant::now();
        let mut total_bytes_sent = 0;
        let mut max_loudness = -60.0f32;

        while running_clone.load(Ordering::SeqCst) {
            // Receive audio data
            match audio_receiver.recv_timeout(Duration::from_millis(100)) {
                Ok(buffer) => {
                    // Calculate loudness
                    let loudness = calculate_loudness(&buffer);
                    max_loudness = max_loudness.max(loudness);

                    // Encode to MP3
                    match encoder.encode(&buffer) {
                        Ok(mp3_data) => {
                            // Get the length before sending
                            let data_len = mp3_data.len();

                            // Send to streamer
                            if let Err(e) = streamer.send(mp3_data) {
                                error!("Failed to send MP3 data: {}", e);
                            } else {
                                total_bytes_sent += data_len;
                            }
                        }
                        Err(e) => {
                            error!("Failed to encode audio: {}", e);
                        }
                    }

                    // Log stats every 10 seconds
                    let now = Instant::now();
                    if now.duration_since(last_stats) >= Duration::from_secs(10) {
                        info!(
                            "Stats: loudness={:.1} dB, bytes_sent={} bytes",
                            max_loudness, total_bytes_sent
                        );
                        last_stats = now;
                        max_loudness = -60.0;
                    }
                }
                Err(crossbeam_channel::RecvTimeoutError::Timeout) => {
                    // Timeout, just continue
                }
                Err(crossbeam_channel::RecvTimeoutError::Disconnected) => {
                    // Channel closed, exit the loop
                    break;
                }
            }
        }

        // Flush the encoder
        if let Ok(flush_data) = encoder.flush() {
            if !flush_data.is_empty() {
                let _ = streamer.send(flush_data);
            }
        }

        // Stop the streamer
        rt.block_on(async {
            let _ = streamer.stop().await;
        });
    });

    // Wait for Ctrl+C
    ctrlc::set_handler(move || {
        info!("Received Ctrl+C, shutting down...");
        running.store(false, Ordering::SeqCst);
    })
    .map_err(|e| MuxError::Internal(format!("Failed to set Ctrl+C handler: {}", e)))?;

    // Wait for the processor thread to finish
    processor_thread
        .join()
        .map_err(|_| MuxError::Internal("Failed to join processor thread".to_string()))?;

    // Stop the audio input
    audio_input.stop()?;

    info!("Shutdown complete");
    Ok(())
}
