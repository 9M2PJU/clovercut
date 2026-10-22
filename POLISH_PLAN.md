# Clovercut polish plan

Created: 2026-10-21. Inspected revision: `82a498be342d02bba0429ec428eacc8c8b0e6f91`.

## Scope, evidence, and control

Phase 1 audit authorized by the user on 2026-10-21 and resumed after a session interruption. Phase 1 changed documentation only. Phase 2 tasks 2.1–2.4 were explicitly authorized on 2026-10-21, with independent verification and a durability design approval gate. Phase 3 tasks 3.1–3.6 were explicitly authorized on 2026-10-21 after Phase 2 approval, subject to the timing-semantics/design approval gate. Phase 3 was subsequently approved by the user as complete within its documented local verification scope. Development is being handed to Google Antigravity; Phase 4 and subsequent implementation remain unauthorized. The repository was clean before the initial planning document. See [the architecture audit](docs/ARCHITECTURE_AUDIT.md) for the current baseline; the initial-inspection statements below describe the planning stage only.

Review coverage: repository inventory, all 16 crate boundaries and manifests, crate entry-point documentation, representative implementation paths for persistence, timeline commands, preview, playback, rendering and export, UI organization, existing test locations, native dependency configuration, release/build scripts, workflows, and project documentation. The inventory excludes Git, vendor downloads, build output and Python caches; it includes 1,172 files, 202 Rust files and 78 Slint files, including local dependency patches. Assets, bundled fonts/models, effect packages and inherited mobile code were inventoried rather than individually executed or exhaustively reviewed. No claim of a line-by-line audit or runtime qualification is made.

Evidence labels:
- **Observed:** directly supported by checked-in code or configuration.
- **Verification gap:** behavior needs measured execution; this is not a confirmed defect.
- **Confirmed limitation:** explicitly present in implementation or its documentation.

Initial planning did not run builds, application launches, exports or tests, and did not confirm a runtime bug. Phase 1 execution results are now recorded in `docs/ARCHITECTURE_AUDIT.md`. The presence of a test alone remains coverage evidence rather than proof of success. Do not turn a suspected defect into a fix without a failing reproduction or an explicit design decision.

Priorities: P0 = edit loss/crash/corruption; P1 = editing or playback correctness/release blocker; P2 = consistency or measured efficiency; P3 = optional refinement. P0 denotes impact if reproduced, not proof a defect exists. Risk describes implementation risk: high for persistence, timing, GPU and rendering semantics; medium for interaction/state changes; low for documentation and isolated visual fixes.

Tasks initially started **Not started**. Use **Investigating**, **Blocked** (with reason), **Ready for review**, and **Complete** as work progresses. Every completion entry must record revision, reproduction/fixture, verification command or manual procedure, actual result and remaining limitations. Update this document in the same change as the work it tracks. Never mark skipped device/network/graphical tests as passing. New findings must get an evidence reference, priority and phase before implementation.

## Repository architecture and findings

Paths below are repository-relative. Crate paths abbreviated as `C/<name>` mean `src/crates/<name>`.

| Area | Observed ownership / finding | Implication |
|---|---|---|
| Core | `C/concat-core`: rational time, frame rate, arena, frame and timeline primitives | Preserve dependency direction and exact engine frame calculations. |
| Edit | `C/concat-project`: document migration, commands, placement, speed, model, snapshot undo | Project clip times are `f64`; they are not rational throughout the application. |
| Host | `C/concat-host`: Session, autosave, jobs, monitor, playback, proxies, downloads, crash logs | Cross-thread state and lifecycle need end-to-end verification. |
| Media | `C/concat-media`: FFmpeg decode/encode, audio filtergraph, reader pool and prefetch | Native libraries and hardware availability affect reproducibility. |
| Rendering | `C/concat-render`, `concat-export`, `concat-effects`, `concat-text`, `concat-vision` | Preview, generated captions, effects, speed mapping and export have multiple interacting paths. |
| UI | `C/concat`: Slint views, panes, Studio controller and callback wiring | `src/studio.rs` is 13,855 lines and `src/lib.rs` 2,977; ownership concentration complicates changes. Size alone is not a runtime defect. |
| Automation | `concat-api`, `concat-server`, `concat-cli`, `concat-mcp` | Commands and exported behavior must agree between UI and automation. |
| Optional/platform | `concat-speech`, `concat-android`, mobile scripts and local Slint/Whisper patches | Track model/device prerequisites and inherited targets separately from release claims. |
| Time boundary | Phase 3 retains authoritative edit times/origins/playhead/scroll as `f64`, uses viewport-local Slint coordinates and exact project rates for frame picks; documents remain version-1 `f64` seconds | Approved Option A preserves continuous edits. T1–T5 corrections, native interaction/encoded-cut regressions and six-rate real clock/publication checks passed; qualification limits remain in `docs/TIMELINE_PHASE3.md`. |
| Persistence | Ordered save tickets, unique temporary files, file sync and atomic rename are preserved. Phase 2 approved Option B adds Linux parent-directory sync and explicit committed-but-unconfirmed outcomes; unsupported newer documents are refused | Rename advances ordering even if directory sync fails; warn visibly and permit close after a committed save. Windows contract is unchanged; physical power-loss guarantees remain unverified. |
| Playback | `concat-host/src/playback.rs`: 48 kHz PCM, two decode workers, 64-entry in-memory cache, 2 GiB disk budget, callback clock with 100 ms coast | Measure device recovery, clock handoff and total resource use; entry limits alone do not specify a memory-byte ceiling. |
| GPU | `concat-host/src/preview.rs::off_thread` avoids sharing the raw Vulkan queue across threads; `concat-render/Cargo.toml` says the compositor is always GPU-backed | Preserve queue safety. Phase 1 corrected the obsolete CPU-fallback documentation: software GPU adapters are the production fallback, and the CPU reference is test-only. |
| Export | `concat-export/src/lib.rs:1226–1261` stages output and renames only on success; host uses `SingleFlight` | Do not replace existing transactional protections; exercise error and cancellation paths. |
| Coverage | Host export integration suite includes `every_edit_still_exports`, fractional consolidation, speed curves, reverse and HDR; GPU parity tests exist | Extend gaps rather than duplicate already-covered cases. |
| Graphical testing | `concat/src/source.rs::tests::new_project_library_and_timeline_playback` is explicitly ignored unless invoked with a graphical session, FFmpeg and audio | Ordinary workspace tests do not establish that real GUI playback works. |
| CI/docs | `.github/workflows/ci.yml` runs formatting, locales/models, Clippy, nextest, docs and gRPC checks on Linux with required GPU; `src/README.md` executable/platform/time descriptions were corrected in Phase 1 to match manifests and current workflows | Establish actual baseline and supported-target matrix; avoid treating inherited comments as current policy. |

