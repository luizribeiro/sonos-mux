use crate::output::{AudioOutput, OutputError};
use async_trait::async_trait;
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, Mutex};
use tokio::time;

// We'll use sonor for Sonos discovery and control
// This is a placeholder until we add the dependency

/// Sonos speaker output
#[derive(Debug)]
pub struct SonosOutput {
    /// Room name (friendly name)
    room: String,
    /// IP address of the speaker
    ip_address: Option<String>,
    /// Stream URL to play
    stream_url: Option<String>,
    /// Buffer size in seconds
    buffer_sec: u32,
    /// Last successful connection time
    last_connection: Option<Instant>,
    /// Health status
    healthy: bool,
    /// Group coordinator (if any)
    #[allow(dead_code)]
    group_coordinator: Option<String>,
    /// Grouped with other rooms
    grouped_with: Vec<String>,
}

impl SonosOutput {
    /// Create a new Sonos output for a specific room
    pub fn new(room: String, buffer_sec: Option<u32>) -> Self {
        SonosOutput {
            room,
            ip_address: None,
            stream_url: None,
            buffer_sec: buffer_sec.unwrap_or(3),
            last_connection: None,
            healthy: false,
            group_coordinator: None,
            grouped_with: Vec::new(),
        }
    }

    /// Get the room name
    pub fn room(&self) -> &str {
        &self.room
    }

    /// Get the buffer size in seconds
    pub fn buffer_sec(&self) -> u32 {
        self.buffer_sec
    }

    /// Get the IP address if available
    pub fn ip_address(&self) -> Option<&str> {
        self.ip_address.as_deref()
    }

    /// Get the stream URL if available
    pub fn stream_url(&self) -> Option<&str> {
        self.stream_url.as_deref()
    }

    /// Get the grouped rooms
    pub fn grouped_with(&self) -> &[String] {
        &self.grouped_with
    }

    /// Discover the Sonos device by room name
    async fn discover_device(&mut self) -> Result<(), Box<dyn Error + Send + Sync>> {
        // This is a placeholder for the actual discovery logic using sonor
        info!("Discovering Sonos device for room: {}", self.room);

        // Simulate discovery - in real implementation, we would:
        // 1. Use SSDP to find all Sonos devices
        // 2. Query each device for its room name
        // 3. Match with our configured room name

        // For now, we'll just pretend we found it at a fake IP
        let ip = format!("192.168.1.{}", 100 + self.room.len() % 100);
        info!("Found device for room '{}' at IP: {}", self.room, ip);
        self.ip_address = Some(ip);
        self.healthy = true;

        Ok(())
    }

    /// Set up device grouping
    async fn setup_group(&mut self) -> Result<(), Box<dyn Error + Send + Sync>> {
        if self.grouped_with.is_empty() {
            debug!("No grouping configured for room '{}'", self.room);
            return Ok(());
        }

        info!(
            "Setting up group for '{}' with rooms: {:?}",
            self.room, self.grouped_with
        );

        // In a real implementation, we would use sonor to:
        // 1. Find all devices in the group
        // 2. Determine the coordinator
        // 3. Join them to the coordinator's group

        Ok(())
    }
}

#[async_trait]
impl AudioOutput for SonosOutput {
    async fn initialize(&mut self) -> Result<(), Box<dyn Error + Send + Sync>> {
        // Discover the device
        self.discover_device().await?;

        // Set up grouping if configured
        self.setup_group().await?;

        info!("Sonos output initialized for room: {}", self.room);
        self.last_connection = Some(Instant::now());
        self.healthy = true;

        Ok(())
    }

    async fn set_stream(&mut self, url: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
        if self.ip_address.is_none() {
            return Err(Box::new(OutputError::DeviceNotFound(self.room.clone())));
        }

        info!("Setting stream URL for room '{}' to: {}", self.room, url);

        // In a real implementation, we would:
        // 1. Use SOAP/UPnP to call SetAVTransportURI on the device
        // 2. Set the URL and metadata
        // 3. Call Play to start playback

        self.stream_url = Some(url.to_string());
        self.last_connection = Some(Instant::now());
        self.healthy = true;

        Ok(())
    }

