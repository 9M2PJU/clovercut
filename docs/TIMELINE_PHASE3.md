# Phase 3 — timeline precision and editing interactions

Date: 2026-10-21. Baseline: `17f942be6bb0a8373b2e09e4d90f930d5f128353` on `safety/phase2-complete-2026-10-21` (also published to origin and independently checked with `git ls-remote`). Work branch: `polish/phase3-timeline-precision`. Phase 2 complete and approved. Phase 3 tasks 3.1–3.6 authorized sequentially; stop before a major architecture or editing-semantics change, and do not start Phase 4.

> Final status: tasks 3.1–3.6 are complete and explicitly approved by the user within the documented local verification scope. The final acceptance and evidence sections below supersede all historical investigation checkpoints. Phase 4 has not started and remains unauthorized. Continuation is documented in [AGENT_HANDOFF.md](AGENT_HANDOFF.md). No new commit or push has been made.


## Policy recorded before timing changes

This policy was recorded before production corrections. The existing document format remains f64 seconds. Core rational time and half-open frame intervals remain authoritative in the engine. Existing frame-grid operations (the razor and video resolve) select nearest nonnegative frame, with positive exact half-frames rounding upward. Audio retains its sample grid. Continuous-time menu splitting/move/trim and f32 UI coordinates currently coexist; globally imposing a frame grid on those paths is an editing-semantics decision, not assumed authorized by the request to improve precision. Before any correction, verify the particular path and distinguish restoring an intended exact-frame operation from changing an intentionally continuous operation.

For continuous transport, floor selects the frame whose half-open interval contains the true clock instant. Nearest is appropriate for a chosen frame boundary, but applying nearest to a running clock advances a picture at the half-frame point. The known transport/preview discrepancy must be traced through actual publication in task 3.6 before choosing a correction. Do not replace all time types or silently clamp the project duration.

## 3.1 Conversion characterization — complete

Fixture: `docs/audit/timeline-ui-probe.rs` imports the production Slint `Lanes` component, opens a disposable 1000×200 window, dispatches actual `WindowEvent` pointer press/release/move events, observes the ruler's scrub callback and checks clip hit-testing in the real scene graph. It does not call a Rust reimplementation of `seconds-at` or `at` to obtain observed UI results. Input x is constructed from a chosen frame's narrowed UI coordinate, so this tests roundtrip preservation after narrowing as well as independent exact-frame identity.

Matrix: all six rates (24, 25, 30, 60, 24000/1001, 30000/1001); zoom in seconds/pixel 0.0005, 0.005, 0.05, 0.5, 1.5; scroll seconds 0, 60, 3600, 21600, 86400, 262144; five visible positions near x=100/300/500/700/900. Total 900 ruler callbacks and 900 clip-position hit tests. The scroll window includes a few seconds to 22.5 minutes beyond each scroll origin depending on zoom; do not describe the last sample as exactly at the origin's duration.

Actual result: process exit 0 in 0.67 s, all 1800 UI callbacks/hit checks and error-bound assertions passed. **Two wrong-frame cases were observed** at 60 fps, scroll 262144 seconds (~72.8 h), closest zoom. Frame 15728643 became 15728644; frame 15728667 became 15728666. Both errors already exist when the exact f64 frame time is narrowed to the incoming f32; no additional frame changed during the ruler roundtrip in this matrix. The named audit frame 15728641 separately becomes 15728642. All matrix cases in scroll windows up to 21600 seconds (~6 h) retained their expected nearest frame. This is measured coverage, not a universal six-hour precision guarantee or a new project limit.

Priority T2/P1: confirmed native UI inability to distinguish some neighbouring frames at sufficiently long durations. Changing to viewport-relative/frame-index/f64 interaction state requires a scoped design; no whole-controller rewrite or format migration is inferred. Existing exact rational engine calculations cannot recover frame information already discarded by the UI.

Build used Rust 1.93.1 from `src/`, native `.env.local` environment, warm existing Slint artifact:

```sh
rustc --edition=2024 ../docs/audit/timeline-ui-probe.rs -L dependency=target/debug/deps --extern slint=target/debug/deps/libslint-53124fc284688498.rlib -o /tmp/clovercut-timeline-ui-probe
/tmp/clovercut-timeline-ui-probe
```

