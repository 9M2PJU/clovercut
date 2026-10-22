# Phase 6: Performance and Memory Optimization

## 1. Executive Summary

Phase 6 established quantitative baselines, characterized real-world bottlenecks on synthetic and representative media workloads, and applied targeted optimizations without correctness regressions across playback, editing, and preview/export parity.

Key benchmark highlights:
- **Project Close / Switch Resource Release (Task 6.3):** Eliminated retention of GPU layer textures (`510 MB -> 0 MB`), live title frame cache (`126 MB -> 0 MB`), studio title block maps (`61 -> 0`), and mapped PCM audio (`65 MB -> 0 MB`), freeing over `245 MB` of system DRM VRAM immediately on project close.
- **4K Title Painting Latency (Task 6.2C):** Reduced 4K title paint time from `89.16–108.83 ms` (`p95 = 89.35–123.01 ms`) down to `22.12–29.11 ms` (`p95 = 30.34–32.61 ms`) — a **3.6× speedup (-72% latency)**, bringing full-resolution 4K title composition with shadows within real-time budgets.
- **Caption-Heavy Timeline Transitions (Task 6.2B):** Separated base non-title timeline flattening from per-cue visible title painting (`flat_base`), cutting cold 60-cue transitions from `3571.36 ms` (`59.52 ms/cue`) down to `1003.04 ms` (`16.71 ms/cue`) — a **3.56× speedup**.
- **Undo/Redo Predictability (Task 6.3):** 200 undo/redo steps across 270 clips execute in `0.0017 ms/step` (undo) and `0.0010 ms/step` (redo) with `0.0 MB` RSS drift across repeated cycles.

---

## 2. Reference Machine & Profile Specification (Task 6.1)

- **CPU:** x86_64 multi-core Linux kernel 6.x
- **Total Physical Memory:** 15.53 GB RAM
- **GPU Backend:** Vulkan via `wgpu` (`radv` driver)
- **Engine Build Profile:** `quick` profile (`opt-level = 3` for all engine crates: `concat-core`, `concat-media`, `concat-render`, `concat-text`, `concat-project`, `concat-export`, `concat-host`).
- **Dynamic ReaderPool Budget:** `frame_budget(15.53 GB) = 993.8 MB` (clamped between 256 MB and 1024 MB).

---

## 3. Workload Baselines (3-Run Averages Before Optimization)

### 3.1 Startup, Seek Latency & Playback (Task 6.1)
- **Cold Init + Publish:** `640.95 ms`
- **Warm Project Open + Publish (3 runs):** `1.03 ms`, `0.56 ms`, `0.55 ms`
- **Cold Seek Latency (20 timestamps across 2 HD clips):**
  - Run 1: `mean = 30.47 ms`, `p95 = 64.06 ms`, `max = 71.66 ms`
  - Run 2: `mean = 14.78 ms`, `p95 = 20.69 ms`, `max = 28.27 ms`
  - Run 3: `mean = 13.10 ms`, `p95 = 20.70 ms`, `max = 25.59 ms`
- **Warm Cached Seek Latency (20 timestamps):**
  - Run 1: `mean = 0.230 ms`, `p95 = 0.869 ms`
  - Run 2: `mean = 0.081 ms`, `p95 = 0.105 ms`
  - Run 3: `mean = 0.196 ms`, `p95 = 0.725 ms`
- **Continuous 90-Frame Playback (30 fps):**
  - Frame times: `mean = 1.54–1.81 ms`, `p95 = 1.98–3.41 ms`, `max = 19.82–20.71 ms`
  - Dropped frames (> 33.3 ms): `0/90` across all 3 runs.

### 3.2 Long Audio (180s stereo 48kHz WAV + 12 spans) (Task 6.2A)
- **Peaks Extraction (`peaks::extract` + `Pyramid::of`):** `45.41 ms`, `45.46 ms`, `44.54 ms`
- **`publish_lanes` with Cold SVG Envelopes:** `1.824 ms`, `1.782 ms`, `1.820 ms`
- **`publish_lanes` with Cached SVG Envelopes:** `0.063 ms`, `0.061 ms`, `0.062 ms` (~29× cache speedup)
- **Mapped PCM Audio:** 13–15 memory-mapped spans totaling `63.7–65.2 MB`.

### 3.3 Caption-Heavy Project (240 cues + 30 video/audio clips = 270 clips) (Task 6.2B)
- **60x `flatten_timeline_in`:** `0.50 ms` (`0.008 ms/call`)
- **60x `preview_clips` Cue Transitions (Cold Text Paints):** `3571.36 ms` (`59.52 ms/cue`)
- **60x `preview_clips` Cue Transitions (Warm `Titles.live` Cache):** `1.03 ms` (`0.017 ms/cue`)
- **20x `publish_lanes`:** `5.79–6.01 ms` (`0.29 ms/pub`)

### 3.4 4K Layered Project (3840×2160, 2 video streams + PiP + 4K title) (Task 6.2C)
- **4K Title Paint (`3840×2160` with shadow + stroke + background plate):**
  - Run 1: `mean = 108.83 ms`, `p95 = 123.01 ms`
  - Run 2: `mean = 89.16 ms`, `p95 = 89.35 ms`
  - Run 3: `mean = 89.37 ms`, `p95 = 89.56 ms`
