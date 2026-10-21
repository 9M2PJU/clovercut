# Clovercut agent handoff

Prepared 2026-10-21 for Google Antigravity. This is a documentation handoff, not a development or release authorization.

## Start here and authorization

The user explicitly approved Phase 3 as complete within its documented local verification scope. Phases 1–3 are complete in that scope. **Do not start Phase 4 without the user's separate explicit authorization.** No commit, push, reset, clean, staging or unrelated code change was performed for this handoff. Those Git operations must not be executed without explicit approval; proposed checkpoint commands below are for review only.

Read, in order: `AGENTS.md`, this document, `POLISH_PLAN.md`, `docs/TIMELINE_PHASE3.md`, `docs/STABILITY_PHASE2.md`, and `docs/ARCHITECTURE_AUDIT.md`. The architecture audit and early report checkpoints describe historical baselines; final phase acceptance and the current code supersede their old status statements. No Codex session, proprietary agent API or shell history is needed to continue.

## Repository and Git state

- Repository on this machine: `/home/turotulco/Projetos/clovercut`. `/home/turotulco/Work/clovercut-phase3-timeline` is an evidence/artifact directory, **not** a Git checkout.
- Current branch: `polish/phase3-timeline-precision`.
- HEAD: `17f942be6bb0a8373b2e09e4d90f930d5f128353` (`checkpoint: completed architecture audit and phase 2 stability`).
- HEAD contains the Phase 1/2 checkpoint. Phase 3 implementation, tests and the handoff documents remain uncommitted. There is no Phase 3 commit to fetch on a new machine.
- The safety branch `safety/phase2-complete-2026-10-21` at HEAD was previously pushed and independently verified on origin. Remote state was not queried or changed during this handoff. Do not mistake that historical checkpoint for the uncommitted Phase 3 implementation.
- Index was empty at handoff review. Preserve every modified and untracked file listed in the inventory at the end of this document.
- All 25 modified/new Phase 3 source/probe files match the final verified source manifest: SHA-256 `97477a00db6366b70a26cb1ee94975821bc63c36d96e21fdd94455b4a38a8829`. The manifest is at `/home/turotulco/Work/clovercut-phase3-timeline/final-source-manifest.json`; documentation-only handoff edits do not change that source identity.

If continuing on another machine, obtain an explicitly approved checkpoint or an approved transfer that includes untracked files. A plain clone of HEAD or a tracked-only diff loses Phase 3. Transfer local native configuration privately and separately; never put secrets in a patch, repository or this handoff.

## Architecture and ownership

Clovercut inherited Concat's crate/API names; `concat` is the GUI package and `clovercut` is its executable. The Rust workspace is `src/`.

| Layer | Ownership |
|---|---|
| `concat-core` | std-only rational time, exact frame rates, timeline/render primitives. Preserve dependency direction. |
| `concat-project` | f64 document times, commands, migration, placement, snapshot undo/redo and unknown-field retention. |
| `concat-media` | Linked FFmpeg decode/encode, audio graphs, reader pool, caches and prefetch. |
| `concat-render` / `concat-export` | GPU compositing; flatten/resolve, preview frame planning, video/audio export and staged final replacement. |
| `concat-host` | Session/project persistence, playback/device clock, monitor, jobs, proxies, titles, model services and ownership. |
| `concat` | Slint views, panes and callback wiring; `src/studio.rs` is the controller, `src/host.rs` bridges worker/event-loop state. |
| `concat-effects`, `concat-text`, `concat-vision`, `concat-speech` | Effects, title shaping/rasterization and optional inference/speech services. |
| `concat-api/server/cli/mcp` | Shared automation vocabulary, JSON-RPC/gRPC adapters and command-line/MCP entry points. |
| `concat-android` | Inherited mobile entry point; source presence is not supported-release qualification. |

