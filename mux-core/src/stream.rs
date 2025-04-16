use thiserror::Error;

#[derive(Debug, Error)]
pub enum StreamError {
    #[error("Failed to start HTTP server: {0}")]
    ServerStart(String),

    #[error("Failed to send data: {0}")]
    Send(String),
}

// For Sprint 2, we'll create a mock implementation of the HTTP streamer
// This allows us to test the pipeline without actually starting an HTTP server
pub struct HttpStreamer {
    port: u16,
    bytes_sent: std::sync::atomic::AtomicUsize,
}

impl HttpStreamer {
    pub fn new(port: u16) -> Self {
        HttpStreamer {
            port,
            bytes_sent: std::sync::atomic::AtomicUsize::new(0),
        }
    }

    pub async fn start(&self) -> Result<(), StreamError> {
        // In a real implementation, this would start an HTTP server
        println!("Mock HTTP streamer started on port {}", self.port);
        println!(
            "In a real implementation, stream would be available at http://localhost:{}/stream.mp3",
            self.port
        );
        Ok(())
    }

    pub fn send(&self, data: Vec<u8>) -> Result<(), StreamError> {
        // In a real implementation, this would send data to connected clients
        // For now, we'll just track how many bytes we've "sent"
        let bytes_sent = self
            .bytes_sent
            .fetch_add(data.len(), std::sync::atomic::Ordering::SeqCst);
        println!(
            "Mock HTTP streamer: sent {} bytes, total: {}",
            data.len(),
            bytes_sent + data.len()
        );
        Ok(())
    }

    pub async fn stop(&self) -> Result<(), StreamError> {
        // In a real implementation, this would stop the HTTP server
        println!("Mock HTTP streamer stopped");
        Ok(())
    }

    pub fn bytes_sent(&self) -> usize {
        self.bytes_sent.load(std::sync::atomic::Ordering::SeqCst)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_http_streamer() {
        // Create and start the streamer
        let streamer = HttpStreamer::new(8000);
        streamer.start().await.unwrap();

        // Generate some test MP3 data
        let test_data = vec![
            0xFF, 0xE0, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, // MP3 frame header
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // Frame data
        ];

        // Send the data
        streamer.send(test_data.clone()).unwrap();

        // Check that we've "sent" the right number of bytes
        assert_eq!(streamer.bytes_sent(), test_data.len());

        // Stop the streamer
        streamer.stop().await.unwrap();
    }
}