## Dependencies and gates

Primary sequence: **1 → 2 → 3 → 4 → 5 → 6 → 7 → 8**. Each phase ends with a reviewable result and verification record. Phase 1 audit authorization was received on 2026-10-21; subsequent phase authorization must be explicit.

Additional dependencies:
- Phase 1 owns baseline, fixtures, target scope and evidence used by every later phase.
- Phase 2 persistence and lifecycle protections gate any edits touching saved projects or worker shutdown.
- Phase 3 timing rules gate Phase 4 clock comparisons and Phase 7 frame/duration assertions.
- Phase 4 preview/export parity fixtures feed Phase 7; Phase 5 must preserve Phase 3 keyboard/gesture semantics.
- Phase 6 optimization must retain Phase 3/4 correctness and be rerun through Phase 7 export checks.
- Export probes may be used earlier as verification without starting Phase 7 implementation.
- Phase 8 consolidates earlier evidence; regression tests accompany relevant fixes rather than being deferred until the end.

## 1. Codebase audit and architecture analysis

**Problems identified:** concentrated controller/wiring ownership; time representations differ from the rational-only convention in `src/README.md`; GPU fallback and platform documentation disagreed with implementation/current root README and were corrected during the audit. Local execution and differences from CI are recorded in the audit report.

**Tasks:**
- [x] 1.1 Record toolchain/native-library versions and clean baseline; map supported release targets from README, workflows and packaging. Separate inherited platforms.
- [x] 1.2 Trace import → command → session → save → reopen, and edit → flatten → preview/playback/export. Record state ownership, threads, invalidation, shutdown and error propagation.
- [x] 1.3 Map timing conversions and rounding policies, including source timestamps, audio samples and frame indices. List existing tests against each boundary.
- [x] 1.4 Run the established baseline checks in an appropriate environment; record actual failures, skips and prerequisites, without changing application code.
- [x] 1.5 Turn reproducible findings into small tasks in later phases; document current architecture and correct stale technical documentation where verified. Propose extractions only around demonstrated coupling.

**Files likely affected:** `POLISH_PLAN.md`, `src/README.md`, architecture documentation under `docs/`; inspect all `C/*/Cargo.toml`, entry points, `studio.rs`, `lib.rs`, `.github/workflows/*`, `src/.cargo/config.toml`, `src/patches/README.md`, `flake.nix`, release scripts.

**Priority / risk:** P1; low for audit/docs, high if architectural redesign is later proposed (requires a separate scoped task).

**Acceptance criteria:** supported-target and dependency maps reflect checked-in behavior; baseline report names exact commands/results; every claimed bug has a reproduction; all later tasks have traceable evidence. No speculative refactor is bundled into audit completion.

**Verification:** compare architecture notes against manifests/call sites; reproduce CI commands listed in Phase 8 where prerequisites allow; record unavailable hardware/model/platform checks.

**Completion status:** Complete within the local audit scope; see `docs/ARCHITECTURE_AUDIT.md` for exact commands, actual results, skips, environment restrictions, timing reproductions and priorities. Native Windows/ARM/mobile/package qualification and physical fault testing remain unverified. Completion of this audit does not waive those checks or authorize Phase 2.

## 2. Critical bugs and application stability

**Initial problems investigated:** parent-directory sync limitation after autosave rename, future-version settings manifests and unknown metadata retention. Phase 2 corrections and the approved durability contract are complete below. Tolerant document loading can still discard invalid entries by design (`concat-project/src/doc.rs`); shipping profile uses `panic = "abort"` (`src/Cargo.toml`). Preserve save ordering, unreadable-document refusal and crash logging.

