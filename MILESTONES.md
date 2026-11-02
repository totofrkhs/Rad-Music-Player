# Rad-Music-Player — Milestones

Milestone plan for the Rust + Slint + Rodio music player. Each milestone maps to the active project skills.

---

## M1 — Project skeleton and Slint shell

**Objective:** Bootstrap the Rust workspace, configure Slint UI with the riced Linux terminal aesthetic (Catppuccin Mocha), and render the main window shell.

**Skills:** `rust-slint-ui`

**Deliverables:**
- `Cargo.toml` with `slint` and `slint-backend` dependencies
- `ui/main.slint` with root window, background `#1e1e2e`, surface panels `#313244`, accent `#fab387`
- Minimal layout: header, main content area, footer controls
- `cargo check` passes; window compiles and runs without web dependencies

**Definition of done:** Empty but functional Slint window opens, themed per the design guidelines, zero web/UI dependencies in Cargo.toml.

---

## M2 — Audio engine with Rodio and Symphonia

**Objective:** Initialize the Rodio output device and decode/playback an audio file using Symphonia.

**Skills:** `audio-dsp-rodio`

**Deliverables:**
- `audio/mod.rs` with `rodio::OutputStream`, `rodio::Sink`, and Symphonia decoder integration
- Play / pause / stop control wired from Slint via callbacks
- Audio runs on Rodio's output thread; UI thread never blocked
- `cargo check` and `cargo test` for audio module pass

**Definition of done:** A local audio file can be loaded and played from the Slint UI without freezing the interface.

---

## M3 — Parametric equalizer with biquad filters

**Objective:** Implement per-band biquad filters (lowpass, highpass, peaking, lowshelf, highshelf) and expose them through the UI.

**Skills:** `audio-dsp-rodio`, `rust-slint-ui`

**Deliverables:**
- `audio/equalizer.rs` with `BiquadFilter` struct and coefficient calculation
- Cascade of biquads applied to PCM before the Rodio sink
- Slint sliders/controls for frequency, Q, and gain per band with `<->` bindings
- Coefficient validation (audible range, positive Q, bounded gain dB)

**Definition of done:** EQ bands are audible in real time and adjustable from the Slint UI without audio glitches.

---

## M4 — Async yt-dlp downloader with covers and ID3

**Objective:** Download audio asynchronously with `tokio::process`, manage a bounded concurrent queue, extract cover art, and read/write ID3 tags.

**Skills:** `ytdlp-process-manager`

**Deliverables:**
- `downloader/mod.rs` with `tokio::process::Command` wrapping yt-dlp
- `DownloadQueue` with concurrency limit and job states (pending, running, completed, failed)
- Cover art extraction (`--write-thumbnail`) saved alongside the audio file
- ID3 read/write via the `id3` crate
- Progress and errors streamed back to the UI via channels

**Definition of done:** A URL can be enqueued, downloaded, tagged, and cover-art extracted without blocking the UI.

---

## M5 — Local persistence with Rusqlite

**Objective:** Store playlists, track metadata, and download cache in a local SQLite database.

**Skills:** (none specific; core Rust + Rusqlite)

**Deliverables:**
- `db/mod.rs` with Rusqlite connection and schema (tracks, playlists, playlist_tracks)
- CRUD operations for tracks and playlists
- Integration with downloader to persist metadata after download
- Wrapped in background tasks so the UI thread is never blocked

**Definition of done:** Tracks survive restart; playlists can be created and listed from the UI.

---

## M6 — Engram state sync and integration polish

**Objective:** Register architecture decisions and milestones in Engram for cross-session traceability, then polish end-to-end integration.

**Skills:** `engram-state-sync`

**Deliverables:**
- `mem_save` entries for each completed milestone (What / Why / Where / Learned)
- Final integration check: play local file, play downloaded file, EQ persists, playlist loads
- `mem_session_summary` closed out

**Definition of done:** All previous milestones work together; decisions are persisted in Engram for the next session.

---

