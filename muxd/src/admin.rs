use log::{error, info};
use mux_core::{Config, SonosManager};
use serde::{Deserialize, Serialize};
use std::io;
use std::path::Path;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, UnixListener};
use tokio::sync::Mutex;

#[derive(Debug, Serialize, Deserialize)]
pub struct AdminResponse {
    pub success: bool,
    pub message: String,
}

pub struct AdminServer {
    config_path: Option<String>,
    sonos_manager: Arc<Mutex<SonosManager>>,
    reload_trigger: Arc<Mutex<Option<tokio::sync::mpsc::Sender<Config>>>>,
}

impl AdminServer {
    pub fn new(
        config_path: Option<String>,
        sonos_manager: Arc<Mutex<SonosManager>>,
        reload_trigger: Arc<Mutex<Option<tokio::sync::mpsc::Sender<Config>>>>,
    ) -> Self {
        Self {
            config_path,
            sonos_manager,
            reload_trigger,
        }
    }

    pub async fn start_unix(&self, socket_path: &str) -> io::Result<()> {
        // Remove the socket file if it exists
        if Path::new(socket_path).exists() {
            tokio::fs::remove_file(socket_path).await?;
        }

        let listener = UnixListener::bind(socket_path)?;
        info!("Admin server started on Unix socket: {}", socket_path);

        loop {
            match listener.accept().await {
                Ok((stream, _addr)) => {
                    let server = self.clone();
                    tokio::spawn(async move {
                        if let Err(e) = server.handle_connection(stream).await {
                            error!("Error handling admin connection: {}", e);
                        }
                    });
                }
                Err(e) => {
                    error!("Error accepting connection: {}", e);
                }
            }
        }
    }

    pub async fn start_tcp(&self, port: u16) -> io::Result<()> {
        let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).await?;
        info!("Admin server started on TCP port: {}", port);

        loop {
            match listener.accept().await {
                Ok((stream, _addr)) => {
                    let server = self.clone();
                    tokio::spawn(async move {
                        if let Err(e) = server.handle_connection(stream).await {
                            error!("Error handling admin connection: {}", e);
                        }
                    });
                }
                Err(e) => {
                    error!("Error accepting connection: {}", e);
                }
            }
        }
    }

    async fn handle_connection<T>(&self, stream: T) -> io::Result<()>
    where
        T: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
    {
        let mut reader = BufReader::new(stream);
        let mut line = String::new();

        // Read the command
        reader.read_line(&mut line).await?;

        let line = line.trim();
        let parts: Vec<&str> = line.split_whitespace().collect();

        if parts.is_empty() {
            return Ok(());
        }

        let response = match parts[0] {
            "version" => AdminResponse {
                success: true,
                message: format!("sonos-mux v{}", mux_core::version()),
            },
            "reload" => {
                if parts.len() < 2 && self.config_path.is_none() {
                    AdminResponse {
                        success: false,
                        message: "No config file specified".to_string(),
                    }
                } else {
                    let config_path = if parts.len() >= 2 {
                        parts[1]
                    } else {
                        self.config_path.as_ref().unwrap()
                    };

                    match Config::load(config_path) {
                        Ok(config) => {
                            let reload_trigger = self.reload_trigger.lock().await;
                            if let Some(trigger) = &*reload_trigger {
                                if let Err(e) = trigger.send(config).await {
                                    AdminResponse {
                                        success: false,
                                        message: format!("Failed to send reload: {}", e),
                                    }
                                } else {
                                    AdminResponse {
                                        success: true,
                                        message: "Configuration reloaded successfully".to_string(),
                                    }
                                }
                            } else {
                                AdminResponse {
                                    success: false,
                                    message: "Reload trigger not available".to_string(),
                                }
                            }
                        }
                        Err(e) => AdminResponse {
                            success: false,
                            message: format!("Failed to load config: {}", e),
                        },
                    }
                }
            }
            "apply" => {
                // Read the config content from the connection
                let mut config_content = String::new();
                let mut content_line = String::new();

                // Read until we get a non-empty line (configuration content)
                loop {
                    match reader.read_line(&mut content_line).await {
                        Ok(bytes) if bytes > 0 => {
                            // Skip empty lines
                            if content_line.trim().is_empty() {
                                content_line.clear();
                                continue;
                            }

                            // Start accumulating config content
                            config_content.push_str(&content_line);
                            content_line.clear();
                            break;
                        }
                        Ok(_) | Err(_) => {
                            // End of input or error
                            break;
                        }
                    }
                }

                // Continue reading the rest of the content
                loop {
                    match reader.read_line(&mut content_line).await {
                        Ok(bytes) if bytes > 0 => {
                            // Empty line marks the end of the config
                            if content_line.trim().is_empty() {
                                break;
                            }

                            config_content.push_str(&content_line);
                            content_line.clear();
                        }
                        Ok(_) | Err(_) => {
                            // End of input or error
                            break;
                        }
                    }
                }

                info!(
                    "Received config content of length: {}",
                    config_content.len()
                );

                if config_content.is_empty() {
                    AdminResponse {
                        success: false,
                        message: "No configuration provided".to_string(),
                    }
                } else {
                    match Config::from_reader(config_content.as_bytes()) {
                        Ok(config) => {
                            let reload_trigger = self.reload_trigger.lock().await;
                            if let Some(trigger) = &*reload_trigger {
                                if let Err(e) = trigger.send(config).await {
                                    AdminResponse {
                                        success: false,
                                        message: format!("Failed to send reload: {}", e),
                                    }
                                } else {
                                    AdminResponse {
                                        success: true,
                                        message: "Configuration applied successfully".to_string(),
                                    }
                                }
                            } else {
                                AdminResponse {
                                    success: false,
                                    message: "Reload trigger not available".to_string(),
                                }
                            }
                        }
                        Err(e) => AdminResponse {
                            success: false,
                            message: format!("Failed to parse config: {}", e),
                        },
                    }
                }
            }
            "stats" => {
                let manager = self.sonos_manager.lock().await;
                let status = manager.health_status().await;

                AdminResponse {
                    success: true,
                    message: serde_json::to_string_pretty(&status)
                        .unwrap_or_else(|_| "{}".to_string()),
                }
            }
            _ => AdminResponse {
                success: false,
                message: format!("Unknown command: {}", parts[0]),
            },
        };

        // Serialize and send the response
        let response_json = serde_json::to_string(&response)?;
        let mut writer = reader.into_inner();
        writer.write_all(response_json.as_bytes()).await?;
        writer.flush().await?;

        Ok(())
    }
}

impl Clone for AdminServer {
    fn clone(&self) -> Self {
        Self {
            config_path: self.config_path.clone(),
            sonos_manager: self.sonos_manager.clone(),
            reload_trigger: self.reload_trigger.clone(),
        }
    }
}