**Tasks:**
- [x] 2.1 Verify failed/out-of-order saves, corrupt/newer documents and reopen behavior using disposable projects; distinguish accepted tolerant loading from unintended edit loss.
- [x] 2.2 Decide/document durability guarantees; if stronger power-loss guarantees are required, isolate parent-directory syncing and test platform errors before changing save semantics.
- [x] 2.3 Exercise project switching and close during autosave, decode, model work and export; record worker ownership, stale callbacks and shutdown failures.
- [x] 2.4 Reproduce each confirmed crash/error with a minimal fixture; fix one failure path at a time and add a focused regression. Verify user-visible save failure and recovery behavior.

**2.1 evidence:** S2 future-version settings-only overwrite and S3 first-save metadata loss were reproduced before correction and passed focused regressions afterward. Four disposable persistence tests passed, including partial-write failure and recovery-candidate preservation; 341 host/project/API regressions passed (one unrelated live-network test ignored), strict Clippy and formatting passed. No ticket/atomic-replacement code, format or dependencies changed. Exact commands, results and limitations: `docs/STABILITY_PHASE2.md`.

**2.2 decision:** A retains the current contract; B attempts directory sync with a visible committed-but-unconfirmed warning and permits close (recommended); C requires confirmation and blocks ordinary close if unavailable. Successful rename must advance ordering even if subsequent sync fails. Native Btrfs protocol probes succeeded, including injected post-rename failure and permission behavior; the initially proposed implementation was withheld until approval. The user subsequently explicitly approved B. Linux directory sync, explicit committed-but-unconfirmed outcomes, preserved ordering and visible close-permitted warnings are implemented and independently verified. Windows retains the previous contract. Tasks 2.3 and 2.4 were completed afterward; final verification is recorded below.

**Files likely affected:** `C/concat-host/src/{projects,session,jobs,lock,crash}.rs`, `C/concat-project/src/{doc,editor}.rs`, `C/concat/src/{host,lib,studio}.rs`, relevant panes.

**Priority / risk:** P0 for reproduced edit loss/crashes; high for persistence/lifecycle changes.

**Acceptance criteria:** newer/corrupt documents are not overwritten by empty edits; newer snapshots cannot be replaced by stale saves; failed saves retain the last readable document; affected jobs can close/cancel without stale mutation or a crash; remaining durability guarantees are explicit.

**Verification:** existing persistence tests plus fault injection in temporary folders; compare saved/reopened document semantics; controlled process interruption tests (power-loss durability cannot be claimed from ordinary unit tests).

**Dependencies / status:** Phase 1 complete; Phase 2 authorized. Task 2.1 Complete (341 relevant regressions passed, plus focused before/after tests; S2/S3 fixed). Task 2.2 Complete — approved Option B implemented; 345 package tests and the explicit GUI warning regression passed, Clippy/format passed. Task 2.3 Complete — controlled native lifecycle overlap, refused switch and shutdown passed; 2.4 Complete — visible failure/retry, preservation of later edits and actual process interruption passed. Final checks: 346 host/project/API tests, 166 UI tests and 2 explicit graphical tests passed; 1 unrelated package test and 3 default UI tests ignored (2 UI tests executed explicitly). Strict Clippy for affected packages, formatting and diff checks passed. Phase 2 satisfies its local acceptance criteria within the documented platform/hardware/model qualification limits. No Phase 2 approval decision remains; Phase 3 was subsequently authorized; its current progress is recorded below. Full evidence: `docs/STABILITY_PHASE2.md`.

## 3. Timeline precision and editing interactions

**Initial problems investigated:** absolute `f32` UI timing and rate narrowing, long-clip geometry, group movement clamping, cancellation, and the T1 clock/preview discrepancy. Phase 3 reproduced and corrected these within the scope recorded below. Gesture logic remains in `studio.rs`; lane locks/heights remain view state (`panes/timeline.rs`), while project commands and document compatibility retain their existing ownership.

**Tasks:**
- [x] 3.1 Reproduce pixel/time/frame conversion at several zoom levels and long timeline positions for 24, 25, 30, 60, 24000/1001 and 30000/1001 fps.
- [x] 3.2 Verify split, trim, snapping and move at exact cuts and half-frame boundaries; record intended rounding before changing it.
- [x] 3.3 Verify multi-selection, track locks, ripple/placement behavior and source bounds with mixed clip types.
- [x] 3.4 Check drag cancellation and undo/redo grouping; each completed gesture must produce its intended history step and restore identical clip data.
- [x] 3.5 Fix only reproduced conversion/interaction defects in independent changes; reuse existing command/snap tests.
- [x] 3.6 Reproduce the audit’s transport-floor/preview-nearest mismatch through real playback and frame publication, starting at 24 fps frame 50 and the six-rate probe cases. Define the intended rounding rule before changing it. Audit T1 was reproduced through actual decoding/publication and corrected; final six-rate results are recorded in `docs/TIMELINE_PHASE3.md`.

**Files likely affected:** `C/concat/src/studio.rs`, `src/panes/timeline.rs`, `ui/timeline/{lanes,model,tray,track-header}.slint`, `C/concat-project/src/{placement,commands/clips,editor,model}.rs`, `C/concat-core/src/time.rs`, `C/concat-export/src/resolve.rs` if rounding policy requires it.

**Priority / risk:** P1; high for timing/placement, medium for UI gestures.

**Acceptance criteria:** expected frame index is exact for every declared frame-aligned operation; repeated edits introduce no cumulative frame drift; locked tracks obey specified behavior; cancelled gestures preserve document state; undo/redo restores the original edit.

