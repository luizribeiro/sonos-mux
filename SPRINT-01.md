# Sprint 01 – Repository & Configuration Scaffold ✅
**Duration:** 2 weeks  
**Sprint Goal:** lay the foundation so every later task plugs into a well‑typed, CI‑guarded workspace.  
**Status:** Completed

---

## Deliverables
1. `sonos‑mux/` Git repository with Apache‑2.0 LICENSE.
2. Cargo workspace:
   ```
   root/
     Cargo.toml              # [workspace]
     mux-core/               # core library crate
     muxd/                   # daemon binary crate
     cli/                    # helper CLI crate
     examples/               # throw‑away playgrounds
   ```
3. GitHub Actions workflow running `cargo check`, `test`, `fmt --check`, `clippy -- -D warnings`.
4. `config` module (in `mux‑core`) that:
   * Parses **inputs / outputs / routes** & logging opts from `config.toml`.
   * Exposes `Config::load(path) -> Result<Self>`.
5. Sample `config.toml` checked into `examples/`.
6. Unit‑tests covering:
   * happy‑path parse
   * unknown `kind` → error
   * duplicate `id` detection.
7. Updated **PROJECT.md** link + README quick‑start snippet:
   ```bash
   git clone …
   cargo run --bin muxd -- --config examples/config.toml
   ```

---

## Task breakdown
| # | Assignee | Task | Acceptance criteria |
|---|----------|------|---------------------|
| 1 | IC3‑A | Init repo, add Apache license & `cargo new ––lib mux-core` | Repo builds empty workspace in CI |
| 2 | IC3‑A | Add `cargo fmt`, `clippy` steps to `.github/workflows/ci.yml` | Pull request fails on style or warnings |
| 3 | IC3‑B | Define `Config` structs with Serde (`Input`, `Output`, `Route`, `Logging`) | Unit‑test deserialises sample |
| 4 | IC3‑B | Implement validation (`validate()`) ensuring id uniqueness, referenced ids exist | Tests cover all error cases |
| 5 | IC3‑C | Write sample `examples/config.toml` illustrating every input/output kind | File passes validation |
| 6 | IC3‑C | Add README "Installing Rust tool‑chain" & quick run instructions | Visible in repo root |
| 7 | IC3‑D | Create `cli` crate stub (using `clap`) with sub‑commands `validate` & `version` | `cargo run --bin cli validate examples/config.toml` returns 0 |

---

## Definition of Done
* All deliverables merged to `main` ✅
* CI green ✅
* `cargo audit` shows no vulnerable deps ✅
* Team walk‑through recorded (5 min loom)