The initial successful invocation used an identical generated source with absolute local import paths under `/home/turotulco/Work/clovercut-phase3-timeline/`; the portable checked-in source uses repository-relative imports and passed separately (`task31-native-portable`, exit 0, 0.68 s). The existing core rational-time suite also passed all 13 tests (`task31-rational`, 0.02 s). Full CSV/summary and JSON receipt: `task31-native-matrix.log/.json` in that directory. This is native Slint pointer/hit-test coverage, not a manual human walkthrough or validation of physical mouse/touch hardware, HiDPI, all window layouts, autoscroll or mobile gestures.

## 3.2 Cut and gesture boundary verification — complete after Option A approval

The following records the pre-approval characterization. At that checkpoint the task remained incomplete pending the coordinate/semantics decision; Option A was subsequently approved, and the final corrections and acceptance results are recorded below.

Fixture `src/crates/concat/src/phase3_timeline_tests.rs` executes actual Studio handlers inside an isolated graphical/GPU subprocess, with disposable projects and isolated XDG directories. It runs 24 cases (six rates × frame 50 plus offsets 0, 0.49, 0.5, 0.51), checks both splitting paths, refuses exact clip endpoints, checks move and tail-trim by a quarter-frame, verifies inside/exact snap thresholds, compares saved/reopened state after each split and checks **exact** Project restoration after undo. These are real controller calls, not physical pointer dragging through the complete App. The preceding 3.1 fixture supplies native pointer/scene-graph coverage for Lanes; the two fixtures' scopes are intentionally distinguished. Preview requests are suppressed in this editing/persistence fixture, so it does not establish transport, decoder or frame-publication correctness.

Observed result: 1 explicit ignored test passed, 24 cases, 0.53 s process time (0.48 s test time). Menu/razor stored times differ in 23 cases; five of six exact-frame menu inputs retain the f32 approximation rather than exact frame time. Four ideal half-frame inputs round down after narrowing. Moves/trims retain quarter-frame offsets with snapping off: free subframe editing is existing behavior, not an assumption that every gesture was already frame-aligned. Nearest clip/playhead snap targets are accepted strictly inside the threshold; an exact threshold is not snapped.

### T3/P1 — confirmed one-frame cut difference through export

At 24000/1001 fps, the ideal frame-50.5 instant narrows to f32 `2.1062707901000977` seconds. Menu splitting stores that input directly. Multiplying it by the full project rate gives `50.499998963438905`, so the video resolver's nearest-frame rule selects frame 50. The razor first narrows the project rate to f32; its product becomes `50.50000033601464`, selecting frame 51. This is a **confirmed numerical defect** in the razor's rate conversion, separately from the policy question whether menu/free gestures should be frame-aligned. The intended nearest-frame calculation for an already-provided instant should use the project rate without the extra f32 narrowing. Its isolated correction belongs in task 3.5 after the preceding task gates.

Export fixture `src/crates/concat-host/tests/timeline_precision.rs` feeds the independently observed current-handler timing formulas into the production exporter, marks the two sides red/blue, encodes 12 videos (menu and razor × six rates), decodes every frame with FFmpeg and checks the cut and all 96 frames per video. It does **not** claim to export the text project from the Studio fixture: using distinct solid images makes the cut visually observable. At 24000/1001, the first blue frame is 50 for menu and 51 for razor. At the other tested half-frame cases the paths agree in the encoded output:

| Rate | Menu first blue frame | Razor first blue frame |
|---|---:|---:|
| 24 | 51 | 51 |
| 25 | 50 | 50 |
| 30 | 50 | 50 |
| 60 | 50 | 50 |
| 24000/1001 | 50 | 51 |
| 30000/1001 | 50 | 50 |

This establishes a short-project visible cut discrepancy, not only an arithmetic probe. It does not establish full playback/audio parity or every source-rate/retime boundary.

### Persistence comparison diagnostic

The first characterization run stopped at the 24000/1001 razor project because a bitwise f64 equality assertion on save/reopen found `2.1271249421853438` becoming `2.127124942185344` (one ULP, ~4.44e-16 s) in JSON parsing. This is recorded separately from frame loss; no changed frame, material edit loss or cumulative drift was reproduced from that diagnostic. The test now checks all other project fields exactly, checks start/duration/source_start within 1e-12 s and identical frame indices, and still requires exact full Project equality after undo. All 24 cases then passed. No parser/format/dependency change was made. The original failing log is retained and is not counted as a pass. An initial fixture compile error (`f64::from(i64)` and redundant import) was also corrected and is retained as a failed build, not an application defect.

