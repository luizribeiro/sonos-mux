use clap::Parser;
use crossbeam_channel::bounded;
use log::{error, info, warn};
use mux_core::{AudioBuffer, Config, HttpStreamer, Lame, MuxError, SonosManager};
use serde::Serialize;
use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;
use std::time::{Duration, Instant};
use tokio::runtime::Runtime;
use tokio::sync::oneshot;
use warp::Filter;

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

#[derive(Debug, Serialize, Clone)]
struct HealthResponse {
    status: String,
    version: String,
    uptime_sec: u64,
    outputs: Vec<OutputHealth>,
}

#[derive(Debug, Serialize, Clone)]
struct OutputHealth {
    id: String,
    kind: String,
    room: Option<String>,
    healthy: bool,
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

    // Create the Sonos manager
    let mut sonos_manager = SonosManager::new();

    // Add all Sonos outputs to the manager
    for output_config in &config.outputs {
        if output_config.kind == "sonos" {
            if let Some(room) = &output_config.room {
                info!("Adding Sonos room: {}", room);
                sonos_manager.add_room(room.clone(), output_config.buffer_sec);
            } else {
                warn!(
                    "Sonos output '{}' has no room name, skipping",
                    output_config.id
                );
            }
        }
    }

    // Wrap the manager in Arc<Mutex> for sharing
    let sonos_manager = Arc::new(tokio::sync::Mutex::new(sonos_manager));

    // Create tokio runtime
    let rt = Arc::new(
        Runtime::new()
            .map_err(|e| MuxError::Internal(format!("Failed to create runtime: {}", e)))?,
    );

    // Create clones for different threads
    let rt_health = rt.clone();
    let rt_processor = rt.clone();

    // Initialize Sonos outputs
    let sonos_manager_clone = sonos_manager.clone();
    rt_health.block_on(async {
        let manager = sonos_manager_clone.lock().await;
        if let Err(e) = manager.initialize_all().await {
            warn!("Some Sonos devices failed to initialize: {}", e);
        }
    });

    // Start the keep-alive task
    let keep_alive_tx = rt_health.block_on(async {
        let manager = sonos_manager.lock().await;
        manager.start_keep_alive_task()
    });

    // For Sprint 2+4, we'll implement a pipeline:
    // 1. Create an audio input from the first input in the config
    // 2. Create an MP3 encoder
    // 3. Create an HTTP streamer
    // 4. Set up Sonos to play the stream

    // For now, use the first input
    let input_config = &config.inputs[0];
    info!("Using input: {} ({})", input_config.id, input_config.kind);

    // Create the audio input
    let mut audio_input = mux_core::input::create_input(input_config)?;

    // Create the MP3 encoder
    let mut encoder = Lame::new(128)?; // 128 kbps

    // Create the HTTP streamer
    let http_port = 8000; // Default port
    let streamer = HttpStreamer::new(http_port);

    // Start the HTTP streamer
    rt_health.block_on(async {
        streamer.start().await.map_err(MuxError::Stream)?;
        Ok::<_, MuxError>(())
    })?;

    info!("HTTP streamer started on port {}", http_port);
    info!(
        "Stream available at http://localhost:{}/stream.mp3",
        http_port
    );

    // Get the stream URL for Sonos
    let hostname = hostname::get()
        .map(|h| h.to_string_lossy().to_string())
        .unwrap_or_else(|_| "localhost".to_string());

    let stream_url = format!("http://{}:{}/stream.mp3", hostname, http_port);
    info!("Full stream URL: {}", stream_url);

    // Set the stream URL on all Sonos outputs
    rt_health.block_on(async {
        let manager = sonos_manager.lock().await;
        for output_config in &config.outputs {
            if output_config.kind == "sonos" {
                if let Some(room) = &output_config.room {
                    info!("Setting stream for room '{}' to {}", room, stream_url);
                    if let Err(e) = manager.set_stream(room, &stream_url).await {
                        warn!("Failed to set stream for room '{}': {}", room, e);
                    }
                }
            }
        }
    });

    // Create channels for the pipeline
    let (audio_sender, audio_receiver) = bounded::<AudioBuffer>(10);

    // Flag to signal shutdown
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();

    // Start the audio input
    audio_input.start(audio_sender)?;

    // Start time for uptime tracking
    let start_time = Instant::now();

    // Create a snapshot of outputs for health check
    let config_outputs = config.outputs.clone();

    // Set up health check endpoint
    let sonos_manager_health = sonos_manager.clone();

    let health_check = warp::path("healthz").and_then(move || {
        let uptime_sec = start_time.elapsed().as_secs();
        let manager_clone = sonos_manager_health.clone();
        let outputs_clone = config_outputs.clone();

        async move {
            let manager = manager_clone.lock().await;
            let sonos_status = manager.health_status().await;

            let mut outputs = Vec::new();
            for output_config in &outputs_clone {
                let healthy = match output_config.kind.as_str() {
                    "sonos" => {
                        if let Some(room) = &output_config.room {
                            sonos_status.iter().any(|s| &s.room == room && s.healthy)
                        } else {
                            false
                        }
                    }
                    _ => true, // Other output types (like HTTP) are assumed healthy
                };

                outputs.push(OutputHealth {
                    id: output_config.id.clone(),
                    kind: output_config.kind.clone(),
                    room: output_config.room.clone(),
                    healthy,
                });
            }

            let all_healthy = outputs.iter().all(|o| o.healthy);

            let response = HealthResponse {
                status: if all_healthy {
                    "ok".to_string()
                } else {
                    "degraded".to_string()
                },
                version: mux_core::version().to_string(),
                uptime_sec,
                outputs,
            };

            Ok::<_, warp::Rejection>(warp::reply::json(&response))
        }
    });

    // Start the health check server
    let (health_tx, health_rx) = oneshot::channel();
    // Use Arc to share the sender between threads
    let health_tx = Arc::new(tokio::sync::Mutex::new(Some(health_tx)));

    let health_server = async move {
        let routes = health_check.with(warp::cors().allow_any_origin());
        let (addr, server) =
            warp::serve(routes).bind_with_graceful_shutdown(([0, 0, 0, 0], 8080), async move {
                health_rx.await.ok();
            });

        info!(
            "Health check endpoint available at http://{}:{}/healthz",
            hostname,
            addr.port()
        );
        server.await;
    };

    rt_health.spawn(health_server);

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
        rt_processor.block_on(async {
            let _ = streamer.stop().await;
        });
    });

    // Clone for the Ctrl+C handler
    let health_tx_clone = health_tx.clone();

    // Wait for Ctrl+C
    ctrlc::set_handler(move || {
        info!("Received Ctrl+C, shutting down...");
        running.store(false, Ordering::SeqCst);

        // Stop the keep-alive task
        drop(keep_alive_tx.send(()));

        // Stop the health server - take the sender out of the Option
        let _ = rt_health.block_on(async {
            if let Some(tx) = health_tx_clone.lock().await.take() {
                tx.send(())
            } else {
                Ok(())
            }
        });
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
