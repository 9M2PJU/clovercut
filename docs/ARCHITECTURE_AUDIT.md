# Phase 1 technical baseline and architecture audit

Historical Phase 1 baseline: Phase 2 was subsequently authorized. Current stability work, corrections and the durability approval gate are tracked in [STABILITY_PHASE2.md](STABILITY_PHASE2.md).

Audit date: 2026-10-21. Application revision: `82a498be342d02bba0429ec428eacc8c8b0e6f91`.
This is an audit of the unchanged application. Documentation changes do not alter editing behavior. Phase 2 is not authorized or started. The initial worktree contained only the untracked, user-requested `POLISH_PLAN.md`.

## Environment and reproducibility

Host: x86_64 Linux, Arch/Omarchy kernel `7.2.8-9-omarchy`; AMD Ryzen 5 5600X (12 logical CPUs), approximately 15 GiB RAM and 15 GiB swap. PCI GPU: AMD Navi 23 (`1002:73ff`). The smoke log identifies the selected adapter as AMD Radeon RX 6600 XT, discrete Vulkan, RADV/Mesa `26.2.5-arch1.1`; this is adapter evidence for that run rather than proof every conditional test exercised hardware decode. Rust and Cargo 1.93.1 match `src/rust-toolchain.toml`; the default toolchain outside `src/` is 1.97.0, so Cargo checks run from `src/`. Local FFmpeg is 9.0.2: libavcodec/libavformat 63.1.102, libavutil 61.1.102, libavfilter 12.1.102, libswscale 10.1.102, libswresample 7.1.102. Protoc 36.1 and actionlint 1.7.12 are available. ALSA 1.2.16.1, fontconfig 2.18.3, FreeType pkg-config 26.7.20, GTK 3.24.52. ONNX Runtime 1.28.0 and Sherpa native archives are already installed; native-library environment is sourced from `.env.local` without recording its values. Lockfile pins Sherpa 1.13.7, Slint 1.18.1, wgpu 30 and patched whisper-rs-sys 0.15.0; manifests and `Cargo.lock` are authoritative for the rest.

The existing `src/target` cache was about 61 GiB at audit start. Core workspace checks are warm-cache, offline and locked where applicable. Initial gRPC checks failed because the offline cache lacked `axum 0.8.9`; retries allow Cargo to fetch lockfile-pinned public crates. This is a prerequisite failure, not a failed application assertion. Results qualify this local dependency combination, not the checksum-pinned BtbN release bundle or a clean-room build. Commands requiring GPU/audio/native build-cache access run outside the filesystem sandbox with authorization. The sandbox hides `/dev/dri`; that initial absence was a sandbox restriction, not absent host hardware. The host exposes `renderD128`. No system packages were installed and no release publication or remote workflow dispatch was performed. Cargo fetched missing gRPC dependencies without changing `Cargo.lock`. Optional model downloads and the OpenGameArt external-service test were not performed.

Raw command logs, JSON exit/timing records and standalone audit probes live at `/home/turotulco/Work/clovercut-phase1-audit/`. This report retains the portable result summary. Historical resume reports were consulted only to locate environment prerequisites; their passing counts are not counted as current audit results.

## Release scope and critical dependencies

| Target | Checked-in release support | Audit limit |
|---|---|---|
| Linux x86_64 | Ubuntu 22.04 runner, AppImage; staged checks and Ubuntu/Debian container smoke scripts | Local host is Arch; package/container compatibility must be qualified separately. |
| Linux aarch64 | Ubuntu 22.04 ARM runner, AppImage | No native ARM execution on this x86_64 host. |
| Windows x86_64 | Windows 2022 runner; unsigned Inno Setup installer and portable ZIP | No native Windows installation/launch test here. |
| macOS, iOS, Android | Inherited source and scripts; Android activity remains a workspace member | Not built by current release workflow; no current `mobile.yml`. Source presence is not release support. |
| Nix Linux x86_64/aarch64 | `flake.nix` provides developer/build paths | Distinct from workflow packages; no Nix build implied by Cargo success. |