### Verification receipts (commands from src/)

Source baseline remains `17f942be`; only audit/test additions and test-module wiring have been made on the work branch. Native environment sourced from `../.env.local` without logging its values.

| Receipt | Exact command | Actual result |
|---|---|---|
| task31-native-portable | `/home/turotulco/Work/clovercut-phase3-timeline/probe-ui-portable` (build command above) | Exit 0, 0.68 s; 900 ruler and 900 clip-hit cases; two characterized wrong-frame results. |
| task31-rational | `/home/turotulco/Work/clovercut-phase3-timeline/time-tests --nocapture` (built with `rustc --edition=2024 --test crates/concat-core/src/time.rs -o /home/turotulco/Work/clovercut-phase3-timeline/time-tests`) | 13 passed, 0.02 s. |
| task32-ui-build-persistence-check | `cargo test --offline --locked -p concat --lib --no-run -j 1` | Exit 0, 316.31 s. Earlier corrected build also passed in 326.81 s. |
| task32-cut-semantics | `timeout --kill-after=10 180 target/debug/deps/concat-84670db89054ae4f studio::phase3_timeline_tests::cut_and_gesture_rounding_characterization --exact --ignored --nocapture --test-threads=1` | 1 passed, 0.53 s, 24 cases. |
| task32-export-cuts | `cargo test --offline --locked -p concat-host --test timeline_precision -j 1 -- --nocapture --test-threads=1` | 1 passed; 12 exports/1152 decoded frames; 149.32 s including waiting for the UI build lock, 1.98 s actual test. |
| task32-ui-regressions | `timeout --kill-after=10 180 target/debug/deps/concat-84670db89054ae4f --test-threads=1` | 166 passed, 0 failed, 4 ignored, 0.74 s. New characterization test was executed explicitly above; the other ignored tests were not rerun here. |
| task32-test-clippy | `cargo clippy --offline --locked -p concat-host --test timeline_precision -j 1 -- -D warnings` | Exit 0, 2.32 s. This is a focused lint, not a new full UI/workspace Clippy claim. |
| task32-format | `cargo fmt --check` | Exit 0, 0.97 s. |

`git diff --check` passed. These runs total 181 passed unit/integration tests (13 core + 166 existing UI + 1 controller characterization + 1 export characterization); native UI matrix events and earlier reruns are not added as separate tests. New tests are characterization evidence and will need to change into intended-policy regressions when fixes are implemented.

## Required precision and editing-policy decision

The request explicitly says: “If a timing correction requires a major architectural decision or changes existing editing semantics, document the alternatives and stop for my approval.” This checkpoint invokes that user requirement, not an extra permission rule.

T2 cannot be corrected by increasing zoom, adding an epsilon or replacing one rounding call: f32 absolute seconds have already merged distinct frame positions. Guaranteeing frame identity at the tested long positions requires changing the coordinate protocol between authoritative state and Slint. T3 also makes it necessary to distinguish a chosen continuous instant from an intended exact frame/half-frame; applying an arbitrary epsilon near half-frames would change genuine clicks immediately below the boundary.

Concrete alternatives at the historical approval checkpoint (Option A subsequently approved and implemented below):

| Option | Precision boundary and editing contract | Compatibility / risk |
|---|---|---|
| **A — precise UI boundary, preserve continuous editing (recommended)** | Keep authoritative frame picks as integer frame indices and other times/origins as f64. Send viewport-relative geometry and local pointer deltas through Slint instead of f32 absolute timeline seconds; reconstruct absolute times using the precise origin. Use the exact project rate at explicitly frame-aligned operations, nearest with positive exact ties upward. Keep existing menu/free move/trim subframe semantics and audio sample precision. Retain floor for continuous transport until task 3.6 establishes the intended published-frame contract. | Document format remains f64/version 1, existing subframe project data is not rewritten. Changes are confined to timing/coordinate boundaries and small helpers, with independent tests; no broad studio.rs rewrite. This is nevertheless an architectural coordinate-contract decision requiring approval before implementation. |
| **B — precise UI boundary plus visual-frame editing grid** | Apply A's coordinate precision, and also make menu/razor/move/trim of visual clips use the same project-frame grid (nearest, positive exact ties upward). Audio retains subframe/sample editing. Define treatment of mixed visual/audio selections and already-off-grid source cuts before implementing those paths. | No format migration is necessary, but editing semantics change: visual placements that currently retain quarter-frame offsets would land on whole frames. Existing off-grid data stays intact until explicitly edited. Broader mixed-selection/source/ripple acceptance tests are required. |