Paths in the table are under `src/crates/`. `Shell` holds the UI-thread `Studio` and persistent Slint models. `Session` owns the authoritative editor/project; an `echo` previews a gesture. Commands commit edits and snapshots restore history. Worker delivery uses project epochs to reject abandoned-project results. Monitor requests have one in flight and coalesce demand; same-project older output may land before the newest wanted frame, a behavior to measure rather than assume a fixed persistent-staleness defect.

Rendering remains GPU-backed, including software adapters; the CPU reference compositor is test-only. Shared-device texture composition stays on the UI thread. `Monitor::off_thread` must not introduce concurrent raw Vulkan queue submissions. Exports preserve destination files by staging and replacing only on success; SingleFlight ownership must remain intact.

## Completed phases and important regressions

| Phase | Result and evidence |
|---|---|
| 1 — audit | Architecture/dependency/platform maps, native baseline, timing characterization and corrected stale technical docs. `docs/ARCHITECTURE_AUDIT.md`; no speculative application rewrite. |
| 2 — stability | S2 rejects future-version settings-only documents before a save can overwrite them; S3 preserves unknown metadata on first/later saves. Approved S1 Option B implements Linux directory sync and explicit committed-but-unconfirmed outcomes. Controlled switch/shutdown and UI durability/lifecycle regressions passed. Final Phase 2 count: 514 passing focused tests; not an additional count to add to Phase 3. `docs/STABILITY_PHASE2.md`. |
| 3 — precision | Approved Option A preserves continuous editing with a precise coordinate boundary. T1–T5 reproduced and corrected. Final count: 524 passing focused tests, native pointer matrix and decoded export checks; strict selected-package all-target Clippy, format and diff checks passed. User approved this local scope. `docs/TIMELINE_PHASE3.md`. |

Phase 3 changes relative to HEAD:

| Finding | Correction / regression |
|---|---|
| T2: absolute f32 times lose neighboring frames at long positions; repeated inverse drags drift | f64 authoritative timing and viewport-local Slint coordinates, 1024-second origins, exact-rate frame stepping. `timeline_time.rs`, `panes/timeline.rs`, `studio.rs`, `lib.rs`, desktop/phone timeline interfaces. 900 ruler + 900 real clip-hit checks, zero wrong frames; 100 inverse drag pairs and exact history restoration pass. |
| T2 geometry: a clip spanning zero to 262144+1/60 s accepted a click beyond its true tail | Bound display geometry to the overscan window and rebase filmstrip/wave/fade views without editing the underlying clip. Native boundary probe and integrated publication regression pass. |
| T3: extra f32 narrowing of project rate makes razor/menu exports differ by one frame | Razor uses the exact project rate; preserve continuous menu/free gestures. Controller 24-case fixture and 12 exports/1152 decoded frames pass. |
| T4: individually clamping a selected group before zero compresses spacing | Apply a common lower-bound shift. Mixed selection, real lock handling and placement/source-bound regressions pass. |
| T5: pointer cancellation emits release/commit | Separate cancellation callback through Lanes→Editor→Studio; discard echo and preserve history. Native pointer exit emits one cancel/zero releases; actual move/trim cancellation restores the full Project. |
| T1: preview-nearest can request the next frame inside the current running-clock interval | In `panes/monitor.rs`, select the containing exact project-frame boundary before clip/title preparation during timeline playback. Keep the audio/raw playhead clock and paused/library policies. New real-clock fixture passes all six rates: 96 moving images and six paused frame-50 checks. |

Small f64 propagation/presentation casts in other panes and `format.rs` are part of Phase 3. They are not unrelated feature changes. No project-format, dependency, lockfile, persistence implementation or exporter implementation changes were made in Phase 3.

Important tests to preserve:

- `src/crates/concat/src/phase3_timeline_tests.rs`: five ignored graphical tests covering cuts/steps/save-reopen, mixed selection/locks/origin clamp, exact drag history/cancellation, long-clip publication, and actual clock/decoded-frame publication. These use disposable child processes; `on_start` suppresses preview in editing-only fixtures, but is disabled in the clock fixture.
- `src/crates/concat/src/timeline_time.rs`: three ordinary helper regressions.
- `src/crates/concat-host/tests/timeline_precision.rs`: twelve actual red/blue cut exports, every one of 1152 decoded frames checked. Handler formulas are independently characterized; this is not an export of the graphical fixture's saved project.
- `docs/audit/relative-timeline-ui-probe.rs` and `timeline-rebase-probe.rs`: real Slint pointer/hit tests and rebase/scroll/cancel checks. `timeline-ui-probe.rs` retains the historical absolute-coordinate characterization; use the relative probe for the corrected protocol.
- Phase 2: host `tests/persistence.rs` (five tests), Session future-version/metadata regressions, and GUI `studio::save_durability_ui_tests::committed_warning_preserves_edits_and_allows_close` / `studio::lifecycle_tests::switch_and_shutdown_with_running_work`.
- Existing `source::tests::new_project_library_and_timeline_playback` passed explicitly in isolated configuration as a compatibility regression; this does not complete Phase 4's audio/device qualification.

The 524 total is 347 package tests + 169 normal UI tests + seven existing graphical fixtures + one new six-rate graphical fixture. Do not count six subprocesses, repeated runs or matrix events as additional unit tests.

## Critical invariants and approved design

1. Phase 2 Option B: save tickets are ordered and atomic rename commits the new snapshot. Advance ordering at rename even if subsequent directory sync fails. Distinguish pre-commit failure from committed-but-unconfirmed durability, show a warning and allow close after a committed save. Preserve the existing Windows contract. Never silently overwrite newer/unreadable documents, erase recovery candidates or roll back to stale bytes.
2. Preserve version-1 f64 documents and unknown fields. Existing tolerant loading can omit malformed/orphan entries; opening does not rewrite the original, and this is not an automatic backup/recovery system.
3. Phase 3 Option A: authoritative edit origins, playhead and scroll stay f64. Convert local Slint values with the **last published** origin. Rebase held drag references when origins change. Only display geometry is sliced. Do not impose Option B's global visual frame grid or a new project-duration cap.
4. Explicit frame picks use exact rational project rate; nearest nonnegative frame, positive exact ties upward. Menu/free move/trim stay continuous. Running timeline playback uses its containing half-open interval, with the existing 1e-6-frame numerical tolerance; paused preview keeps nearest. Audio stays sample-based at 48 kHz and retains its clock.
5. No cumulative drag drift; cancellation changes neither Project nor history. Finished gestures keep intended grouping/exact undo and redo. UI track locks are view state, not an API-level project lock.
6. Preserve UI-thread Slint access, project epochs, GPU queue ownership, transactional exports and deliberate native process-lifetime shutdown (`leave`/Linux `_exit`). Do not replace that shutdown with broad thread joining as an incidental cleanup.
7. Keep the prior continuous Rubber Band pitch-preserving audio processor and its source-clock/cache behavior. Required FFmpeg filters include `rubberband`, `asendcmd`, `asetnsamples`; reverting to piecewise retiming or continuous atempo is not authorized by the handoff.

## Environment and prerequisites

Recorded local verification environment (not a fresh hardware inventory): x86_64 Arch/Omarchy, Ryzen 5 5600X, about 15 GiB RAM + 15 GiB swap, Radeon RX 6600 XT / RADV Vulkan. Rust/Cargo **1.93.1**, pinned by `src/rust-toolchain.toml`; run Cargo inside `src/` because the host's outside-directory default differs. Slint 1.18.1/Skia, wgpu 30 and the lockfile/native patches are significant.

Native prerequisites: FFmpeg 7+ headers and shared development libraries plus the `ffmpeg`/`ffprobe` executables; local verified FFmpeg was 9.0.2. C/C++ toolchain, CMake, pkg-config and libclang for native/bindgen builds; Linux fontconfig/FreeType, GTK3, xkbcommon/GL, ALSA, Vulkan loader and Mesa/hardware drivers. ONNX Runtime 1.28.0 and Sherpa native libraries are required by the desktop feature/dependency set. Protoc is needed for gRPC checks; cargo-nextest for CI-equivalent test commands. Optional inference needs its own models; no model inference/download coverage is implied.