- **4K Multi-Layer Decode (`frame_sources`):** `mean = 22.34–23.56 ms`, `p95 = 98.33–103.85 ms`
- **4K Multi-Layer GPU Composite (`texture_of`):** `mean = 7.09–7.52 ms`, `p95 = 14.69–32.27 ms`

### 3.5 Memory Retention & 200-Step Undo/Redo (Task 6.3)
- **200-Step Undo/Redo:** 210 applies (`~28.5 ms`), 200 undos (`0.35 ms`, `0.0017 ms/step`), 200 redos (`0.25 ms`, `0.0012 ms/step`). RSS remained constant at `828.0 MB` across all runs.
- **Project Close Retention (Baseline Leak):**
  - `CPU Frame Pool`: `427.15 MB -> 0.00 MB` (released)
  - `GPU Layer Pool`: `510.21 MB -> 510.21 MB` (RETAINED)
  - `Live Titles Cache`: `126.56 MB -> 126.56 MB` (RETAINED)
  - `Title Blocks Map`: `61 -> 61` (RETAINED)
  - `PCM Mapped Audio`: `65.19 MB -> 65.19 MB` (RETAINED)
  - `DRM VRAM`: `1905.7 MB` held after close.

---

## 4. Applied Optimizations (Task 6.4)

### Optimization 1: Full Lifecycle Resource Release on Project Close / Switch
1. **GPU Layer Textures:** Updated `Monitor::clear()` in `concat-host/src/preview.rs` to call `gpu.shrink_pool(0.0)`, returning pooled textures to the device.
2. **Live Title Cache:** Added `Titles::clear_live()` in `concat-host/src/titles.rs` (clearing `LivePaintings.kept` and resetting byte tracking to zero). Called in `Studio::forget_art()` along with `self.title_blocks.clear()`.
3. **PCM Memory Mappings:** In `Playback::set_clips()` (`concat-host/src/playback.rs`), cleared `self.cache` and `self.queue.jobs` when the project path changes or is empty. In `decode_worker`, verified the job project still matches the active project before inserting decoded `Mmap` entries.
- **Measured Result:** After `close_project()`, CPU pool = `0.00 MB`, GPU pool = `0.00 MB`, Live titles = `0.00 MB`, Title blocks = `0`, PCM audio = `0.00 MB`. System DRM VRAM decreased by `245 MB`.

### Optimization 2: 4K Title Painting, Bounded Shadow Layers & Row-Major Blur
1. **Bounded Shadow Pixmap:** Replaced full-frame `Pixmap::new(width, height)` allocation (`33.17 MB` at 4K) in `concat-text/src/lib.rs` with a tightly bounded pixmap derived from `words.bounds()` expanded by stroke and blur radius (`sub_right - sub_left, sub_bottom - sub_top`), reducing layer size by over `19×` for typical titles.
2. **Row-Major 4-Channel Box Blur:** Rewrote `blur_full()` to process all 4 RGBA channels in a single pass with sequential row-major access, eliminating cache misses during column traversal and auto-vectorizing arithmetic while preserving exact byte-for-byte blur output.
3. **In-Place Premultiplied to Straight RGBA:** Updated `straight_rgba()` to consume `canvas.take()` in-place, eliminating the allocation and copy of a 33 MB buffer and skipping `demultiply()` for fully transparent (`a == 0`) and fully opaque (`a == 255`) pixels.
- **Measured Result:** 4K title paint time dropped from `89.16–108.83 ms` down to `22.12–29.11 ms` (**3.6× speedup**).

### Optimization 3: Base Timeline Flattening Cache & Asynchronous Font Loading
1. **Base Timeline Cache (`flat_base`):** In `Studio::preview_clips()` (`concat/src/studio.rs`), introduced a cache for non-title clips (`flat_base`) keyed by `(revision, project_dir)`. When seeking across subtitle cue boundaries, non-title tracks are reused without re-flattening the timeline.
2. **Asynchronous Font Warming:** In `Titles::new()` (`concat-host/src/titles.rs`), spawned background warming of `Fonts::new()` on thread `concat-fonts`, eliminating UI event-loop freezes on the first rendered title.
3. **Compact Reveal Map Hash:** In `RevealMap::from_rects` (`concat-core/src/shader.rs`), hashed non-zero word bounds instead of sequentially iterating over 8.29 million mostly-zero bytes on 4K frames.
- **Measured Result:** Cold caption cue transitions dropped from `3571.36 ms` down to `1003.04 ms` (**3.56× speedup**).

---

## 5. Verification & Parity Results (Task 6.5)

After applying all optimizations, regression suites were executed with zero failures:
1. **Phase 6 Performance Suite:** `phase6_performance_and_memory_suite` passed in `7.22s` (down from `10.62s`).
2. **Phase 3 Timeline Timing & Edit Robustness:** `studio::phase3_timeline_tests` (5/5 passed, `7.14s`).
3. **Phase 4 Preview/Export Parity:** `studio::phase4_preview_export_parity_tests` (1/1 passed, covering speed curves, reverse, transitions, captions, multi-track audio, proxies, fractional rates).
4. **Phase 4 Playback Robustness:** `studio::phase4_playback_robustness_tests` (1/1 passed, `6.47s`).
5. **Concat & Core Unit Tests:** `concat --lib` (171 passed), `concat-core --lib` (65 passed), `concat-text --lib` (33 passed).
