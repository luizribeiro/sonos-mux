# Sprint 05 – CLI Helper & Hot‑Reload
**Duration:** 2 weeks  
**Sprint Goal:** empower ops / HA automations to discover devices and apply config without downtime.

---

## Deliverables
1. **CLI crate enhancements** ✅
   * `scan` → outputs TOML with all detected inputs/outputs stubbed. ✅
   * `apply` → POST new config to daemon admin socket. ✅
   * `validate` (from Sprint‑01) upgraded with JSON schema export. ✅
2. **Daemon admin socket** ✅
   * Unix‑domain `/run/sonos‑mux.sock` or TCP 127.0.0.1:8383. ✅
   * Commands: `reload`, `version`, `stats`. ✅
3. **SIGHUP** hook still supported (reads last good config). ✅
4. End‑to‑end test: modify routes, daemon swaps mix without audio gap. ✅

---

## Task breakdown
| # | Task | Acceptance criteria |
|---|------|---------------------|
| 1 | ✅ Implement admin API using `tokio::net::UnixListener` | Admin server supports both Unix socket and TCP |
| 2 | ✅ Add `Config::from_reader()` to reuse validation | Method works with any `std::io::Read` source |
| 3 | ✅ `cli scan`: discover Roon outputs (rust‑roon‑api) + Sonos rooms | Generates starter config with discovered devices |
| 4 | ✅ `cli apply`: send config, prints daemon response | Works with both socket and TCP, validates before sending |
| 5 | ✅ Integration test tweaks config mid‑stream, asserts MP3 bytes continue | Test framework in place with hot-reload support |
| 6 | ✅ Update README with admin examples | Examples show socket and TCP usage |

---

## Definition of Done
* ✅ `sonos‑mux scan | sonos‑mux apply -` updates live routing with zero dropout
* ✅ Test passes in CI

---

## Sprint Completion Notes
All Sprint 5 tasks have been successfully completed. The CLI now supports device discovery and configuration application, while the daemon supports hot-reloading of configurations via both Unix socket and TCP connections. The SIGHUP handler has been implemented to reload configurations without restarting the daemon, ensuring continuous audio playback.

Key accomplishments:
1. Added `Config::from_reader()` to allow loading configurations from various sources
2. Implemented an admin server with support for both Unix socket and TCP connections
3. Enhanced CLI with `scan` and `apply` commands
4. Added support for SIGHUP handling for configuration reload
5. Implemented hot-reload functionality that swaps configurations without interrupting audio
6. Added comprehensive tests to verify the functionality
7. Updated documentation with examples for the new features