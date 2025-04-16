<h1 align="center">sonos‑mux</h1>
<p align="center"><em>Zero‑gap audio multiplexer for Roon → Sonos households</em></p>

[![build](https://github.com/yourorg/sonos-mux/actions/workflows/ci.yml/badge.svg)](…)
[![license](https://img.shields.io/badge/license-Apache--2.0-blue)](LICENSE)

> **Note:** This README describes the <strong>vision</strong> for v1.0. Implementation is in‑flight – see <code>PROJECT.md</code> and sprint files for status.

---

## ✨ Features
* Keeps a **permanent** MP3 stream open to every Sonos room – no handshake delays
* Mixes unlimited PCM sources with per‑route gain / ducking
* Hot‑reloads `config.toml` without audible drop
* Auto‑discovers Sonos players and Roon outputs (`sonos‑mux scan`)
* Prometheus metrics & health endpoint
* Static binaries for x86‑64 / arm64 (Pi 4)

## 🚀 Quick Start
```bash
# 1. prerequisites
sudo apt install alsa-utils lame
sudo modprobe snd-aloop pcm_substreams=2   # once per boot

# 2. build & run
git clone https://github.com/yourorg/sonos-mux.git
cd sonos-mux
cargo run --bin muxd -- --config examples/config.toml
```
Add `http://<mux-host>:8000/stream.mp3` as a custom radio station in the Sonos app  
→ music should play; change tracks in Roon, zero gaps 😊

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
# or
kill -HUP $(pidof muxd)                  # via signal
```

## 📦 Installation Options
| Method | Command |
|--------|---------|
| **Cargo** | `cargo install sonos-mux --locked` |
| **Docker** | `docker run -d --net=host ghcr.io/yourorg/sonos-mux:latest` |
| **Nix** | `nix run github:yourorg/sonos-mux` |

## 🔌 Control API (v1.0)
* WebSocket `ws://localhost:8000/ws`
  ```json
  {"set_volume":{"room":"Kitchen","db":-5}}
  ```
* Metrics `http://localhost:8000/metrics`
* Health `http://localhost:8000/healthz`

## 🤝 Contributing
Please read [`CONTRIBUTING.md`](CONTRIBUTING.md). Good first issues are tagged **help‑wanted**.

## 📄 License
Apache‑2.0 © 2025 Your Org