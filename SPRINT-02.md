# Sprint 02 – End‑to‑end Audio Prototype
**Duration:** 2 weeks  
**Sprint Goal:** hear Roon music through Sonos using a single hard‑wired path.

---

## Deliverables
1. **ALSA input module** (`mux-core/src/input/alsa.rs`)
   * Uses `cpal` + `alsa-sys` to capture S16LE 44.1 kHz frames.
2. **MP3 encoder** (`mux-core/src/encoder.rs`)
   * Wrapper around `lame-sys`; accepts interleaved stereo, returns Vec<u8>.
3. **HTTP streamer** (`mux-core/src/stream.rs`)
   * Hyper service at `/stream.mp3` doing chunked transfer.
4. **Daemon binary** (`muxd/src/main.rs`)
   * Reads config
   * Creates pipeline: `alsa -> encoder -> stream`
   * Logs every 10 s: loudness, bytes sent.
5. **Dev script** `scripts/run_dev.sh`:
   * Loads loopback (`modprobe snd-aloop`), launches `muxd`.
6. Integration test under `tests/`:
   * Spawns `muxd`, pulls 3 s from `/stream.mp3`, ensures non‑zero frames.

---

## Task breakdown
| # | Assignee | Task | Acceptance criteria |
|---|----------|------|---------------------|
| 1 | IC3‑A | Build `input::alsa` capturing to crossbeam channel | Unit‑test reads 1 000 frames |
| 2 | IC3‑B | Implement `encoder::Lame` with CBR 128k | Test encodes 1 s sine wave |
| 3 | IC3‑C | Implement hyper server; chunk write from broadcast channel | cURL `localhost:8000/stream.mp3` streams indefinitely |
| 4 | IC3‑B | Wire prototype in `muxd::main` (hard‑coded input/output) | Roon‑Bridge -> Sonos works in lab |
| 5 | IC3‑D | Write dev script + docs | New dev can run fire‑and‑forget |
| 6 | IC3‑A | Add integration test harness (uses `assert_cmd`) | `cargo test` passes |

---

## Definition of Done
* Streaming stable >30 min without underruns
* Latency <3 s (Sonos buffer) measured via clap‑test
* README section “Quick demo” updated