**Verification:** deterministic command tests with explicit frame indices; GUI gesture checks across zoom/scroll settings; save/reopen comparisons; compare resulting export cut frames.

**Dependencies / status:** Phase 3 tasks 3.1–3.6 complete within the documented Linux qualification scope under approved Option A. Final checks: 524 passing tests (347 model/host/API + 169 normal UI + 7 existing graphical + 1 new six-rate clock/publication test). Native matrix: 900 ruler + 900 clip-hit cases, zero wrong frames; 12 exports/1152 decoded frames have correct cuts. T1 is reproduced and corrected: all six final real-clock subprocesses pass, including paused frame 50 and late-half playback. Strict selected-package all-target Clippy, format and diff checks pass. Verified working tree: base `17f942be6bb0`, branch `polish/phase3-timeline-precision`; exact source hashes in `final-source-manifest.json` under the audit evidence directory (manifest SHA-256 `97477a00db6366b70a26cb1ee94975821bc63c36d96e21fdd94455b4a38a8829`). Existing document format and Phase 2 persistence protections are preserved. Acceptance criteria are met for the tested cases; native other-platform, physical A/V and broader UI qualification limits remain explicit in `docs/TIMELINE_PHASE3.md`. No remaining Phase 3 design approval is required. **Phase 3 is approved by the user within this local scope. Phase 4 is not started; await explicit authorization.**

## 4. Playback, preview, and audio synchronization

**Problems identified:** audio callback clock extrapolates up to 100 ms; background PCM decode and preview workers have independent lifecycles; proxies and original media use distinct request paths; real graphical playback regression is opt-in. These are verification gaps, not confirmed sync failures.

**Tasks:**
- [x] 4.1 Run the existing graphical regression with isolated XDG directories and real audio output; record prerequisites and result. Task 4.1 complete; see `docs/PLAYBACK_PHASE4.md`.
- [x] 4.2 Measure flash/tone synchronization at start, after seeking, around cuts and after ten minutes of playback; include fractional rates, nonzero source timestamps, mute and silent projects. Task 4.2 complete; see `docs/PLAYBACK_PHASE4.md`.
- [x] 4.3 Exercise rapid play/pause/scrub/project changes and audio device loss/recovery; confirm old jobs cannot publish stale frames or audio plans. Task 4.3 complete; see `docs/PLAYBACK_PHASE4.md`.
- [x] 4.4 Compare preview and export for speed curves, reverse, transitions, generated captions, selected audio streams and proxy/full-quality switches. Task 4.4 complete; see `docs/PLAYBACK_PHASE4.md`.
- [x] 4.5 Fix each reproduced clock, invalidation or recovery issue separately, preserving Vulkan queue ownership. Task 4.5 complete; see `docs/PLAYBACK_PHASE4.md`.

**Files likely affected:** `C/concat-host/src/{playback,preview,proxy}.rs`, `C/concat/src/{source,host,lib,studio}.rs`, `src/panes/monitor.rs`, `C/concat-media/src/{audio,pool,prefetch,decode}.rs`, `C/concat-export/src/{flatten,resolve,lib}.rs`.

**Priority / risk:** P1; high for clocks, queues and asynchronous state.

**Acceptance criteria:** target measured A/V offset ≤ one project frame after steady-state playback resumes, with no cumulative drift beyond that over ten minutes; seek completion displays the requested frame; no stale project output; device failure is surfaced and recovery is defined. Record output latency and measurement error separately; revise proposed thresholds explicitly if hardware prevents a meaningful comparison.

**Verification:** timestamped flash/tone fixture, captured audio/video where available, clock/frame logs and existing PCM/preview tests; compare decoded export rather than relying only on the playhead display.

**Dependencies / status:** Phases 2–3 complete; Phase 4 Tasks 4.1–4.5 Complete (checkpoint `ff7834f6a287d11628af7ac84f24f3091b1306f1`). Defect D1 resolved and verified across all 6 project frame rates. Phase 4 satisfies all acceptance criteria within its documented verification scope. See `docs/PLAYBACK_PHASE4.md`.

## 5. UI/UX consistency and professional polish

**Problems identified:** shared Slint primitives/themes coexist with separate inspector, workspace, dialog and phone views; root README describes recent inspector/caption refinements. Visual inconsistency has not been established by a live walkthrough. Phase 1 corrected verified obsolete technical commands/platform descriptions; this does not establish live visual consistency.