Evidence: `.github/workflows/{build-app,release,ci,nix}.yml`, `docs/RELEASING.md`, root README, `flake.nix`. Linux package builds override `profile.app.lto=false`, UI opt-level 0 and codegen-units 16, unlike the generic fat-LTO profile. Release CI and all package builds must pass before assets upload to a **draft** release.

| Layer | Dependencies and responsibilities |
|---|---|
| `concat-core` | std only: rational/frame time, arena, frame data, render timeline and speed curves. |
| `concat-project` | core + serde: `f64` persisted clip ranges, commands, migration, placement, Arc-backed project/history. Depends on core; the original layout table omitted it. |
| `concat-media` | core + linked FFmpeg through ffmpeg-the-third; decoding, encoding, probing, audio graph, scheduling, frame caches and prefetch. Only layer using libav APIs directly. |
| `concat-render` | core + wgpu 30: pure frame planning, production GPU executor and metrics. CPU reference is `cfg(test)` only; software GPU adapters are the production fallback. |
| `concat-effects` | project + core + serde/toml/naga: catalogue, effect metadata, WGSL generation and audio-chain templates. |
| `concat-vision` | core + project; optional `infer` enables ONNX Runtime. Export reads cached masks without requiring inference; host enables inference. |
| `concat-text` | fontdb/rustybuzz/ttf-parser/unicode/tiny-skia: typography, font discovery and rasterization. |
| `concat-export` | core/media/render/project/effects/vision: flattening, retime/transition resolution, preview plans, rendering and audio/video muxing. |
| `concat-host` | engine layers + text, CPAL, downloads and filesystem services: session, save ordering, jobs, monitor, PCM playback, generated titles, models/proxies and recovery logs. |
| `concat-speech` | host/media, Whisper and Sherpa; desktop app enables `chatterbox` with ONNX/tokenizers/vision. Feature selection affects lint coverage. |
| `concat-api` | host + project/export/effects/core: common request dispatch and project ownership; optional schema derives. |
| `concat-server` | API over JSON-RPC sockets; optional gRPC enables tonic/prost/tokio. Render work remains threaded, not Tokio-based. |
| `concat-cli` / `concat-mcp` | adapters over common API/server; CLI also has direct engine probe/render/effect commands. |
| `concat` | Slint/Skia GUI + host/speech/API/server/MCP and engine types; executable `clovercut`. Linux adds Wayland drops. |
| `concat-android` | inherited activity hosting the same UI library. |

Two local dependency patches require deliberate upgrade review: Slint Skia text/surface behavior and Android Vulkan→OpenGL surface fallback; Whisper's shared CPU-backend loader and variant packaging. The latter patch omits several upstream GPU backend sources; enabling those unsupported features is not a valid generic build matrix.

The GUI uses Slint 1.18/Skia and shares wgpu 30 with the engine. GPU handles and queue ownership are correctness constraints: `Monitor::texture_of` runs on the UI thread; `Monitor::off_thread` does not share the raw Vulkan submission queue with export workers. `--software` selects a software GPU/renderer path, not the removed production CPU reference compositor.

## Complete editing workflow and ownership

