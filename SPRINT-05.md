# Sprint 05 – CLI Helper & Hot‑Reload
**Duration:** 2 weeks  
**Sprint Goal:** empower ops / HA automations to discover devices and apply config without downtime.

---

## Deliverables
1. **CLI crate enhancements**
   * `scan` → outputs TOML with all detected inputs/outputs stubbed.
   * `apply` → POST new config to daemon admin socket.
   * `validate` (from Sprint‑01) upgraded with JSON schema export.
2. **Daemon admin socket**
   * Unix‑domain `/run/sonos‑mux.sock` or TCP 127.0.0.1:8383.
   * Commands: `reload`, `version`, `stats`.
3. **SIGHUP** hook still supported (reads last good config).
4. End‑to‑end test: modify routes, daemon swaps mix without audio gap.

---

## Task breakdown
| # | Task | Acceptance criteria |
|---|------|---------------------|
| 1 | Implement admin API using `tokio::net::UnixListener` |
| 2 | Add `Config::from_reader()` to reuse validation |
| 3 | `cli scan`: discover Roon outputs (rust‑roon‑api) + Sonos rooms |
| 4 | `cli apply`: send config, prints daemon response |
| 5 | Integration test tweaks config mid‑stream, asserts MP3 bytes continue |
| 6 | Update README with admin examples |

---

## Definition of Done
* `sonos‑mux scan | sonos‑mux apply -` updates live routing with zero dropout
* Test passes in CI