# Sonos‑Mux Project Charter

> **Status:** Work‑in‑Progress (Sprints 1-3 completed, see `SPRINT-04.md` → `SPRINT‑06.md` for active tasks)

## 1  Problem Statement
Roon‑managed households with multiple Sonos zones suffer audible gaps when switching tracks or when Home Assistant tries to inject TTS/alert media. Gaps come from repeating the Sonos HTTP handshake and RAAT endpoint discovery. We need a **resident multiplexer** that:

* keeps a single, perpetual HTTP audio stream open to every Sonos room, even while silent,
* mixes multiple PCM sources (Roon, HA TTS, radio) with gains & ducking,
* can be hosted on any Linux box (Pi, NUC, NAS) separate from Roon Core,
* is fully declarative, hot‑reloadable, and observable.

## 2  Goals & Non‑Goals
| | In scope | Out of scope |
|---|---|---|
| **G1** | Latency ≤ 3 s (dominated by Sonos buffer) | Bit‑perfect audiophile DSP |
| **G2** | 24×7 operation < 2 % CPU on Raspberry Pi 4 | Native Dolby Digital passthrough |
| **G3** | Hot‑reload config with zero audio dropout | Multi‑tenant / permission model |
| **G4** | CLI to scan and generate starter config | Mobile app GUI |

## 3  Functional Requirements
### 3.1  Inputs  
Supported `kind` values and options:

| kind | Options | Notes |
|------|---------|-------|
| `alsa` | `device`, optional `format` | Captured via `cpal` |
| `file` | `path`, `loop`, `on_demand` | Decoded by **Symphonia** |
| `http` | `url`, `reconnect_sec`, `icy_metadata` | MP3/AAC remote streams |
| `command` | `cmd`, optional `format` | FFmpeg/YT‑DL to stdout |
| `fifo` | `path`, `wildcard` | Plays & deletes new files |
| `silence` | `level_db` | Digital silence for keep‑alive |

_All inputs are resampled to 44 100 Hz stereo S16LE._

### 3.2  Outputs  
Only `kind = "sonos"` shipped in `v1`; future: `file`, `null`.

### 3.3  Routing
```toml
[[routes]]
input     = "roon_main"
outputs   = ["living_room", "kitchen", "office"]
gain_db   = 0

[[routes]]
input     = "ha_alerts"
outputs   = ["living_room", "kitchen"]
gain_db   = +3
duck_db   = -15        # attenuate all *other* active routes
priority  = 10         # higher wins ties
```

### 3.4  Hot‑Reload
Daemon accepts SIGHUP **or** `admin reload <file>` via Unix‑socket; validates and swaps routing tables gap‑free.

### 3.5  Observability
* `/healthz` JSON status per room  
* `/metrics` Prometheus counters (frames, bytes, underruns, CPU %)  

## 4  Architecture
```
Roon Core ──(RAAT)──► Roon Bridge (same host as mux)
                            │ (ALSA loopback)
                            ▼
                    +-----------------+
                    |  sonos‑muxd     |
                    |  1. capture     |
                    |  2. mix/duck    |
                    |  3. encode MP3  |
                    |  4. serve HTTP  |
                    +-----------------+
                            │
             ┌──────────────┴──────────────┐
             ▼                             ▼
         Sonos Room 1                 Sonos Room N
   (AVTransport URI = /stream.mp3)  … keeps playing forever
```

## 5  Dependency Matrix

| Area | Crates | Notes |
|------|--------|-------|
| Audio capture | `cpal`, `alsa` | cross‑platform |
| Decode / DSP | `symphonia`, `rubato` | resampler |
| Mix / Gain | custom + `dasp_sample` | |
| Encode | `lame-sys` (*MP3*), optional `twolame` | |
| HTTP / WebSocket | `hyper`, `warp` | |
| Sonos control | `sonor` | SSDP + SOAP |
| CLI / Config | `clap`, `serde`, `toml_edit` | |
| Observability | `prometheus`, `tracing` | |
| Tests | `assert_cmd`, `rstest`, `criterion` | |

_Minimum Rust 2021, MSRV 1.70._

## 6  Roadmap & Sprints
| Sprint | Theme | Key Deliverable |
|--------|-------|-----------------|
| 01 ✅ | Scaffold & config | Repo + CI + validator |
| 02 ✅ | E2E audio prototype | hear music through Sonos |
| 03 ✅ | Mixer + routing | multi‑source with ducking |
| 04 | Sonos resilience | auto‑set URI & regroup |
| 05 | CLI scan/apply | hot‑reload, admin socket |
| 06 | API, metrics, release | v1.0, systemd, Docker |

_Detailed task lists: see `SPRINT-0X.md`._

## 7  Risks & Mitigations
| Risk | Mitigation |
|------|------------|
| Roon changes licencing of Bridge | Vend own RAAT endpoint later (long‑shot) |
| Sonos firmware rejects long MP3 | Fallback to Ogg/Opus or HLS chunker |
| CPU spikes on Pi | Runtime bench & DSP SIMD gating |

## 8  Glossary
* **RAAT** – Roon Advanced Audio Transport.  
* **TTS** – Text‑to‑Speech; HA uses Polly/Google.  
* **Duck** – Reduce competing audio so alert pops.

---

For implementation details of each sprint refer to `SPRINT-01.md…SPRINT-06.md`.