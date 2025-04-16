# Sprint 04 – Sonos Control & Resilience
**Duration:** 2 weeks  
**Sprint Goal:** daemon can autonomously push / maintain the stream on any configured Sonos rooms.

---

## Deliverables
1. **Sonos output module** (`mux-core/src/output/sonos.rs`)
   * Uses `sonor` for SSDP discovery + SOAP control.
   * Functions: `set_stream(room, url)`, `group(rooms)`, `keep_alive`.
2. Reconnect job (every 60 s) verifies URI & groups; re‑applies if lost.
3. Integration test with `upnp-rs` mock (simulated speaker).
4. New config option `outputs[].buffer_sec` default 3.

---

## Task breakdown
| # | Task | Acceptance criteria |
|---|------|---------------------|
| 1 | Discover Sonos devices on startup, map friendly name → IP |
| 2 | Implement `set_stream` using AVTransport `SetAVTransportURI` |
| 3 | Implement resilient group maintainer task |
| 4 | Add health‑check `/healthz` returning JSON list of rooms OK/KO |
| 5 | Write mock Sonos service for test harness |
| 6 | Docs: add “Adding new room” section |

---

## Definition of Done
* Kill power on a speaker → restored in ≤30 s.
* Health endpoint green in steady state.