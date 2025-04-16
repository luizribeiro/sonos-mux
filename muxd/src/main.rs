use clap::Parser;
use mux_core::Config;
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about = "Sonos audio multiplexer daemon")]
struct Args {
    /// Path to the configuration file
    #[arg(short, long)]
    config: PathBuf,
}

fn main() {
    let args = Args::parse();

    println!(
        "muxd v{} - Sonos audio multiplexer daemon",
        mux_core::version()
    );
    println!("Loading configuration from: {}", args.config.display());

    // Load and validate configuration
    match Config::load(&args.config) {
        Ok(config) => {
            println!("Configuration loaded successfully!");
            println!("Inputs: {}", config.inputs.len());
            println!("Outputs: {}", config.outputs.len());
            println!("Routes: {}", config.routes.len());

            // In a future sprint, we'll implement the actual audio processing here
            println!("This is a scaffold implementation. Audio processing will be implemented in a future sprint.");
        }
        Err(err) => {
            eprintln!("Error loading configuration: {}", err);
            std::process::exit(1);
        }
    }
}
