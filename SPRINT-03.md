# Sprint 03 – Multi‑Input Mixer & Routing
**Duration:** 2 weeks  
**Sprint Goal:** support multiple sources (file, http, silence) mixed and routed per config.

---

## Deliverables
1. **Mixer** (`mux-core/src/mixer.rs`)
   * Accepts N `InputStream`s, applies per‑source gain (dB‑linear), priority ducking, soft‑knee.
2. **Input kinds**
   * `file` (Symphonia)
   * `http` (reqwest stream)
   * `silence`
3. **Routing engine**
   * Parses `[[routes]]` and builds runtime map → mixer → per‑output sub‑mix.
4. Updated config validator & example.
5. Unit tests:
   * Gain accuracy ±0.1 dB
   * Ducking reduces competing source within 10 ms.
6. Benchmark (`cargo bench`) mixing 4 inputs <2 % CPU on Ryzen 7.

---

## Task breakdown
| # | Task | Acceptance criteria |
|---|------|---------------------|
| 1 | Trait `Input` → async `next_frames(&mut self) -> Option<&[i16]>` |
| 2 | Implement `file` input with on‑demand read & `loop` option |
| 3 | Implement `http` input (icy reconnection) |
| 4 | Implement `silence` generator |
| 5 | Build `Mixer` with ring‑buffer and rodio‑style timing |
| 6 | Implement ducking side‑chain per `Route.duck_db` |
| 7 | Update validator, sample config, tests |
| 8 | Add `cargo bench` using `criterion` |

---

## Definition of Done
* Play Roon + alert MP3 concurrently, alert ducks music by configured dB.
* CPU usage meets benchmark.
* All new code covered ≥80 % by tests.