1. **Startup:** `concat/src/main.rs` and `launch.rs` choose CLI/MCP/smoke/window startup. `lib.rs::start` creates GPU/UI and services. `host.rs::Shell` owns a weak Slint window, `RefCell<Studio>` and persistent Slint models on the event-loop thread. `Studio` is the UI controller; `concat-host::Session` holds the authoritative editor/project. An `echo` clone previews uncommitted gestures. Slint view files contain input/layout/model interfaces, not the saved project authority.
2. **Import:** Slint callbacks in `lib.rs` and Wayland drops route to `MediaMsg::Import` in `panes/media_bin.rs`. `spawn_in_project` probes files through `concat-host::media::probe` → `concat-media::probe`; results become `NewMedia` entries and a batched `Command::AddMedia`. Probe errors become user notifications. Adding selected media to a lane is a separate `AddClipsInSequence` command; import alone must not edit the timeline. Remote API imports use the same project/media vocabulary.
3. **Edit:** callbacks for move/trim/split/keys route to `studio.rs` and panes. `Studio::apply` flushes pending commits, removes gesture echo and calls `Session::apply` → `Editor` → `commands::apply`. `Editor` records Arc-shared whole-state snapshots, capped at 200; `apply_within` coalesces one gesture into one undo step. Rejected commands notify without claiming success. `after_change` increments revision, trims stale selection, schedules autosave, refreshes audio/art/preview and analysis requirements. View-only locks/zoom/heights belong to `TimelinePane`; direct API commands have no access to those UI-only locks.
4. **Preview:** `Studio::preview_clips` chooses timeline/source, appends generated title sources and active treatment overrides. `panes/monitor.rs::request` permits one in-flight frame and coalesces demand with `wanted`; it captures time, resolution, proxy policy and flattened clip snapshot. Workers decode through `concat-host::Monitor` and the shared reader pool; export's `PreviewPlan` and `preview_sources_of` resolve frame content. GPU sources return to the event-loop thread for `texture_of`; a non-shared monitor returns pixels. Prefetch runs separately. Project epochs discard results for closed/replaced projects. Same-project requests are coalesced, not individually cancelled: an older frame may land before the latest wanted frame. This is an identified behavior to measure, not a proven persistent stale-frame bug.
5. **Library playback:** `source.rs::scratch` clones the project, uses one unmuted visible track and adds only the chosen media. `Source` owns its transport state, `f64` time and timer. Source selection temporarily borrows monitor/speakers without adding a real timeline clip; `picture_name` independently handles video/still/audio and timeline gaps. The prior black-library-source issue already has a fix and regression in this revision; historical reports are not new defects.
6. **Timeline playback:** `Studio::sync_audio` flattens session clips, converts via `audio_specs` and hands `ClipSpec`s to `Playback`. Two PCM workers decode/cache at 48 kHz; cached WAV PCM is mmap-backed. The supervisor opens CPAL output and rebuilds on stream/default-device changes, with retries/backoff when output is unavailable. Device callbacks mix samples/update clock and gain meters; the 8 ms Slint transport timer polls `position_now`, converts to `f32`, updates frame/playhead and requests preview. `Shared` stores integer microseconds and interpolates for at most 100 ms without another callback. Source transport uses its own timer. Current continuous pitch-preserving speed curves depend on FFmpeg `rubberband`, `asendcmd` and `asetnsamples`; smoke checks their presence.
7. **Save:** edits restart a 1,500 ms single-shot debounce. `Studio::save` calls `Session::prepare_save`, takes a save ticket at capture time and submits `projects::save_ordered` on a worker. Unique create-new temporary files are serialized by per-project write state, `sync_all`ed, closed and renamed over the manifest; obsolete tickets skip. Save errors restore `dirty` and notify. File contents are flushed, but the parent directory is explicitly not synced. A successful ordinary-save test does not establish power-loss rename durability.
8. **Reopen / project switch:** `open_project` first saves/releases the old project, claims ownership through `OpenProjects`, and opens `Session`. The session distinguishes missing/settings-only documents from corrupt/unreadable/newer ones. `doc.rs` migrates/settles valid entries and preserves unknown fields, but intentionally drops malformed list items or orphan clips. Effects/masks may be upgraded; retired fields are reported. Opening clears view/cache state, advances project epoch, sweeps stale proxies, schedules assets and checks missing media. Tolerant loading is a documented policy; its data-loss tradeoff merits recovery UX review rather than assuming corruption is ignored wholesale.
9. **Export:** `panes/export.rs` validates filename/overwrite consent, claims `Exporter::begin`, snapshots flattened clips plus title sources, selects dimensions/rate/codec, pauses playback, then dispatches a worker with `Monitor::off_thread` where safe. `export::run_on` → `concat-export::render_on` resolves transitions/retime, rounds video time to the requested frame grid, renders/encodes video, mixes sample-positioned audio, muxes, and renames the finished staged file. Errors and progress return to the UI; `Job` drop releases the SingleFlight slot. API export shares the exporter when attached to the window; standalone API instances have separate slots. No global destination lock is visible in the export renderer.
10. **Close and shutdown:** `save_and_release` flushes typing, pauses, synchronously saves the latest snapshot, and stops teardown on failure. `release_project` cancels relevant analysis jobs, releases project ownership, stops timers, clears source/monitor/cache state and advances epoch. Export is dispatched through non-project-gated `spawn`, so its completion is not an abandoned project result. Final process exit flushes logs and uses Linux `_exit` without joining process-lifetime GPU/audio/worker threads. This is deliberate native-library shutdown avoidance; it must not be replaced casually with general thread joining.