Use `scripts/ci/deps.lock`, `.github/workflows/ci.yml`, `build-app.yml`, `src/patches/README.md`, `src/Cargo.toml`, and `src/README.md` as the executable dependency/profile references. Do not substitute unsupported Whisper backend features. Preserve lockfile pins and environment consistency across checks to avoid rebuilding the whole graph. Dev/test incremental caches are disabled intentionally; `quick` is a separate profile.

The trusted local `.env.local` at repository root configures native paths. Its contents were **not read, copied or embedded** for this handoff. Configure native paths privately on a new machine. Do not print environment dumps, run shell tracing while loading it, stage it or transfer it through public artifacts. GPU/audio/display access must be available to the test process; a sandbox hiding a device is a prerequisite restriction, not proof the host lacks it.

## Build and test commands

These are continuation commands, **not commands executed again for this documentation handoff**. The recorded successes are in the phase report and JSON receipts. Run one heavy build at a time with `-j 1`; native/Slint compilation uses many GiB and can take minutes without producing output. Do not delete the large existing `src/target` cache to solve a quiet build.

From repository root, enter the workspace and optionally load the owner's trusted native environment without tracing:

```bash
cd /home/turotulco/Projetos/clovercut/src
set +x
set -a
source ../.env.local
set +a
```

Adjust the checkout path on another machine; use a configured development shell instead of sourcing a missing/untrusted file. Then:

```bash
# Focused package regressions (recorded: 347 passes; unrelated live service ignored)
cargo test --offline --locked -p concat-project -p concat-host -p concat-api -j 1 -- --test-threads=1

# Compile the GUI tests once and capture the actual executable path
cargo test --offline --locked -p concat --lib --no-run -j 1 --message-format=json > /tmp/clovercut-test-artifacts.jsonl
handoff_ui_test=$(python3 -c 'import json,sys; xs=[json.loads(s) for s in open(sys.argv[1])]; paths=[x["executable"] for x in xs if x.get("reason")=="compiler-artifact" and x.get("target",{}).get("name")=="concat" and x.get("profile",{}).get("test") and x.get("executable")]; assert len(paths)==1, paths; print(paths[0])' /tmp/clovercut-test-artifacts.jsonl)

# Ordinary GUI unit tests; ignored graphical tests are not passes here
timeout --kill-after=10 180 "$handoff_ui_test" --test-threads=1

# Five Phase 3 graphical tests, each providing isolated child fixtures
timeout --kill-after=10 180 "$handoff_ui_test" studio::phase3_timeline_tests:: --ignored --nocapture --test-threads=1

# Exact T1 regression if only investigating clock/publication
timeout --kill-after=10 180 "$handoff_ui_test" studio::phase3_timeline_tests::transport_clock_matches_decoded_and_published_frame --exact --ignored --nocapture --test-threads=1

# Focused export fixture (already included in the package run)
cargo test --offline --locked -p concat-host --test timeline_precision -j 1 -- --nocapture --test-threads=1

# Final Phase 3 lint/format scope
cargo clippy --offline --locked -p concat-host -p concat-project -p concat-api -p concat --all-targets -j 1 -- -D warnings
cargo fmt --check
git diff --check

# App build/run when separately needed; use a disposable project
cargo build --offline --locked -p concat --bin clovercut -j 1
cargo run --offline --locked -p concat --bin clovercut -- --project /absolute/path/to/disposable-project
```

Do not hardcode the prior test executable hash `concat-84670db89054ae4f` in new automation. Check that a filtered run executes the intended test(s); zero matches are not success. Offline commands require a populated cache; if missing, obtain the lockfile-pinned prerequisites through the normal authorized setup rather than changing dependency versions.