A is recommended for preserving established behavior while correcting loss of precision. B is appropriate only if mandatory visual frame alignment is the intended product rule. Do not quietly choose B or impose a new maximum project duration as a workaround. Both options preserve Phase 2 ordered tickets, atomic replacement and durability outcomes; neither authorizes a persistence rewrite.

**Stopping point:** task 3.1 complete; task 3.2 characterized and awaiting the explicit coordinate/rounding contract; tasks 3.3–3.6 not started. No production fix has been implemented, so no before/after improvement is claimed. Phase 3 does not yet satisfy its acceptance criteria. There is no claim of exact media-source trim bounds, locked/ripple mixed selection, cancellation, gesture undo grouping, cumulative long-edit drift, real transport/publication T1 correction or full A/V parity. Continue sequentially only after the user selects a policy; Phase 4 remains unstarted.



## Option A approved — sequential continuation

The user explicitly selected Option A. The preceding stopping-point and alternatives describe the historical approval checkpoint. Preserve continuous menu/move/trim editing, sample/subframe audio and version-1 f64 documents. Implement viewport-relative UI timing, precise authoritative origins and exact-rate frame picks in small changes during task 3.5; do not impose Option B or start Phase 4. With this contract selected, the verified task 3.2 boundary characterization is complete. Tasks 3.3 and 3.4 now verify the remaining interactions before production corrections.

## 3.3 Selection, placement, locks and source boundaries — verification complete

`task33-project-baseline`: 170 existing model tests passed (0.04 s), including placement, ripple, source-head clamping/speed mapping and split continuity. Existing tail-trim contract stretches duration and stops at the minimum, without imposing a new source-tail policy; no automatic freeze/trim semantic change is introduced. The Studio fixture uses video/audio metadata plus text clips to check mixed selection, removes selected locked lanes through the actual track-lock handler, refuses locked press/split, and preserves group spacing for an ordinary positive move. It then reproduces **T4/P1**: dragging clips at 0.5 and 2.5 seconds on different lanes by -3 seconds puts both at zero. Independent per-clip clamping destroys rigid group spacing. `task33-origin-before` exited 101 at the expected assertion (actual 0 versus expected 2 for the second clip), in 5.82 s. No production fix yet; correction belongs to 3.5. Synthetic media paths produce a missing-thumbnail diagnostic; this fixture tests metadata selection/controller placement, not native decoding or real source playback.

No stale lock-selection bug was established: the real lock handler removes locked clips from selection. Do not fix the earlier hypothetical manually injected selection case.

## 3.4 Cancellation and history — verification complete

Real Studio click/no-op and complete-gesture undo/redo restored the exact full Project in the new fixture. The repeated-drag regression then failed: 100 pairs of +0.125/-0.125 s at start 262144+1/60 s leave 262144.03125 rather than 262144.01666666666 (`task34-drift-before`, exit 101, 5.95 s). This confirms T2 in an editing path: each drag captures a narrowed f32 origin. No model/history drift was inferred from a mere arithmetic probe.

**T5/P1:** `task34-cancel-before` dispatches actual pointer press, movement and `WindowEvent::PointerExited` through production Lanes. Slint delivers its real Cancel event; Lanes incorrectly emits `clip-released`, requesting a commit (actual count 1, expected 0; exit 101, 0.10 s). The fix must discard the echo without a history entry, stop autoscroll and keep ordinary release as one commit. No Escape semantics or unrelated stage-gesture refactor is inferred.

The failing graphical child also reports the existing native TLS-abort-on-unwind behavior after its assertion; the primary failing assertion is retained above, and is not mislabeled as a newly fixed application shutdown defect. Task 3.5 now implements isolated corrections and their after-fix regressions.


## 3.5 Approved corrections — complete within local scope