    async fn keep_alive(&mut self) -> Result<(), Box<dyn Error + Send + Sync>> {
        // If we've never connected or don't have an IP, try discovery
        if self.ip_address.is_none() {
            warn!("No IP address for room '{}', rediscovering", self.room);
            self.discover_device().await?;
        }

        // If we have a stream URL but haven't connected recently, reconnect
        if let Some(last) = self.last_connection {
            if last.elapsed() > Duration::from_secs(60) {
                if let Some(url) = &self.stream_url.clone() {
                    info!("Reconnecting stream for room '{}'", self.room);
                    self.set_stream(url).await?;
                }
            }
        }

        // Verify grouping is still correct
        self.setup_group().await?;

        debug!("Keep-alive check completed for room '{}'", self.room);
        self.healthy = true;

        Ok(())
    }

    async fn health_check(&self) -> bool {
        self.healthy
    }
}

/// Health status for a Sonos room
#[derive(Debug, Serialize, Deserialize)]
pub struct SonosHealth {
    pub room: String,
    pub ip_address: Option<String>,
    pub healthy: bool,
    pub last_connection: Option<u64>, // timestamp
    pub grouped_with: Vec<String>,
}

/// Sonos device manager for handling multiple rooms
#[derive(Debug, Default)]
pub struct SonosManager {
    rooms: HashMap<String, Arc<Mutex<SonosOutput>>>,
}

impl SonosManager {
    /// Create a new Sonos manager
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a room to manage
    pub fn add_room(&mut self, room: String, buffer_sec: Option<u32>) {
        let output = SonosOutput::new(room.clone(), buffer_sec);
        self.rooms.insert(room, Arc::new(Mutex::new(output)));
    }

    /// Initialize all rooms
    pub async fn initialize_all(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        for (room, output) in &self.rooms {
            let mut output = output.lock().await;
            match output.initialize().await {
                Ok(_) => info!("Initialized room: {}", room),
                Err(e) => {
                    error!("Failed to initialize room {}: {}", room, e);
                    // Continue with other rooms even if one fails
                }
            }
        }
        Ok(())
    }

    /// Start the keep-alive task
    pub fn start_keep_alive_task(&self) -> mpsc::Sender<()> {
        let rooms = self.rooms.clone();
        let (tx, mut rx) = mpsc::channel::<()>(1);

        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_secs(60));

            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        debug!("Running keep-alive checks for all rooms");
                        for (room_name, room_arc) in &rooms {
                            let room_name = room_name.clone();
                            let room_arc = room_arc.clone();

                            // Process each room in its own task to avoid holding locks across awaits
                            tokio::spawn(async move {
                                // Lock the mutex and process the room
                                let mut output = room_arc.lock().await;
                                match output.keep_alive().await {
                                    Ok(_) => debug!("Keep-alive succeeded for room {}", room_name),
                                    Err(e) => error!("Keep-alive failed for room {}: {}", room_name, e),
                                }
                            });
                        }
                    }
                    _ = rx.recv() => {
                        debug!("Stopping keep-alive task");
                        break;
                    }
                }
            }
        });

        tx
    }

    /// Get health status for all rooms
    pub async fn health_status(&self) -> Vec<SonosHealth> {
        let mut status = Vec::new();

        for (room, output) in &self.rooms {
            let output = output.lock().await;
            status.push(SonosHealth {
                room: room.clone(),
                ip_address: output.ip_address.clone(),
                healthy: output.healthy,
                last_connection: output.last_connection.map(|t| t.elapsed().as_secs()),
                grouped_with: output.grouped_with.clone(),
            });
        }

        status
    }

    /// Set stream URL for a specific room
    pub async fn set_stream(
        &self,
        room: &str,
        url: &str,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let output = self
            .rooms
            .get(room)
            .ok_or_else(|| OutputError::DeviceNotFound(room.to_string()))?;

        let mut output = output.lock().await;
        output.set_stream(url).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sonos_output_creation() {
        let output = SonosOutput::new("Living Room".to_string(), Some(5));
        assert_eq!(output.buffer_sec(), 5);
        assert_eq!(output.room(), "Living Room");
        assert!(output.ip_address().is_none());
    }

    #[tokio::test]
    async fn test_sonos_manager() {
        let mut manager = SonosManager::new();
        manager.add_room("Living Room".to_string(), Some(5));
        manager.add_room("Kitchen".to_string(), None);

        assert_eq!(manager.rooms.len(), 2);
        assert!(manager.rooms.contains_key("Living Room"));
        assert!(manager.rooms.contains_key("Kitchen"));

        let status = manager.health_status().await;
        assert_eq!(status.len(), 2);
    }
}