Concurrency evidence: `host.rs` has three process-lifetime workers fed by an unbounded mpsc queue plus project-epoch delivery guards; media scheduler priorities handle artwork/filmstrips. Preview is serialized/coalesced, save order is ticketed, and long operations use independent SingleFlight/cancel handles. Epoch checks prevent stale application of results, not necessarily wasted work already running. `panic=abort` in the shipping profile ends the process on panic; unwinding-based slot release protects unwind builds but cannot recover an aborted application.

## Timing boundary map

| Boundary | Actual representation/policy | Existing coverage / remaining gap |
|---|---|---|
| UI geometry, callbacks and playhead | Slint numbers / Rust `f32`; `Studio::frame_rate()` also narrows rational rate to `f32` | Snap/gesture tests exist; no comprehensive six-rate, long-duration roundtrip matrix found. |
| Saved clip and command ranges | `model::Clip::{start,duration,source_start}` are `f64`; menu `split_at` passes converted time directly, razor `split_clip_at` rounds using an `f32`-derived fps | Split/source continuity and trim/undo tests exist. Menu/razor rounding policies differ by inspection; visible impact needs reproduction. |
| Core timeline | exact reduced `i64` rational, arithmetic through `i128`; `frame_at` floors and ranges are half-open | `ntsc_never_drifts`, frame-boundary, range, checked-rate and arithmetic tests. |
| Floating input into core | `Rational::approximate` rounds to integer microseconds, rejects invalid/huge inputs | Exact rational guarantees do not recover precision discarded by UI or this conversion. |
| Video flatten/resolve | `ExportClip` holds `f64`; `resolve::quantise` rounds `seconds*fps` to nearest nonnegative frame; `quantise_at` rebases on the global grid | Fractional-start consolidation and smooth speed-curve transition/export regressions exist. |
| Preview instant | `preview_sources_of` quantizes to nearest project frame | Transport/cadence use floor with epsilon, so they do not share exactly the same instant-selection rule. |
| Decode/source timestamps | integer stream ticks → rational timebase, normalized by stream start; seeks compare real PTS | Existing nonzero-origin and sequential-versus-seek tests, plus TS fixtures. Variable frame rate breadth remains to qualify. |
| Audio mix | 48 kHz; clip start/duration and mix end round to sample counts; `adelay=...S`, `atrim=end_sample`, sample-based PTS | Sample graph tests and real MP3/ramp/export regressions. Audio/video grids intentionally differ. |
| Retime | speed curves use floating integration; preserve-pitch uses persistent Rubber Band filter with source-clock commands; command blocks are 48 samples (1 ms), equal rates suppressed at 1e-6 decimal precision | Continuous clock and tone-quality regressions exist; their tolerances are looser than a universal one-frame sync guarantee. |
| Playback clock → UI | integer microseconds → `f64` → `f32`; transport frame is `floor(f64(position)*fps + 1e-6)` | Clock stall/coast test exists; no end-to-end clock/UI/preview rounding parity test found. |

Standalone `timing-probe.rs` compiles the repository's actual `concat-core/src/time.rs`, asserts exact rational frame roundtrips, and applies the exact checked-in transport and preview conversion formulas. It records arithmetic evidence rather than exercising Slint events or claiming a GUI defect:

- At 24 fps, exact frame 50 is 2.083333333… seconds; `f32` stores 2.0833333, transport floor returns 49, preview nearest-frame returns 50.
- First examples below 10,000 frames: 25 fps frame 39; 30/60 fps frame 35; 24000/1001 frame 25; 30000/1001 frame 31.
- At 60 fps, frame 15,728,641 (262,144.016666… seconds, about 72.8 hours) becomes `f32` 262144.03 and re-quantizes to frame 15,728,642. This confirms a precision limit, not a claim typical short projects are broken.

Do not change every stored time type in response: first add a focused reproduction through the affected transport/edit path, define desired rounding and preserve document compatibility.