Authoritative edit origins, snap comparisons, playhead and timeline scroll now retain f64 precision. Only local viewport coordinates/deltas cross Slint; origin changes use integer seconds and update held drag references. Explicit frame operations use the exact project rational rate; continuous menu/free edits and version-1 document serialization retain their contract. Selected group movement uses one common lower-bound clamp, preserving spacing. Pointer cancellation discards the gesture echo without committing history.

After-fix receipts already passed: `task35-origin-after` (actual mixed-selection/locks/common-zero clamp), `task35-drift-after` (100 opposite drag pairs/exact history), `task35-cuts-after` (24 controller cases with save/reopen), `task35-export-after` (12 exports, 1152 decoded frames, zero cut differences), `task35-cancel-after` (native pointer cancellation), and `task35-native-relative` (900 ruler plus 900 clip hits, zero frame mismatches). These were intermediate receipts before the final geometry addition; the completed integrated and final regressions are recorded below.

A tighter real native hit test confirmed an additional manifestation of T2: a clip spanning zero to 262144+1/60 s accepted a click at x=34 px, beyond its true 33.33 px tail at 0.0005 s/px (`task35-long-geometry-boundary-before`, exit 101). Viewport rebasing alone left a huge clip width. Publication now bounds displayed clip geometry to the overscan window, preserving the authoritative clip and rebasing waveform/filmstrip/fade views. `task35-long-geometry-after` passed (exit 0, 0.07 s); `task35-coordinate-window-tests` passed all three production-helper tests (exit 0, 0.00 s). This is a reproduced geometry correction, not a change to cut timing. The integrated publication/cancellation additions subsequently passed, as recorded below.

Build failures from the f64 boundary conversion and a mistakenly duplicated display-only fade field in SelectedClipData are recorded separately, corrected, and not counted as application regressions or passing checks. At that historical checkpoint task 3.5 was incomplete; tasks 3.5 and 3.6 are now complete and approved. Phase 4 remains unstarted.


`task35-native-rebase` passed (exit 0, 0.07 s): production Lanes pointer events preserve the drag delta while origin changes from 0 to 1024 seconds, then include a 0.05-second scroll delta correctly; pointer exit emits exactly one cancellation and zero releases. Portable source: `docs/audit/timeline-rebase-probe.rs`. The integrated fixture additionally checks cancelled move/trim leave the entire Project unchanged and do not add undo history. Those integrated additions subsequently passed in the final controller regressions below.


Final geometry integration build `task35-windowed-final-build`: exit 0, 370.93 s. `task35-windowed-ui-baseline`: 169 passed, 0 failed, 7 ignored (0.75 s command; 0.70 s test report). `task35-windowed-drift-cancel`: 1 passed, 0 failed (0.51 s), including exact Project restoration after move/trim cancellation, absence of a cancelled history step, exact single-gesture undo/redo and 100 inverse drag pairs. `task35-native-relative-final`: exit 0, 0.75 s; 900 ruler callbacks + 900 real clip hits, zero frame mismatches with final Slint geometry. The final publication fixture correction reads the published Models.clips directly rather than an Editor global not bound by the isolated fixture; that test and additional six-rate frame-step assertions subsequently passed in `task35-final-controller-regressions`. No zero-filter run is counted.


`task35-regression-packages`: exit 0, 388.21 s including the UI-build lock wait. **347 tests passed, zero failed, one unrelated ignored** across concat-api (27), concat-host library (122), export integration (21), persistence integration (5), stabilization integration (1), timeline export integration (1) and concat-project (170). Three empty doc-test suites are excluded from this total. The timeline integration re-exports 12 videos/1152 decoded frames and passes the corrected cut-frame assertions. Existing Phase 2 persistence protections and model command regressions passed unchanged. The final UI fixture build subsequently passed; see the final receipts below.


## 3.5 Final functional verification after interruption

Recovered the interrupted final build: `task35-final-test-build-resumed` passed, 323.35 s. The final binary passed `task35-final-ui-regressions` (169 passed, 7 ignored, 1.03 s) and `task35-final-controller-regressions` (all four explicitly ignored Phase 3 fixtures passed, 2.16 s), including published bounded long-clip geometry and exact six-rate positive/negative frame stepping. Together with the already recorded 347 package regressions, this is 520 passing tests on the final functional changes; native pointer matrix cases are not counted as unit tests. `task35-format-resumed` passed (1.03 s), and `git diff --check` passed. Strict selected-package Clippy was still pending at this checkpoint; its final successful receipt is recorded below. Functional corrections are verified independently before the following task.