To run all eight ignored GUI fixtures, including the existing source/durability/lifecycle cases, first isolate the parent too:

```bash
handoff_xdg=$(mktemp -d /tmp/clovercut-handoff-tests-XXXXXX)
(
  trap 'rm -rf "$handoff_xdg"' EXIT
  env XDG_CONFIG_HOME="$handoff_xdg/config" XDG_DATA_HOME="$handoff_xdg/data" XDG_CACHE_HOME="$handoff_xdg/cache" XDG_STATE_HOME="$handoff_xdg/state" TMPDIR="$handoff_xdg" \
    timeout --kill-after=10 240 "$handoff_ui_test" --ignored --nocapture --test-threads=1
)
```

The cleanup above owns only the fresh disposable test directory; it is not a Git clean command. Keep any data needed to diagnose a failed run before cleanup. Never redirect tests toward a real project or terminate the user's running editor forcibly.

Native pointer probes are separate Rust executables importing production Slint components. Their build recipe and historical dependency artifact are recorded in `TIMELINE_PHASE3.md`; on a fresh build resolve the matching Slint rlib rather than assuming an old hash or picking an arbitrary stale artifact. The pure helper tests can be run from `src/` with `rustc --edition=2024 --test crates/concat/src/timeline_time.rs -o /tmp/clovercut-timeline-time-tests` followed by that executable.

For later full CI-equivalent qualification, use Phase 8's exact commands in `POLISH_PLAN.md` and the current CI workflow: workspace fmt/Clippy/nextest with required GPU, docs, gRPC checks and root locale/model checks. They were not rerun on the final Phase 3 tree and must not be represented as final Phase 3 full-workspace/release qualification.

## Evidence and verification limits

Raw receipts/logs are local under `/home/turotulco/Work/clovercut-phase1-audit`, `clovercut-phase2-stability`, and `clovercut-phase3-timeline`. For Phase 3 final evidence preserve `task35-regression-packages`, `phase3-final-ui-tests`, `phase3-final-graphical-regressions`, `task36-clock-publication-final`, `phase3-final-clippy`, `phase3-final-format` (`.json` and `.log`) and `final-source-manifest.json`. Portable regression sources are in the repository; local binaries/large fixtures are not required Git inputs. Capture future commands, revision/source identity, exit, matched test counts, timing and skips. Some old `/tmp/phase*.py` scripts are non-idempotent editing scripts, not continuation commands: do not rerun them.

Local Linux verification is not Windows/ARM/package qualification. Linux x86_64/aarch64 AppImages and Windows x86_64 installers/portable ZIPs are workflow targets; native Windows/ARM were not tested here. macOS/iOS/Android scripts are inherited; Nix paths are separately unqualified. No full final workspace CI or release packaging run is claimed.

Still unverified: physical power-loss/storage durability, actual ENOSPC/network filesystems and cross-process save ordering; broad model inference/device/network behavior; physical mouse/touch/HiDPI walkthroughs and exhaustive extreme-duration timecode inputs/readouts; broad VFR/retime source matrices; ten-minute physical A/V synchronization and output/display latency. The T1 fixture checks frame codes modulo eight and publication through the GPU-backed headless/pixel-upload path; it does not detect arbitrary eight-frame seek errors or separately qualify shared-window texture publication/cadence. Source-player preview policy is unchanged. JSON parsing can vary by one ULP; the save/reopen tests allow <1e-12 s only with unchanged frame identity, while undo/redo still requires exact Project equality.

## Phase 4: exact authorized-plan text, implementation not authorized

The following is copied from the current plan to prevent scope drift. Previous Phase 3 compatibility runs do not mark any Phase 4 task complete. Begin only after the user's explicit Phase 4 authorization.

## 4. Playback, preview, and audio synchronization

**Problems identified:** audio callback clock extrapolates up to 100 ms; background PCM decode and preview workers have independent lifecycles; proxies and original media use distinct request paths; real graphical playback regression is opt-in. These are verification gaps, not confirmed sync failures.

