use clap::{Parser, Subcommand};
use mux_core::Config;
use std::path::PathBuf;

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
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Validate { config_file } => {
            println!("Validating configuration file: {}", config_file.display());

            match Config::load(config_file) {
                Ok(_) => {
                    println!("Configuration is valid!");
                    std::process::exit(0);
                }
                Err(err) => {
                    eprintln!("Error: {}", err);
                    std::process::exit(1);
                }
            }
        }
        Commands::Version => {
            println!("sonos-mux CLI v{}", mux_core::version());
            println!("Core library v{}", mux_core::version());
        }
    }
}