## 3.6 Real clock and publication investigation

The intended contract is the policy recorded before changes: a continuous playback clock selects its containing half-open frame interval (floor with the existing 1e-6-frame numerical tolerance); paused chosen instants keep the existing nearest-frame preview rule. No audio clock or persistence semantics change is proposed. The new isolated graphical fixture encodes lossless frame-identifying binary RGB video with exact rational NUT timestamps, starts the actual playback engine and Studio transport timer at frame 50, and observes asynchronous decoding, rendering and publication to the real Editor image. It uses the production pixel publication path and a test-only observation of completed frame bytes. This was the pre-reproduction checkpoint; the observed discrepancy and final successful correction are recorded below.


`task35-clippy-resumed` exited 101 after 608.8 s (including the earlier build lock wait): 15 same-type conversions remained after moving authoritative timing to f64. Each reported conversion was removed at its exact call site; this is a type-cleanup correction, not a new editing behavior. The failed lint is retained and is not counted as passing. The first 3.6 reproduction build was pending at that checkpoint; both it and the later final strict lint have recorded results below.


### T1/P1 confirmed through real playback and publication

`task36-reproduction-build` passed (508.9 s including the prior Clippy lock wait). `task36-clock-publication-before` failed (exit 101, 7.22 s), running all six rate subprocesses and observing 16 real published moving pictures per rate. Wrong containing-frame identities: 24 fps 1/16, 25 fps 1/16, 30 fps 2/16, 60 fps 2/16, 24000/1001 0/16, 30000/1001 1/16. This fixture samples the actual playback-clock distribution; absence of an error in one sampled rate is not proof of correctness there. Example: 24 fps clock 2.189748 s is in frame 52, while the decoded binary code was frame 53 modulo 8. Slint's Editor image bytes matched the completed Monitor image in every child before the mismatch assertion, so the discrepancy reached publication. The fixture does not infer screen presentation or audio-device output latency.

Root cause: transport asks once per floor-selected interval, but a delayed/coalesced monitor request can carry a clock in the late half of that interval; preview's nearest quantization then asks the decoder for the next frame. The correction selects the containing project-frame boundary in MonitorPane before title/clip preparation and rendering during timeline playback only. Continuous playhead/audio clock, paused nearest-frame picks, library-source preview and persisted projects are unchanged. The final fixture also explicitly publishes paused exact frame 50 and starts actual playback at frame 50.75 to guarantee late-half coverage at all six rates. Final verification subsequently passed at all six rates; see the final result below.


### Final static and existing graphical regressions

`phase3-final-clippy`: **passed**, exit 0, 1047.02 s including the final application build lock and checking both library/test targets. Exact command from `src/`: `cargo clippy --offline --locked -p concat-host -p concat-project -p concat-api -p concat --all-targets -j 1 -- -D warnings`. Existing patched whisper-rs-sys dependency warnings remain subject to dependency lint caps; no unrelated dependency cleanup was made. `phase3-final-format` passed (1.39 s); `git diff --check` passed.

On the final production correction, `phase3-final-ui-tests` passed all 169 normal tests (8 explicitly ignored, 0.93 s). `phase3-final-graphical-regressions` explicitly ran the other seven ignored fixtures in isolated XDG/TMPDIR (3.66 s): all four prior Phase 3 controller fixtures, both Phase 2 durability/lifecycle fixtures and the existing library/timeline playback fixture passed. That last test is a scoped compatibility regression, not completion of Phase 4 A/V qualification. The source implementation remains based on revision `17f942be6bb0a8373b2e09e4d90f930d5f128353`, branch `polish/phase3-timeline-precision`; no engine/persistence/format changes were added in Phase 3. Task 3.5 is complete.

The first strengthened T1 post-fix fixture (`task36-clock-publication-after`) stopped at its paused setup assertion because normal project opening had already queued a frame-zero request before the explicit frame-50 seek. This is a test synchronization error, not a reproduced application regression. The fixture now waits for the matching seek result before starting playback. Its corrected build and final six-rate execution subsequently passed; the failed receipt remains retained.


## 3.6 Final result — complete within local scope

