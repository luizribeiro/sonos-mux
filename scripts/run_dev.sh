#!/bin/bash
set -e

echo "Setting up development environment for sonos-mux..."

# Check if we're running as root
if [ "$EUID" -ne 0 ]; then
  echo "Please run as root to load the ALSA loopback module"
  exit 1
fi

# Load ALSA loopback module if not already loaded
if ! lsmod | grep -q "^snd_aloop"; then
  echo "Loading ALSA loopback module..."
  modprobe snd-aloop
  echo "ALSA loopback module loaded."
else
  echo "ALSA loopback module already loaded."
fi

# Make the script executable
chmod +x "$0"

# Drop privileges to run the actual daemon
if [ "$SUDO_USER" ]; then
  echo "Dropping privileges to run muxd..."
  exec sudo -u "$SUDO_USER" bash -c "cd $(pwd) && RUST_LOG=info cargo run --bin muxd -- --config examples/config.toml"
else
  echo "Running muxd..."
  RUST_LOG=info cargo run --bin muxd -- --config examples/config.toml
fi