**Tasks:**
- [ ] 4.1 Run the existing graphical regression with isolated XDG directories and real audio output; record prerequisites and result.
- [ ] 4.2 Measure flash/tone synchronization at start, after seeking, around cuts and after ten minutes of playback; include fractional rates, nonzero source timestamps, mute and silent projects.
- [ ] 4.3 Exercise rapid play/pause/scrub/project changes and audio device loss/recovery; confirm old jobs cannot publish stale frames or audio plans.
- [ ] 4.4 Compare preview and export for speed curves, reverse, transitions, generated captions, selected audio streams and proxy/full-quality switches.
- [ ] 4.5 Fix each reproduced clock, invalidation or recovery issue separately, preserving Vulkan queue ownership.

**Files likely affected:** `C/concat-host/src/{playback,preview,proxy}.rs`, `C/concat/src/{source,host,lib,studio}.rs`, `src/panes/monitor.rs`, `C/concat-media/src/{audio,pool,prefetch,decode}.rs`, `C/concat-export/src/{flatten,resolve,lib}.rs`.

**Priority / risk:** P1; high for clocks, queues and asynchronous state.

**Acceptance criteria:** target measured A/V offset ≤ one project frame after steady-state playback resumes, with no cumulative drift beyond that over ten minutes; seek completion displays the requested frame; no stale project output; device failure is surfaced and recovery is defined. Record output latency and measurement error separately; revise proposed thresholds explicitly if hardware prevents a meaningful comparison.

**Verification:** timestamped flash/tone fixture, captured audio/video where available, clock/frame logs and existing PCM/preview tests; compare decoded export rather than relying only on the playhead display.

**Dependencies / status:** Phases 2–3; Not started.

## Safe continuation in Antigravity

1. Open the actual repository and read the rules/reports. Confirm branch, HEAD, full status including untracked files, and the empty index before edits. Recheck the source manifest if it is available. If files diverged, preserve and review the new work rather than restoring a prior tree.
2. Confirm the handoff files and all regression/probe sources are present. Record current local state; do not assume a remote checkout has Phase 3. Retain the existing safety checkpoint and obtain explicit permission before making any new Git checkpoint/transfer.
3. Configure private native prerequisites and display/GPU/audio access. Reuse valid caches. Read existing successful/failed receipts before running expensive checks; documentation-only handoff changes do not require a new application build.
4. Wait for Phase 4 authorization. Once granted, work sequentially through 4.1–4.5, independently verifying each. Establish measured flash/tone baselines and device/output latency before changing clocks. Preserve Phase 3 policy and tests.
5. Reproduce each suspected defect with disposable fixtures, make one scoped correction, and rerun focused plus relevant regressions. A major timing/architecture/semantics decision must be documented and approved before implementation. Do not reintroduce earlier audio or queue problems.
6. Update the plan and a Phase 4 report with actual results, failures, skips, measurement error and unresolved limits. Do not claim release readiness or silently waive device/platform checks. Stop at the next requested approval gate.

## Proposed Git checkpoint — review only

No command in this section was executed. Ask for explicit approval before staging or committing; approval for the handoff alone is not Git approval. Preserve the current branch and make the checkpoint on it, without switching/resetting/cleaning. First review:

```bash
git -C /home/turotulco/Projetos/clovercut status --short
git -C /home/turotulco/Projetos/clovercut diff --check
git -C /home/turotulco/Projetos/clovercut diff --stat
```

After approval, use the explicit path allowlist in the inventory below (including all untracked tests/probes and handoff docs), review the staged diff and only then commit. Avoid blanket `git add .`/`git add -A`, forced adds, or including `.env.local`, native libraries, test binaries, `src/target`, real projects or local evidence directories. Suggested commit message: `checkpoint: verified phase 3 and Antigravity handoff`. A push remains a separate explicitly approved action.