`task36-fixture-final-build` passed (exit 0, 1016.03 s, including the Clippy lock wait). `task36-clock-publication-final` passed (one explicit ignored test, six isolated rate subprocesses, 6.95 s). Every child publishes paused exact frame 50 through the actual asynchronous monitor and verifies its decoded binary identity. It then starts the real playback clock at frame 50.75, explicitly requests the late-half image, and continues through the production transport timer, decode workers and rendering. All completed moving images match the containing original-clock frame's binary RGB code; the prepared request also names that exact frame boundary. The Editor's published image bytes match the monitor's completed image at the paused checkpoint and at the end of each child.

| Project rate | Moving images checked | Late-half raw-clock samples | Result |
|---|---:|---:|---|
| 24/1 | 16 | 2 | Pass |
| 25/1 | 16 | 1 | Pass |
| 30/1 | 16 | 2 | Pass |
| 60/1 | 16 | 1 | Pass |
| 24000/1001 | 16 | 1 | Pass |
| 30000/1001 | 16 | 1 | Pass |

These are 96 moving-image checks and six paused frame-50 checks, not additional unit-test counts. Three-bit RGB codes identify frames modulo eight and distinguish the reproduced one-frame error; this fixture is not a test for arbitrary eight-frame seek errors. Frames may arrive late or be skipped under load: this task verifies which source frame was requested for the clock sample, not physical display cadence or audio output latency. The fixture uses GPU-backed headless composition and the production pixel-upload publication path; shared-window texture publication is not separately qualified by this new fixture.

### Final Phase 3 acceptance

| Criterion | Final evidence |
|---|---|
| Declared frame-aligned operations retain the intended frame | Exact-rate razor/controller and six-rate frame stepping pass; native relative-coordinate matrix has zero wrong frames; 12 encoded cuts/1152 decoded frames agree. |
| Repeated editing does not accumulate frame drift | 100 opposite long-position drag pairs, exact full-Project undo/redo and cancelled move/trim history checks pass. |
| Selection, locks, placement and source constraints retain their contract | Mixed audio/video/text selection, actual lock handling, common before-zero group clamp and 170 existing model tests pass. |
| Cancelled gestures preserve document/history | Native pointer exit emits exactly one cancellation/zero releases; actual Studio cancellation leaves the full Project and history unchanged. |
| Saved state and persistence protections are retained | Per-cut and long-state save/reopen assertions, all five host persistence tests and both Phase 2 graphical durability/lifecycle regressions pass. |
| Real running-clock frame selection is consistent | The before-fix publication discrepancy is reproduced; paused exact frame 50 plus forced late-half and timer-driven playback pass at all six rates after correction. |

**Tasks 3.1–3.6 satisfy their local acceptance criteria, are complete, and have been explicitly approved by the user within this documented scope.** No design decision remains within the approved Phase 3 scope. Phase 4 remains unstarted and requires the user's explicit authorization.

Final focused test total: **524 passed** = 347 package tests + 169 normal UI tests + seven existing graphical tests + one new six-rate clock/publication test. Earlier repetitions and native matrix events are excluded from this count. Strict all-target Clippy for concat/concat-host/concat-project/concat-api passed, as did `cargo fmt --check` and `git diff --check`; no full-workspace/target-release qualification is claimed.

Verified source identity: base revision `17f942be6bb0a8373b2e09e4d90f930d5f128353`, branch `polish/phase3-timeline-precision`, with the exact modified/new Rust and Slint source hashes in `/home/turotulco/Work/clovercut-phase3-timeline/final-source-manifest.json` (SHA-256 `97477a00db6366b70a26cb1ee94975821bc63c36d96e21fdd94455b4a38a8829`). This is the verified working tree, not a claim of a new commit or push. Raw logs, exact commands, timings and retained failed receipts remain in the same evidence directory.

Remaining limitations: this is one Linux/GPU environment; no native Windows/ARM/mobile qualification, physical mouse/touch/HiDPI walkthrough, exhaustive timecode-input/readout coverage at extreme durations, broad variable-rate/retimed-source matrix, or ten-minute physical A/V synchronization measurement. Source-player preview policy remains unchanged. The pre-existing JSON parser's one-ULP variation is tolerated only below 1e-12 s with unchanged frame identity; no format/parser change was made. Physical presentation/device latency and shared-texture-path qualification remain later work; passing these focused tests is not a release claim.