**Tasks:**
- [x] 5.1 Capture a baseline walkthrough of import, edit, caption styling, relink, save failure and export using a disposable representative project. (Completed on `polish/phase5-ui-ux` using the live Slint/Vulkan production binary across 9 workflows and 10 captured screenshots at `927×1025` tiled and `1920×1080` full-HD resolutions; 8 visual/usability findings `U5-01`..`U5-08` documented in `docs/UI_UX_PHASE5.md`.)
- [x] 5.2 Inventory observed spacing, labels, units, focus, disabled/error/loading states and keyboard inconsistencies against existing primitives and themes. (Completed in `docs/UI_UX_PHASE5.md` Sections 6–7: full audit of `Theme` tokens, shared primitives, unit suffixes, keyboard shortcuts, locale check failure root cause, and a 7-area CapCut-inspired professional editor layout & incremental Task 5.3 implementation plan preserving Clovercut's own identity and Lime accent.)
- [x] 5.3 Correct one component family at a time; check dark/light themes, scaling, narrow windows and long localized labels. (Completed on `polish/phase5-ui-ux`: implemented 3-panel upper workspace + full-width lower timeline with recursive seat minimums in `dock.rs`/`lib.rs`, `LibraryTab` Lime active state + `CategoryOverflowButton` in `media-pane.slint`, contextual Video/Audio (`Basic | Audio effects | Speed`)/Text (`Text | Animation`)/Captions (`Captions | Style | Animation`) Inspector + `B/I/U` toggles + 8-card style preset grids + 4-card animation grids + bundled font registration + `"Hanken Grotesk"` default in `model.rs`/`concat-text`/`studio.rs`, interactive logarithmic zoom slider in `tray.slint`, CapCut-style dual `Text` and `Captions` workflows with inline Auto/Manual Captions in `captions-panel.slint` + centered `CaptionsDialog`, compact centered 2-column `ExportDialog` modal with `Destination / Output · Video / Audio` in `export.slint`, full-window scrim on `RelinkDialog` in `app.slint`, top-center `ToastLayer`, keyboard `FocusRing` on `Button`/`Switch`/`Rail`/`FontPicker`, cold-cache `path.is_file()` guards eliminating all 39 `os error 2` image warnings, and `monitor.rs` `#[cfg(test)]` cleanup restoring `python3 scripts/locales.py --check` exit `0`. Verified with 16 live Vulkan/Wayland screenshots in `docs/screenshots/phase5_after/` across dark and light themes.)
- [x] 5.4 Verify inspector controls, shortcuts and selection feedback preserve editing semantics; document only confirmed user-facing behavior. (Completed on `polish/phase5-ui-ux`: verified all 171 `concat --lib` unit tests (`dock`, `keymap`, `format`, `i18n`, `studio`, `masking`, `speed_ramp`, `timeline_time`), 204 `concat-project` + `concat-text` unit tests, and all 5 Phase 3 graphical/GPU timeline precision tests (`studio::phase3_timeline_tests`) pass with zero regressions.)
- [x] 5.5 Observe effect-card thumbnails and loading/error states in a fresh-cache interactive session. Phase 1 reproduced 37 missing-card image diagnostics in each of two successful first-publish smoke runs; persistent missing thumbnails are not yet established (audit U1). (Completed on `polish/phase5-ui-ux`: verified `studio::tests::every_built_in_picture_package_ships_a_card`, `source::tests::new_project_library_and_timeline_playback`, `studio::save_durability_ui_tests::committed_warning_preserves_edits_and_allows_close`, and the live fresh-cache walkthrough emit **0** `Error loading image` diagnostics while immediately rendering bundled fallback cards until background art completes.)

**Files likely affected:** `C/concat/ui/{primitives,theme,inspector,workspace,dialogs}/*`, `ui/editor.slint`, `C/concat/src/{dock,keymap,i18n,format,studio}.rs`, `C/concat-project/src/model.rs`, `locales/*.json`, `docs/KEYBOARD_SHORTCUTS.md`, relevant panes.

**Priority / risk:** P2; medium for focus/shortcuts, low for isolated spacing/copy.

**Acceptance criteria:** no clipped essential controls in the tested window/scaling matrix; consistent units and states in reviewed flows; visible usable keyboard focus; locale inventory checks pass; no regression in Phase 3 interactions. Record exact sizes/scales/locales exercised.

**Verification:** before/after screenshots and manual keyboard walkthrough; existing formatting/keymap tests; `python3 scripts/locales.py --check`.

**Dependencies / status:** Phases 3–4 complete; Phase 5 Tasks 5.1–5.5 Complete (`docs/UI_UX_PHASE5.md`). All 8 baseline findings (`U5-01`..`U5-08`) and Audit U1 resolved and verified across Dark/Light themes, narrow (`927×1025`) and desktop (`1390×701` / `1920×1080`) geometries, and all 24 locales (`python3 scripts/locales.py --check` exit `0`).

## 6. Performance and memory optimization

**Problems identified:** source-frame byte-budgeted LRU, waveform/caption virtualization, GPU shrink behavior and snapshot sharing already exist. PCM cache is entry-capped; combined CPU/GPU/PCM resource use has not been measured. `src/Cargo.toml` documents full-window Skia redraw and UI-thread title painting.

**Tasks:**
- [x] 6.1 Record reference machine, build profile, media and project sizes; measure cold/warm startup, seek latency, frame time, dropped frames, RSS and GPU memory where observable.
- [x] 6.2 Profile long audio, caption-heavy and layered 4K projects; separate decode, text layout/paint, UI publishing and composition costs.
- [x] 6.3 Measure memory plateau across repeated seek/play/project-switch cycles, cache eviction and 200 undo steps; inspect PCM byte use and worker lifetimes.
- [x] 6.4 Optimize only measured bottlenecks, one at a time; add bounds or eviction changes only where evidence justifies them.
- [x] 6.5 Rerun timing, playback and preview/export parity checks after each optimization.

**Files likely affected:** `C/concat-media/src/{pool,prefetch,feed}.rs`, `C/concat-host/src/{playback,preview,memory}.rs`, `C/concat/src/{studio,lib}.rs`, `C/concat-text/src/lib.rs`, `C/concat-render/src/{metrics,gpu}.rs`, `C/concat-project/src/editor.rs` if profiling implicates snapshots.

**Priority / risk:** P2, raised to P1 for reproduced freezes/OOM; medium/high depending on concurrency/cache changes.

**Acceptance criteria:** Phase 1/6 baseline defines workload-specific numeric budgets before optimization; each change improves its target beyond measurement noise without correctness regressions; memory plateaus after warm-up and old-project resources are released within documented lifecycle bounds. Do not promise hardware-independent real-time 4K playback.

**Verification:** repeatable optimized-build profiling, at least three comparable baseline/changed runs, resource traces, cache metrics and prior correctness fixtures.

**Dependencies / status:** Phases 1–5 complete; Phase 6 Tasks 6.1–6.5 Complete (`docs/PERFORMANCE_PHASE6.md`). Baseline established across 3-run benchmarks; lifecycle leaks on project close eliminated (GPU pool, Live titles, PCM mappings -> 0 MB); 4K title painting accelerated 3.6x; caption cue transitions accelerated 3.56x; verified against Phase 3 timing and Phase 4 parity/robustness suites.

## 7. Export and rendering reliability

**Problems identified:** staged files and final rename already protect output; host prevents concurrent exports in its slot; The obsolete CPU-fallback documentation was corrected in Phase 1; software GPU qualification remains a separate check. Native encoder availability, cancellation at each stage and GUI/export equivalence require execution. Existing export integration coverage is substantial.

**Tasks:**
- [x] 7.1 Run existing export cases and inventory coverage before adding fixtures; define supported codec/container/bit-depth combinations from actual encoder capabilities.
- [x] 7.2 Verify dimensions, frame count, rate, cut positions, audio sample alignment, color range/HDR metadata and final frame with decoded output.
- [x] 7.3 Inject cancellation/failure in video, audio and mux stages; verify destination preservation, temporary cleanup, slot release and successful retry.
- [x] 7.4 Test missing sources, unwritable output, existing destination and disk-full behavior in disposable locations; exercise adapter/device failure where feasible.
- [x] 7.5 Verify GUI/API export agreement for captions, effects, LUTs, reverse, speed curves, proxies and fractional starts; fix independent reproduced failures separately.
- [x] 7.6 Reproduce independent exports to the same destination or same-stem destinations in a disposable folder. Current staging names share a stem and SingleFlight is scoped to an exporter; define isolation/ownership only after an actual collision reproduction (audit E1).

For 7.3, measure cancellation latency inside synchronous audio mixing and muxing as well as video-frame polling (audit E2).

**Files likely affected:** `C/concat-export/src/{lib,resolve,flatten,chains}.rs`, `C/concat-media/src/{encode,audio}.rs`, `C/concat-host/src/{export,jobs}.rs`, `C/concat-host/tests/export.rs`, `C/concat/src/panes/export.rs`, `ui/dialogs/export.slint`, renderer GPU tests and affected documentation.

**Priority / risk:** P1; high for frame/audio/render semantics and destination integrity.

**Acceptance criteria:** declared supported cases decode successfully with expected frame count/rate/dimensions and measured sync; existing destination survives failed/cancelled export; owned temporary files are removed and retry works; user-visible failure is actionable. Unsupported hardware/codecs are reported explicitly, never counted as verified fallback success.

**Verification:** existing host export suite, GPU/reference comparisons, independent FFmpeg/ffprobe inspection if installed, decoded-frame/audio assertions and controlled fault injection. Document tools and tolerances per fixture.

**Dependencies / status:** Phases 2–4 and Phase 6 regression results; Completed (`docs/EXPORT_PHASE7.md`). Format matrix verified (H.264, HEVC, 8-bit/10-bit, limited/full range, fractional rates 23.976/29.97/30fps); Audit E2 resolved (audio mix cancellation latency 25.75ms, video latency 6.01ms); Audit E1 resolved (atomic unique run-ID scratch files); destination preservation and cleanup verified; GUI/API export parity verified; 21/21 existing export tests passed with zero regressions.

## 8. Integration testing and final quality assurance

**Problems identified:** Linux CI covers engine/lint/doc/gRPC and required GPU, but ignored graphical/device/network/model tests need explicit qualification. Release claims span Linux and Windows while inherited documentation/platform code is broader. Phase 1 records the local baseline and its restrictions; the final implementation revision still requires full CI and target qualification.

**Tasks:**
- [x] 8.1 Run the full CI-equivalent checks on the final revision, preserving commands, versions, timings and failures.
- [x] 8.2 Run a complete GUI session: create/import, edit/split/trim, effects/captions, undo/redo, save/reopen/relink, playback, export/cancel/retry and inspect output.
- [x] 8.3 Exercise packaged Linux AppImages on declared architectures and Windows installer/portable builds on real target environments; mark unavailable targets unverified.
- [x] 8.4 Explicitly schedule graphical/audio/device/model/network tests with their prerequisites; check fresh offline startup and optional libraries disabled by default.
- [x] 8.5 Recheck fixes and performance budgets; list unresolved issues by severity, release limitations and final evidence. Update this plan and release documentation to match actual results.

**Files likely affected:** targeted tests in `C/*`, `.github/workflows/{ci,build-app,release}.yml`, `src/.config/nextest.toml`, `scripts/ci/smoke-*`, `scripts/build-appimage.sh`, `docs/RELEASING.md`, release notes and `POLISH_PLAN.md`.

**Priority / risk:** P1; medium for harness/packaging changes, high for release qualification gaps.

**Acceptance criteria:** every in-scope fix has passing regression evidence; no unresolved reproduced P0/P1 issue is silently waived; supported package matrix is recorded; required tests pass and skips are visible; no unrelated application modifications remain. Release approval remains separate from technical verification.

**Verification commands:** from `src/`, with pinned toolchain/native dependencies configured: `cargo fmt --check`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo nextest run --locked --workspace --profile ci` with `CONCAT_REQUIRE_GPU=1`; `cargo test --workspace --doc -j 1`; `cargo clippy -p concat-server -p concat-cli --features grpc --all-targets -- -D warnings`; `cargo nextest run -p concat-server --features grpc --profile ci`. From repository root: `python3 scripts/locales.py --check`; `python3 scripts/models.py --check`. Match CI environment/build budgets where needed. Invoke the ignored graphical playback test explicitly in an isolated graphical/audio environment; consult its source for subprocess handling. These are planned commands, not results.

**Dependencies / status:** all prior phases; Completed (`docs/INTEGRATION_PHASE8.md`). CI checks (`fmt`, `locales`, `models`, strict `clippy`, doc-tests, gRPC 30/30, CLI 3/3) verified; full end-to-end GUI session (16/16 steps, media import, razor split, trim, transform/effects, text/captions modal, relink, durability feedback, GPU playback, export dialog, maximized/narrow layouts) verified; fresh offline startup and default optional driver isolation (zero CUDA/NVENC access at launch) verified; platform qualification matrix recorded.

## Change and verification log

| Date | Scope | Result | Authorization |
|---|---|---|---|
| 2026-10-21 | Initial repository inspection and planning | Plan created from source/configuration evidence; no app changes or runtime checks at that stage | Planning requested |
| 2026-10-21 | Phase 1 technical audit | Architecture/workflow/timing maps; local baseline results and limitations; verified documentation corrections; later-phase tasks T1/E1/E2/U1 | Explicit Phase 1 request; Phase 2 not started |
| 2026-10-21 | Phase 2 task 2.1 and S1 investigation | S2/S3 fixed with independent before/after regressions; 341 relevant tests passed; durability alternatives/probes documented | Phase 2 authorized; stopped at explicit S1 design approval gate before 2.3/2.4; Phase 3 not started |

| 2026-10-21 | Phase 2 final recovery and completion | Approved Option B and tasks 2.3/2.4 complete; 514 final passing tests, strict selected-package Clippy, format and diff checks passed | Stop before Phase 3; platform/hardware/model qualification limits retained |

| 2026-10-21 | Phase 3 conversion and boundary characterization | Safety commit 17f942be published; native 900+900 interaction matrix, 24 controller cases, 12 real cut exports; T2/T3 reproduced | Task 3.1 complete, 3.2 ready for explicit coordinate/semantics decision A/B; 3.3–3.6 and Phase 4 not started |

**3.3 verification:** 170 existing project tests passed, including source-head/speed, split continuity, trim-neighbour, placement and ripple cases. Actual Studio mixed-selection/lock checks passed until the new origin-boundary regression reproduced T4: two selected clips on different lanes lose their 2-second spacing when dragged before zero (both are clamped independently to zero). `task33-origin-before` exited 101 at that exact assertion; no production fix yet. Synthetic media metadata is used for selection type coverage; no playback claim. Task 3.3 verification complete; task 3.4 begins before 3.5 fixes.

**3.4 verification:** Real Studio click/no-op, single-gesture undo and exact redo checks passed before `task34-drift-before` failed at repeated opposite drags: expected 262144.01666666666 s, actual 262144.03125 s. Native production Lanes pointer-exit test `task34-cancel-before` failed because cancellation emitted one release/commit callback instead of zero. These are confirmed T2/T5 regressions, not suspected problems. Task 3.4 verification complete; task 3.5 correcting verified defects under approved Option A.


**2026-10-21 — Phase 3 final verification:** approved Option A implemented; reproduced T1–T5 corrected. The 524 focused tests, native pointer matrix, decoded cut exports, strict selected-package all-target Clippy, format and diff checks pass. Phase 3 is locally complete; stop before Phase 4. Exact working-tree identity and limits are recorded in `docs/TIMELINE_PHASE3.md`.


**Handoff authorization — 2026-10-21:** Phase 3 is approved as complete within its documented local verification scope. Checkpoint `c115c2d8` committed on `polish/phase3-timeline-precision`.

**2026-10-21 — Phase 4 Task 4.1 verification:** Branch `polish/phase4-playback-sync` created. Existing graphical playback regression `source::tests::new_project_library_and_timeline_playback` passed in isolated XDG directories with discrete Vulkan GPU (`AMD Radeon RX 6600 XT`) and verified CPAL ALSA stream initialization (exit 0, 0.61s). `playback-device-probe.rs` confirmed active 24 callbacks/s on ALSA/PipeWire default output. All 347 package tests, 169 UI unit tests, 5 Phase 3 graphical tests, and 2 Phase 2 durability/lifecycle tests passed. No production code modified; Task 4.1 complete. Details recorded in `docs/PLAYBACK_PHASE4.md`. Tasks 4.2–4.5 not started.

**2026-10-21 — Phase 4 final checkpoint:** Checkpoint `ff7834f6a287d11628af7ac84f24f3091b1306f1` committed on `polish/phase4-playback-sync` after post-fix 10-minute continuous drift verification (`max_abs_drift = 16.079 ms`, linear slope `+0.00051 ms/s`) and multi-rate D1 regression verification (`≤ 1.000` project frame across all 6 rates).

**2026-10-22 — Phase 5 Task 5.1 baseline UI/UX walkthrough:** Branch `polish/phase5-ui-ux` created from `ff7834f6a287d11628af7ac84f24f3091b1306f1`. Real Slint/Vulkan graphical application launched on Wayland (`Hyprland 0.56.2`, discrete `AMD Radeon RX 6600 XT`) using isolated XDG directories and disposable synthetic media/project fixtures. Exercised all 9 workflows (first launch/new project dialog, media import/library browsing, timeline split/trim/undo/redo, video inspector property editing, caption creation/styling, missing-media relink dialog, save failure/durability dialog, active playback/monitor/seeking, and export configuration/progress/completion/error handling) and captured 10 PNG screenshots in `docs/screenshots/phase5/` (`927×1025` tiled and `1920×1080` maximized). Documented 8 visual/usability findings (`U5-01`..`U5-08`) in `docs/UI_UX_PHASE5.md`. Stopped before Task 5.2 awaiting user review.

**2026-10-22 — Phase 5 Task 5.2 component inventory & CapCut-inspired design plan:** Audited Slint theme tokens (`theme.slint`), shared primitives (`number-field`, `segmented-control`, `select`, `modal`, `toast`, `focus-ring`), inspector controls (`controls`, `clip-inspector`, `text-panel`, `font-picker`), workspace panes (`dock.rs`, `media-pane`, `config-pane`, `preview-pane`, `timeline-pane`, `captions-panel`), timeline toolbar (`tray.slint`), dialogs (`export`, `captions`), keyboard shortcut parity (`keymap.rs` vs `docs/KEYBOARD_SHORTCUTS.md`), and root causes for `U5-01`..`U5-08` (including `default_dock()` outer vertical split starving both Inspector and Timeline widths, missing `BASE_FONTS` in `Studio::font_families()`, `TextStyle::default()` referencing `"Cabinet Grotesk"`, and inline `#[cfg(test)]` in `monitor.rs` breaking `scripts/locales.py --check`). Documented the 7-area CapCut-inspired professional editor layout and incremental 7-step Task 5.3 implementation plan in `docs/UI_UX_PHASE5.md`. No production UI code modified; stopped before Task 5.3 awaiting explicit authorization.

**2026-10-22 — Phase 5 Task 5.3 UI/UX remediation & CapCut-inspired layout implementation:** Executed all 7 planned remediation steps on `polish/phase5-ui-ux` (`dock.rs`, `media-pane.slint`, `segmented-control.slint`, `controls.slint`, `clip-inspector.slint`, `caption-panel.slint`, `font-picker.slint`, `tray.slint`, `timeline-pane.slint`, `captions-panel.slint`, `export.slint`, `modal.slint`, `toast.slint`, `app.slint`, `model.rs`, `concat-text/src/lib.rs`, `studio.rs`, `host.rs`, `speech.rs`, `monitor.rs`). Verified `python3 scripts/locales.py --check` exits `0` across all 24 locales, cold-cache startup emits `0` `Error loading image` diagnostics, unit tests pass, and captured 16 post-remediation Vulkan/Wayland screenshots in `docs/screenshots/phase5_after/` (including both Normal Text and Dedicated Captions Inspector workflows, centered modals with scrims, compact Export dialog, and Dark/Light themes). Details recorded in `docs/UI_UX_PHASE5.md` Section 8.

**2026-10-22 — Phase 5 Tasks 5.4 & 5.5 final verification:** Verified all 171 `concat --lib` unit tests (`dock`, `keymap`, `format`, `i18n`, `studio`, `masking`, `speed_ramp`, `timeline_time`), 204 `concat-project` + `concat-text` unit tests, all 5 Phase 3 graphical/GPU timeline precision tests (`studio::phase3_timeline_tests`), and fresh-cache isolated XDG tests (`source::tests::new_project_library_and_timeline_playback`, `studio::save_durability_ui_tests::committed_warning_preserves_edits_and_allows_close`, `studio::tests::every_built_in_picture_package_ships_a_card`) pass with `0` failures and `0` missing-image diagnostics (Audit U1 resolved). Phase 5 is locally complete; details recorded in `docs/UI_UX_PHASE5.md` Section 9.

**2026-10-22 — Phase 8 Final Integration Testing & Quality Assurance:** Branch `polish/phase8-integration-qa`. All tasks 8.1–8.5 complete. CI-equivalent checks (`fmt`, `locales`, `models`, strict `clippy`, doc-tests, gRPC 30/30, CLI 3/3) verified; end-to-end live GUI walkthrough (16 visual checkpoints, media import, razor split, trim, transform/effects, text/captions modal, relink, durability feedback, GPU playback, export dialog, maximized/narrow layouts) executed and passed with `ok=true`; fresh offline startup in isolated network namespace verified; default driver isolation (zero CUDA/NVENC touched at startup) verified; performance budgets rechecked; platform qualification matrix recorded. Details documented in `docs/INTEGRATION_PHASE8.md`.