## Baseline execution results

Local baseline execution is complete within the scope and limitations below. The interrupted Clippy invocation had no recorded exit status and was not counted as passed; the resumed invocation passed. Initial offline gRPC attempts failed on an uncached dependency, then both checks passed after locked Cargo downloads. No application test assertion failed in the completed runs.

Already completed before interruption (JSON exit records retained): `cargo fmt --check` (0.93 s), locale inventory (0.10 s), model manifest consistency (0.06 s), locked/offline workspace metadata (0.03 s), actionlint (0.03 s), 13 isolated core-time tests (0.02 s), and the timing conversion probe (exit 0). The latter succeeds by asserting exact rational roundtrips and exposing divergent float formulas; success does not mean those formulas agree.

| Check / recorded command | Actual result | Wall time |
|---|---|---|
| fmt: `cargo fmt --check` | Exit 0 | 0.93 s |
| locales: `python3 scripts/locales.py --check` | Exit 0 | 0.1 s |
| models: `python3 scripts/models.py --check` | Exit 0 | 0.06 s |
| metadata: `cargo metadata --offline --locked --no-deps --format-version 1` | Exit 0 | 0.03 s |
| actionlint: `/tmp/clovercut-release-tools/actionlint` | Exit 0 | 0.03 s |
| rational-time-tests: `/home/turotulco/Work/clovercut-phase1-audit/time-tests --nocapture` | Exit 0; 13 passed, 0 failed, 0 ignored (all target summaries, including doctests) | 0.02 s |
| timing-probe: `/home/turotulco/Work/clovercut-phase1-audit/timing-probe` | Exit 0 | 0.0 s |
| clippy-workspace: `cargo clippy --offline --locked --workspace --all-targets -- -D warnings` | Exit 0 | 675.13 s |
| workspace-tests: `timeout 1800 cargo test --offline --locked --workspace -j 1 --no-fail-fast -- --test-threads=1` | Exit 0; 1052 passed, 0 failed, 5 ignored (all target summaries, including doctests) | 1460.64 s |
| grpc-clippy: `timeout 900 cargo clippy --offline --locked -p concat-server -p concat-cli --features grpc --all-targets -j 1 -- -D warnings` | Exit 101 | 0.63 s |
| grpc-tests: `timeout 900 cargo test --offline --locked -p concat-server --features grpc -j 1 -- --test-threads=1` | Exit 101 | 0.18 s |
| grpc-clippy-fetched: `timeout 900 cargo clippy --locked -p concat-server -p concat-cli --features grpc --all-targets -j 1 -- -D warnings` | Exit 0 | 508.4 s |
| grpc-tests-fetched: `timeout 900 cargo test --locked -p concat-server --features grpc -j 1 -- --test-threads=1` | Exit 0; 30 passed, 0 failed, 0 ignored (all target summaries, including doctests) | 366.28 s |
| gui-build: `timeout 1200 cargo build --offline --locked -p concat --bin clovercut -j 1` | Exit 0 | 342.37 s |
| gui-smoke: `timeout --kill-after=10 180 target/debug/clovercut --smoke` | Exit 0 | 0.95 s |
| cold-smoke-repro: `timeout --kill-after=10 180 target/debug/clovercut --smoke` | Exit 0 | 0.49 s |
| graphical-playback: `timeout --kill-after=10 300 /home/turotulco/Projetos/clovercut/src/target/debug/deps/concat-0d977a708c3d3a3d source::tests::new_project_library_and_timeline_playback --exact --ignored --nocapture --test-threads=1` | Exit 0; 1 passed, 0 failed, 0 ignored (all target summaries, including doctests) | 0.89 s |

The workspace run completed with **1,050 unit/integration tests and 2 doctests passed**, zero failures and five ignored tests. Its wall time includes waiting for the earlier Clippy artifact lock. The normal workspace run ignored graphical playback, OpenGameArt live access, Android MediaCodec/device coverage, installed-model transcription and the stabilization performance benchmark. Graphical playback passed separately (one test, exit 0); the other four are not counted as passed. The network integration was not invoked against its live service, the Android device and transcription fixture/model were unavailable, and the release-mode performance benchmark is outside this correctness baseline.

