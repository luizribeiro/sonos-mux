# Adding a New Sonos Room

This guide explains how to add a new Sonos room to your sonos-mux configuration.

## Finding Available Rooms

First, use the CLI to scan for available Sonos devices on your network:

```bash
cargo run --bin cli scan
```

This will display a list of all Sonos rooms found on your network with their room names and IP addresses.

## Updating Your Configuration

Add the new room to your `config.toml` file in the `outputs` section:

```toml
[[outputs]]
id = "kitchen"                # A unique identifier for this output
kind = "sonos"                # Must be "sonos" for Sonos speakers
room = "Kitchen"              # The exact room name as shown in the scan
buffer_sec = 3                # Optional: buffer size in seconds (default: 3)
```

Note that the `room` value must exactly match the room name as reported by the Sonos device.

## Routing Audio to the New Room

To route audio to your new room, add its ID to a route in your configuration:

```toml
[[routes]]
input = "main_input"           # The input ID to route from
outputs = ["living_room", "kitchen"]  # Include your new room here
gain_db = 0.0                  # Optional: gain in dB
```

## Grouping Rooms

Sonos rooms can be grouped together to play the same audio in sync. Sonos-mux will automatically maintain these groups.

To group rooms, simply include them in the same route. The first room in the list will be the group coordinator.

## Verifying the Configuration

After updating your configuration, verify it with:

```bash
cargo run --bin cli validate config.toml
```

## Applying the Configuration

If you're updating a running daemon, you can hot-reload the configuration:

```bash
cargo run --bin cli apply config.toml
```

Or restart the daemon with the new configuration:

```bash
cargo run --bin muxd -- --config config.toml
```

## Troubleshooting

If you encounter issues with a new room:

1. Check the health endpoint: http://localhost:8080/healthz
2. Verify the room name exactly matches what's shown in the scan
3. Ensure the Sonos device is on the same network and reachable
4. Check the daemon logs for any specific errors related to the room

The daemon will automatically attempt to reconnect to any rooms that become unavailable, so temporary network issues or power interruptions should resolve automatically.