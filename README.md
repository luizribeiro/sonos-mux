<h1 align="center">sonos‑mux</h1>
<p align="center"><em>Zero‑gap audio multiplexer for Roon → Sonos households</em></p>

[![build](https://github.com/yourorg/sonos-mux/actions/workflows/ci.yml/badge.svg)](https://github.com/yourorg/sonos-mux/actions)
[![license](https://img.shields.io/badge/license-Apache--2.0-blue)](LICENSE)

> **Note:** This README describes the <strong>vision</strong> for v1.0. Implementation is in‑flight – see <code>PROJECT.md</code> and sprint files for status.

---

## ✨ Features
* Keeps a **permanent** MP3 stream open to every Sonos room – no handshake delays
* Mixes unlimited PCM sources with per‑route gain / ducking
* Hot‑reloads `config.toml` without audible drop
* Auto‑discovers Sonos players and Roon outputs (`sonos‑mux scan`)
* Prometheus metrics & health endpoint
* Static binaries for x86‑64 / arm64 (Pi 4)

## 🚀 Quick Start
```bash
# 1. prerequisites
sudo apt install alsa-utils lame
sudo modprobe snd-aloop pcm_substreams=2   # once per boot

# 2. build & run
git clone https://github.com/yourorg/sonos-mux.git
cd sonos-mux
sudo ./scripts/run_dev.sh
```
Add `http://<mux-host>:8000/stream.mp3` as a custom radio station in the Sonos app  
→ music should play; change tracks in Roon, zero gaps 😊

## 🎮 Quick Demo
1. In one terminal, run the development script:
   ```bash
   sudo ./scripts/run_dev.sh
   ```

2. In another terminal, play audio through the ALSA loopback device:
   ```bash
   # Install sox if not already installed
   sudo apt install sox

   # Generate a test tone and play it through the loopback device
   play -n synth 60 sine 440 gain -6 remix 1 2 silence 1 5 1% @0:10 1 5 1%
   ```

3. Open `http://localhost:8000/stream.mp3` in your browser or media player to hear the audio.

4. Add this URL as a custom radio station in your Sonos app to stream to your Sonos speakers.

## 🛠️ Configuration
```toml
# inputs
[[inputs]]
id     = "roon_main"
kind   = "alsa"
device = "hw:Loopback,1"

[[inputs]]
id   = "ha_alerts"
kind = "fifo"
path = "/var/spool/ha_tts"

# outputs
[[outputs]]
id   = "living_room"
kind = "sonos"
room = "Living Room"

# routing
[[routes]]
input   = "roon_main"
outputs = ["living_room"]
gain_db = 0

[[routes]]
input   = "ha_alerts"
outputs = ["living_room"]
gain_db = +3
duck_db = -15
```
*Full schema & kind matrix in [`PROJECT.md`](PROJECT.md).*

### Hot‑Reload
```bash
sonos-mux apply new_config.toml          # via CLI
sonos-mux scan | sonos-mux apply -       # scan and apply
kill -HUP $(pidof muxd)                  # via signal
```

### Admin Commands
The daemon listens for admin commands on a Unix socket (`/run/sonos-mux.sock`) and TCP port (8383):

```bash
# Apply a new configuration
echo "apply" | nc -U /run/sonos-mux.sock
cat new_config.toml | nc -U /run/sonos-mux.sock

# Get version information
echo "version" | nc 127.0.0.1 8383

# Get statistics
echo "stats" | nc 127.0.0.1 8383
```

## 📦 Installation Options
| Method | Command |
|--------|---------|
| **Cargo** | `cargo install sonos-mux --locked` |
| **Docker** | `docker run -d --net=host ghcr.io/yourorg/sonos-mux:latest` |
| **Nix** | `nix run github:yourorg/sonos-mux` |

## 🔌 Control API (v1.0)
* WebSocket `ws://localhost:8000/ws`
  ```json
  {"set_volume":{"room":"Kitchen","db":-5}}
  ```
* Metrics `http://localhost:8000/metrics`
* Health `http://localhost:8000/healthz`

## 🤝 Contributing
Please read [`CONTRIBUTING.md`](CONTRIBUTING.md). Good first issues are tagged **help‑wanted**.

## 📄 License
Apache‑2.0 © 2025 Your Org