The local UI Clippy process was observed at approximately 9.5 GiB RSS, and test/app compilation used swap. These are observations rather than sampled peak measurements, and describe compilation rather than runtime application memory. Later-phase performance work still requires explicit runtime workloads and measurements. Timings of queued Cargo commands include artifact-lock waits; they are not standalone throughput benchmarks.

Cargo commands run from `src/` with the pinned toolchain and `.env.local` native dependency setup; no environment values/secrets are included in logs. The local runner explicitly overrides `CARGO_BUILD_JOBS=1` (the machine environment otherwise sets 2). Dev/test debug and codegen settings use repository/Cargo defaults; unlike CI, no `CARGO_PROFILE_{DEV,TEST}_DEBUG=0` or codegen-units=16 override was set. This difference increases local UI build memory and prevents treating timings as CI-equivalent. `cargo-nextest` is not installed, so the Cargo fallback lacks nextest's process-per-test isolation, per-test timeout and JUnit behavior. Serial libtest execution limits concurrent interference but does not provide that isolation. `CONCAT_REQUIRE_GPU=1` is set for the workspace tests. Local FFmpeg differs from CI's pinned download, so this is a local baseline rather than a clean CI reproduction.

The standalone timing probe mirrors the checked-in formulas and imports the actual core time module. Reproduce from repository root with Rust (the standalone run used the root's default toolchain):

```sh
rustc --edition=2024 --test src/crates/concat-core/src/time.rs -o /tmp/clovercut-time-tests
/tmp/clovercut-time-tests --nocapture
rustc --edition=2024 docs/audit/timing-probe.rs -o /tmp/clovercut-timing-probe
/tmp/clovercut-timing-probe
```

The probe searches the first mismatch below 10,000 frames at each rate; it does not test every possible time. The long-timeline case explicitly asserts frame information was lost. Hardware/codec tests may return early under their own checks; see the coverage limits below.

Both strict Clippy configurations passed. The explicit gRPC run passed all 30 tests after its dependencies were fetched (366.28 s including compilation, with tests taking 12.69 s). The patched Whisper dependency emitted one build-script and five generated-binding warnings during workspace checks; these did not fail the strict workspace lint and were not modified in this audit.

The GUI build passed (342.37 s). The first isolated smoke passed (0.95 s), as did the explicitly invoked graphical regression (one test, 0.89 s including process overhead). Two separate fresh-cache smoke runs produced the same 37 missing-effect-card image diagnostics before `smoke: ok`; the second passed in 0.49 s. This is recorded as U1 rather than silently calling startup warning-free. The smoke test exits after first publish and does not establish whether thumbnails remain absent in a running interactive window.

## Findings, classification and next-phase handoff

| ID / priority | Classification and evidence | Handoff |
|---|---|---|
| S1 / P0 impact if power loss occurs | **Confirmed durability limitation**, not reproduced edit loss: `concat-host/src/projects.rs::Temporary::{write,commit}` syncs file contents before rename but deliberately does not sync the parent directory. Ordinary unit tests cannot qualify power-loss durability. | Phase 2 task 2.2: decide the guarantee, then isolate any implementation and platform failure tests. Preserve ordered tickets and current refusal/recovery semantics. |
| T1 / P1 | **Reproduced arithmetic mismatch** in the six-rate timing probe: transport floor and preview nearest-frame disagree after conversion through `f32`. 24 fps frame 50 is a short-project example; the 72.8-hour example separately demonstrates loss of a whole frame in `f32`. No visible GUI editing defect is claimed. | Phase 3 tasks 3.1–3.2 and new 3.6: reproduce through actual transport/publication and define rounding before fixing it. Do not replace all document time types. |
| D1 / P2 | **Verified documentation defects**: source README described rational-only times, optional production GPU, Linux-only releases, obsolete Nix executable and a nonexistent mobile CI workflow; export crate documentation promised a removed production CPU fallback. Evidence is in manifests, renderer cfgs, flake and release workflows. | Corrected in this audit as documentation only; no compositor or editing behavior changed. |
| E1 / P1 if concurrent exports collide | **Source-confirmed shared staging names; runtime failure unverified**: `concat-export/src/lib.rs::render_on` builds video/audio/mux sibling names from the destination stem, without unique job identity. Independent exporters have independent SingleFlight slots. | New Phase 7 task 7.6: disposable-directory concurrent reproduction before selecting a fix. Do not claim real user exports collided. |
| E2 / P2 | **Cancellation coverage gap**: synchronous audio mix/mux calls have cancellation checks around them, not an interrupt callback inside them. No measured upper bound or freeze reproduction. | Expand Phase 7 task 7.3 to measure cancellation at each stage. |
| U1 / P2 | **Reproduced nonfatal fresh-cache diagnostics**: two isolated `clovercut --smoke` runs each emitted 37 `Error loading image ... /cards/... No such file` messages and then `smoke: ok`, exit 0. | New Phase 5 task 5.5: observe a fresh-cache interactive session and thumbnail completion. The smoke exits after first publish, so persistent missing cards or a user-visible rendering failure are not established. |
| C1 / P2 | **Qualification gaps**: opt-in graphical/audio/model/device tests, native Windows/ARM/package targets and global-state isolation differ from ordinary local Cargo runs. | Phase 4 playback measurements and Phase 8 target matrix; do not count skipped or early-return hardware paths as verified. |

The first Phase 2 work should verify existing save-failure/corrupt-document/reopen protections in disposable projects (2.1), then decide and document directory-sync durability (2.2). Do not weaken the already-present ticket ordering, atomic staging or corrupt-document refusal. Lifecycle interruption checks (2.3) follow before unrelated controller extraction. Timing T1 is the first focused Phase 3 correctness investigation. No speculative runtime fix or major refactor is part of this audit.

## Coverage gaps and audit limits

- Hardware GPU qualification is local only; software adapters, Windows DX12, ARM64 and packaged artifacts are separate matrix entries. GPU test helpers can return successfully without exercising rendering if no adapter exists; `CONCAT_REQUIRE_GPU=1` makes adapter absence fatal in the parity helper. Other tests have their own early-return conditions, so aggregate success is not unconditional codec/hardware coverage.
- The explicitly ignored graphical source/timeline regression passed when explicitly invoked and covers real video and transport ownership but does not measure audible A/V phase alignment or ten-minute drift. The audit invokes the exact GUI test executable built by the workspace directly, with fresh XDG configuration/data/cache/state directories; this avoids rebuilding a different Cargo feature graph. Device disconnect/reconnect and sound-card changes are not represented by pure PCM tests.
- Export cancellation is checked around `audio::mix_to_file`/`mux`, whose synchronous signatures take no cancel token. Cancellation latency inside those stages has no bound from per-video-frame polling. Measure before defining a responsiveness fix.
- Export staging names derive only from destination stem (`.<stem>.concat-video.mp4`, audio and mux variants). Independent renderer callers/processes are not protected by one shared SingleFlight. Collision and stale-file handling need a focused disposable-directory reproduction; no concurrent user output was touched during audit.
- Parent-directory syncing, actual power loss, disk-full behavior and physical GPU/device failure were not simulated against real user projects.
- No major refactoring is justified by file size alone. The concrete coupling points for future small extractions are gesture time conversion, after-change invalidation and frame publication; they need behavior-preserving regressions first.

## Completion and review gate

Phase 1 tasks 1.1–1.5 are complete for this local audit. Deliverables are this report, the updated `POLISH_PLAN.md`, corrected engine/export documentation and `docs/audit/timing-probe.rs`. Native packaging/platform and device/model/fault qualification gaps remain explicit. No application behavior or project format was changed; the export source edit is crate documentation only. Phase 2 is not started and requires the user’s approval.

Post-documentation verification passed: `cargo fmt --check` (1.17 s), `git diff --check`, compilation/execution of the relocated portable timing probe (0.75 s / exit 0), and its standalone rustfmt check. A comparison against HEAD verified that the export crate content from `pub mod chains;` onward is byte-for-byte unchanged; the tracked source changes are README text and the crate documentation header only. All Phase 1 checkboxes are complete, and no Phase 2–8 task was marked started or complete. Raw receipts remain in the audit artifact directory.
