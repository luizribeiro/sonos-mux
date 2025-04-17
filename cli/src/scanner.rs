use anyhow::Result;
use mux_core::{Config, Input, Output, Route};
use std::time::Duration;
use tokio::time::sleep;

pub async fn scan() -> Result<Config> {
    println!("Scanning for Sonos devices...");

    // In a real implementation, we would use sonor to discover Sonos devices
    // For now, we'll create a sample configuration with mock data
    let mut config = Config {
        inputs: vec![],
        outputs: vec![],
        routes: vec![],
        logging: None,
    };

    // Simulate a scan delay
    sleep(Duration::from_millis(500)).await;

    // Add a default silence input
    config.inputs.push(Input {
        id: "silence".to_string(),
        kind: "silence".to_string(),
        device: None,
        path: None,
        loop_playback: None,
        url: None,
    });

    // Add a default ALSA input (for Roon)
    config.inputs.push(Input {
        id: "roon_main".to_string(),
        kind: "alsa".to_string(),
        device: Some("hw:Loopback,1".to_string()),
        path: None,
        loop_playback: None,
        url: None,
    });

    // Add a default HTTP input (for streaming)
    config.inputs.push(Input {
        id: "web_radio".to_string(),
        kind: "http".to_string(),
        device: None,
        path: None,
        loop_playback: None,
        url: Some("http://example.com/stream".to_string()),
    });

    // Add a default file input
    config.inputs.push(Input {
        id: "alert_sound".to_string(),
        kind: "file".to_string(),
        device: None,
        path: Some("/path/to/alert.mp3".to_string()),
        loop_playback: Some(false),
        url: None,
    });

    // Simulate finding Sonos devices
    let discovered_rooms = vec!["Living Room", "Kitchen", "Bedroom", "Office"];

    for room in discovered_rooms {
        config.outputs.push(Output {
            id: room.to_lowercase().replace(' ', "_"),
            kind: "sonos".to_string(),
            room: Some(room.to_string()),
            buffer_sec: Some(5),
            host: None,
            port: None,
        });
    }

    // Create default routes
    for output in &config.outputs {
        config.routes.push(Route {
            input: "silence".to_string(),
            outputs: vec![output.id.clone()],
            gain_db: 0.0,
            duck_db: 0.0,
        });
    }

    // Add a sample route for Roon to all outputs
    config.routes.push(Route {
        input: "roon_main".to_string(),
        outputs: config.outputs.iter().map(|o| o.id.clone()).collect(),
        gain_db: 0.0,
        duck_db: 0.0,
    });

    Ok(config)
}
