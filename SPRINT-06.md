# Sprint 06 – Control API, Metrics & Packaging
**Duration:** 2 weeks  
**Sprint Goal:** production‑ready release with automation hooks and observability.

---

## Deliverables
1. **Control API**
   * WebSocket at `/ws` (or MQTT topic)  
     Commands: `{"set_volume": { "room":"Kitchen", "db":-5 }}`  
     Replies: `{"ok":true}`
2. **Prometheus metrics**
   * `/metrics` – counters for frames mixed, underruns, per‑room bytes, CPU%.
3. **Packaging**
   * `systemd/sonos‑mux.service` + tmpfiles.d
   * Nix flake `packages.x86_64-linux.sonos‑mux`
   * Docker `Dockerfile` (scratch + musl build).
4. **Integration tests**
   * GitHub Actions workflow spinning up Roon Bridge + mock Sonos in containers, verifying end‑to‑end for 60 s.
5. **Docs**
   * README sections for API, metrics, deployment.
   * Changelog v1.0.0.

---

## Task breakdown
| # | Task | Acceptance criteria |
|---|------|---------------------|
| 1 | Implement WebSocket control with `warp` |
| 2 | Volume handler routes to mixer gain in real time |
| 3 | Expose Prometheus metrics via `prometheus` crate |
| 4 | Create systemd unit & ensure graceful shutdown |
| 5 | Build static binaries (musl) in CI, attach to release |
| 6 | GitHub Actions matrix test (arm64 via QEMU) |
| 7 | Update CHANGELOG & tag v1.0.0 |

---

## Definition of Done
* One‑liner install on Pi works
* Grafana dashboard shows live metrics
* Release assets uploaded automatically on tag push