After explicit approval, the staging allowlist for this snapshot is:

```bash
git -C /home/turotulco/Projetos/clovercut add -- \
  POLISH_PLAN.md \
  src/crates/concat/src/format.rs \
  src/crates/concat/src/lib.rs \
  src/crates/concat/src/panes/captions.rs \
  src/crates/concat/src/panes/export.rs \
  src/crates/concat/src/panes/glide.rs \
  src/crates/concat/src/panes/media_bin.rs \
  src/crates/concat/src/panes/monitor.rs \
  src/crates/concat/src/panes/sounds.rs \
  src/crates/concat/src/panes/speech.rs \
  src/crates/concat/src/panes/timeline.rs \
  src/crates/concat/src/panes/voiceover.rs \
  src/crates/concat/src/source.rs \
  src/crates/concat/src/studio.rs \
  src/crates/concat/ui/editor.slint \
  src/crates/concat/ui/phone/editor.slint \
  src/crates/concat/ui/timeline/lanes.slint \
  src/crates/concat/ui/timeline/model.slint \
  src/crates/concat/ui/workspace/seat.slint \
  src/crates/concat/ui/workspace/timeline-pane.slint \
  AGENTS.md \
  docs/AGENT_HANDOFF.md \
  docs/TIMELINE_PHASE3.md \
  docs/audit/relative-timeline-ui-probe.rs \
  docs/audit/timeline-rebase-probe.rs \
  docs/audit/timeline-ui-probe.rs \
  src/crates/concat-host/tests/timeline_precision.rs \
  src/crates/concat/src/phase3_timeline_tests.rs \
  src/crates/concat/src/timeline_time.rs
```

Then inspect `git diff --cached --check`, `git diff --cached --stat` and `git diff --cached` in that repository. Stop if any unexpected path, credential, binary or omitted regression appears. After the approved staged review, the checkpoint command is:

```bash
git -C /home/turotulco/Projetos/clovercut diff --cached --check &&
git -C /home/turotulco/Projetos/clovercut commit -m "checkpoint: verified phase 3 and Antigravity handoff"
```

Neither staging nor commit was executed. These commands do not push anything.

## Exact uncommitted path inventory at handoff

`M` means a tracked modification; `??` means a new untracked file. This includes this handoff and the newly created rules file. HEAD and branch above are unchanged; no staged changes. Any new path/status after this snapshot requires review before checkpointing.

```text
 M POLISH_PLAN.md
 M src/crates/concat/src/format.rs
 M src/crates/concat/src/lib.rs
 M src/crates/concat/src/panes/captions.rs
 M src/crates/concat/src/panes/export.rs
 M src/crates/concat/src/panes/glide.rs
 M src/crates/concat/src/panes/media_bin.rs
 M src/crates/concat/src/panes/monitor.rs
 M src/crates/concat/src/panes/sounds.rs
 M src/crates/concat/src/panes/speech.rs
 M src/crates/concat/src/panes/timeline.rs
 M src/crates/concat/src/panes/voiceover.rs
 M src/crates/concat/src/source.rs
 M src/crates/concat/src/studio.rs
 M src/crates/concat/ui/editor.slint
 M src/crates/concat/ui/phone/editor.slint
 M src/crates/concat/ui/timeline/lanes.slint
 M src/crates/concat/ui/timeline/model.slint
 M src/crates/concat/ui/workspace/seat.slint
 M src/crates/concat/ui/workspace/timeline-pane.slint
?? AGENTS.md
?? docs/AGENT_HANDOFF.md
?? docs/TIMELINE_PHASE3.md
?? docs/audit/relative-timeline-ui-probe.rs
?? docs/audit/timeline-rebase-probe.rs
?? docs/audit/timeline-ui-probe.rs
?? src/crates/concat-host/tests/timeline_precision.rs
?? src/crates/concat/src/phase3_timeline_tests.rs
?? src/crates/concat/src/timeline_time.rs
```
