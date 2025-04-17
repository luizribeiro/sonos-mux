use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use mux_core::Config;
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;
use std::process::exit;
use thiserror::Error;

mod admin;
mod scanner;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Validate a configuration file
    Validate {
        /// Path to the configuration file
        config_file: PathBuf,
    },

    /// Display version information
    Version,

    /// Scan for available Sonos devices
    Scan {
        /// Output format (toml or json)
        #[arg(short, long, default_value = "toml")]
        format: String,
    },

    /// Apply a new configuration to a running daemon
    Apply {
        /// Path to the configuration file (use "-" for stdin)
        config_file: String,

        /// Unix socket path
        #[arg(short, long)]
        socket: Option<String>,

        /// TCP port for admin commands
        #[arg(short, long, default_value = "8383")]
        port: u16,

        /// Host for TCP connection
        #[arg(short, long, default_value = "127.0.0.1")]
        host: String,
    },
}

#[derive(Error, Debug)]
enum CliError {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("Config error: {0}")]
    Config(#[from] mux_core::ConfigError),
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Validate { config_file } => {
            println!("Validating configuration file: {}", config_file.display());

            match Config::load(config_file) {
                Ok(_) => {
                    println!("Configuration is valid!");
                    exit(0);
                }
                Err(err) => {
                    eprintln!("Error: {}", err);
                    exit(1);
                }
            }
        }
        Commands::Version => {
            println!("sonos-mux CLI v{}", mux_core::version());
            println!("Core library v{}", mux_core::version());
        }
        Commands::Scan { format } => match scanner::scan().await {
            Ok(config) => match format.to_lowercase().as_str() {
                "toml" => {
                    let toml_str = toml::to_string_pretty(&config)
                        .context("Failed to serialize config to TOML")?;
                    println!("{}", toml_str);
                }
                "json" => {
                    let json_str = serde_json::to_string_pretty(&config)
                        .context("Failed to serialize config to JSON")?;
                    println!("{}", json_str);
                }
                _ => {
                    eprintln!("Unknown format: {}. Using TOML.", format);
                    let toml_str = toml::to_string_pretty(&config)
                        .context("Failed to serialize config to TOML")?;
                    println!("{}", toml_str);
                }
            },
            Err(err) => {
                eprintln!("Error scanning for devices: {}", err);
                exit(1);
            }
        },
        Commands::Apply {
            config_file,
            socket,
            port,
            host,
        } => {
            // Read the configuration
            let config_content = if config_file == "-" {
                let mut buffer = String::new();
                io::stdin().read_to_string(&mut buffer)?;
                buffer
            } else {
                fs::read_to_string(config_file)?
            };

            // Validate the configuration
            match Config::from_reader(config_content.as_bytes()) {
                Ok(_) => {
                    // Configuration is valid, now send it to the daemon
                    let result = if let Some(socket_path) = socket {
                        admin::send_to_unix_socket(socket_path, "apply", &config_content).await
                    } else {
                        admin::send_to_tcp(host, *port, "apply", &config_content).await
                    };

                    match result {
                        Ok(response) => {
                            println!("{}", response);
                            if !response.contains("\"success\":true") {
                                exit(1);
                            }
                        }
                        Err(err) => {
                            eprintln!("Error sending configuration to daemon: {}", err);
                            exit(1);
                        }
                    }
                }
                Err(err) => {
                    eprintln!("Invalid configuration: {}", err);
                    exit(1);
                }
            }
        }
    }

    Ok(())
}
