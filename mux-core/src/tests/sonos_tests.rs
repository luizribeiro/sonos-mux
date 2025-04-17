use crate::output::sonos::{SonosManager, SonosOutput};
use crate::output::AudioOutput;

#[tokio::test]
async fn test_sonos_output_creation() {
    let output = SonosOutput::new("Test Room".to_string(), Some(5));
    assert_eq!(output.room(), "Test Room");
    assert_eq!(output.buffer_sec(), 5);
    assert!(output.ip_address().is_none());
}

#[tokio::test]
async fn test_sonos_manager() {
    let mut manager = SonosManager::new();
    manager.add_room("Living Room".to_string(), Some(5));
    manager.add_room("Kitchen".to_string(), None);

    let status = manager.health_status().await;
    assert_eq!(status.len(), 2);
    assert!(status.iter().any(|s| s.room == "Living Room"));
    assert!(status.iter().any(|s| s.room == "Kitchen"));

    // All rooms should start as unhealthy
    for s in &status {
        assert!(!s.healthy);
    }
}

#[tokio::test]
async fn test_sonos_keep_alive() {
    let mut output = SonosOutput::new("Test Room".to_string(), Some(5));

    // Initialize should succeed and set the output as healthy
    let result = output.initialize().await;
    assert!(result.is_ok());
    assert!(output.health_check().await);

    // Keep-alive should succeed
    let result = output.keep_alive().await;
    assert!(result.is_ok());
}

// Mock UPnP service test
struct MockSonosService {
    room_name: String,
    transport_uri: Option<String>,
}

impl MockSonosService {
    fn new(room_name: &str) -> Self {
        Self {
            room_name: room_name.to_string(),
            transport_uri: None,
        }
    }

    fn get_room_name(&self) -> &str {
        &self.room_name
    }

    fn set_transport_uri(&mut self, uri: &str) -> Result<(), &'static str> {
        self.transport_uri = Some(uri.to_string());
        Ok(())
    }

    fn get_transport_uri(&self) -> Option<&str> {
        self.transport_uri.as_deref()
    }
}

#[tokio::test]
async fn test_with_mock_service() {
    // Create a mock Sonos service
    let mut mock_service = MockSonosService::new("Mock Room");

    // Test setting a transport URI
    mock_service
        .set_transport_uri("http://example.com/stream.mp3")
        .unwrap();
    assert_eq!(
        mock_service.get_transport_uri(),
        Some("http://example.com/stream.mp3")
    );

    // Verify the room name
    assert_eq!(mock_service.get_room_name(), "Mock Room");
}
