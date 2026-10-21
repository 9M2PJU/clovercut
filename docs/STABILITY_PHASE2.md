# Phase 2 — critical bugs and stability

Date: 2026-10-21. Source baseline: `82a498be342d02bba0429ec428eacc8c8b0e6f91`, with the Phase 1 documentation preserved. Phase 2 tasks 2.1–2.4 were explicitly authorized, in order, with independent verification and a durability design approval gate. Phase 3 is not authorized.

## Scope and status

- **2.1 Complete:** disposable-project persistence verification, two reproduced defects corrected independently, focused before/after regressions and relevant host/project/API checks passed.
- **2.2 Complete:** approved Option B implemented and verified; 345 package tests passed (one unrelated ignored), explicit GUI warning/close regression passed, focused Clippy/format passed.
- **2.3 Complete:** real controlled lifecycle overlap passed, including refused switch and shutdown with jobs already running.
- **2.4 Complete:** save-error UI, post-capture edits and actual process interruption verified; final relevant regressions passed. No additional production crash was reproduced.

No failure injection used a real user project. Test fixtures use `tempfile::TempDir`; the Linux file-size-limit test validates a dedicated temporary-path prefix and fixture marker before writing. Its resource limit and SIGXFSZ disposition affect only an exact-test subprocess and are restored before retry. No system configuration, project format, dependency or lockfile was changed. Task 2.1 left the save protocol unchanged. The approved task 2.2 adds Linux directory sync after replacement while retaining the existing temporary, file-sync and replacement protections; the ticket is published at commit.

## 2.1 Confirmed defects and corrections

### S2 — future settings-only manifest accepted as an empty edit

**Before:** `Session::open` attempted the settings-only fallback before rejecting a future `DOCUMENT_VERSION`. A disposable settings-only manifest with version 2 and an opaque `futureEdits` object opened successfully. An ordinary `Session::save` then rewrote it as version 1 and removed `futureEdits`.

**Reproduction:** `session::tests::a_newer_settings_manifest_cannot_be_opened_and_saved_as_an_empty_edit`. Before the fix it failed with `version=1, futureEdits=None`, exit 101. This resave exists only in the disposable test's failure branch.

**Correction:** check the existing future-version rejection before permitting the settings-only fallback. This changes no format or supported-version policy; it enforces the rejection already required for complete edit documents.

**Verification:** the same test passed after the correction, checks the existing actionable update error, and verifies refusal preserves the original manifest bytes. The integration matrix also verifies a complete future edit is refused under both `concat.json` and legacy `wolfcut.json` names.

### S3 — supported settings-only manifest loses unknown metadata

**Before:** the settings-only path constructed a new editor without the original document. Unknown fields preserved by the normal document reader were lost on the first save. The regression failed with `pluginMetadata = Null` rather than the original nested object.

**Correction:** seed the missing initial timeline, then pass the supported manifest through the existing document reader. That reader retains unknown fields in `Project::extra`. Existing migrations, normalization, initial four tracks, frame settings and empty undo history are retained. No new persistence format or version is introduced.

**Verification:** `unknown_settings_manifest_fields_survive_the_first_and_later_saves` passed after the fix for a legacy manifest with no explicit version and a current-version manifest. It checks opening is read-only, frame/track/history behavior, first save, reopening and a subsequent save.

## 2.1 Persistence and recovery matrix

New integration suite: `src/crates/concat-host/tests/persistence.rs`.

| Case | Actual verified behavior |
|---|---|
| Truncated JSON, null, array, invalid timeline shape and future complete document | Refused; original bytes preserved, no replacement manifest created. Each case ran under current and legacy manifest names. |
| Directory occupying the manifest path | Refused; existing directory contents preserved. |
| Complete uncommitted `.saving` candidate beside a valid manifest | Normal open uses the committed manifest; opening and a later save preserve the candidate byte-for-byte. Recovery candidates are **not automatically promoted**. This fixture models an orphan; it is not a physical power-loss test. |
| Tolerant read with valid, malformed and orphan clip entries | Existing policy retains the valid clip, filters malformed/orphan entries, preserves unknown fields and leaves the original source file untouched during open. The test deliberately does not resave normalized entries. |
| Partial write, Linux exact-test subprocess | `RLIMIT_FSIZE=256` forces the save's temporary-file write to fail after a partial write. The old manifest remains identical/readable, the owned temporary is removed, and a valid retry with an older ticket succeeds because the failed newer ticket did not advance ordering. This is EFBIG-style fault injection, **not** an actual disk-full/ENOSPC test. |
| Existing save tests | Focused package regressions include concurrent saves, stale snapshots, failed saves, normal save/reopen, document migrations, unknown-field retention, API project editing and existing real export regressions. |

The four new integration tests passed. The complete relevant package run passed **341 tests, zero failures, one ignored** (OpenGameArt live-service integration, not persistence coverage). Strict Clippy and formatting passed. Counts from the focused reruns are not added to 341 as distinct tests.

| Check | Exact command (from `src/`) | Actual result | Wall time |
|---|---|---|---|
| future-manifest-before-fix | `cargo test --offline --locked -p concat-host --lib -j 1 session::tests::a_newer_settings_manifest_cannot_be_opened_and_saved_as_an_empty_edit -- --exact --nocapture` | Exit 101; 0 passed, 1 failed, 0 ignored | 7.54 s |
| future-manifest-after-fix | `cargo test --offline --locked -p concat-host --lib -j 1 session::tests::a_newer_settings_manifest_cannot_be_opened_and_saved_as_an_empty_edit -- --exact --nocapture` | Exit 0; 1 passed, 0 failed, 0 ignored | 4.4 s |
| metadata-before-fix | `cargo test --offline --locked -p concat-host --lib -j 1 session::tests::unknown_settings_manifest_fields_survive_the_first_and_later_saves -- --exact --nocapture` | Exit 101; 0 passed, 1 failed, 0 ignored | 4.21 s |
| metadata-after-fix | `cargo test --offline --locked -p concat-host --lib -j 1 session::tests::unknown_settings_manifest_fields_survive_the_first_and_later_saves -- --exact --nocapture` | Exit 0; 1 passed, 0 failed, 0 ignored | 4.13 s |
| persistence-matrix | `cargo test --offline --locked -p concat-host --test persistence -j 1 -- --test-threads=1 --nocapture` | Exit 0; 4 passed, 0 failed, 0 ignored | 4.31 s |
| task21-regressions | `timeout --kill-after=10 1200 cargo test --offline --locked -p concat-host -p concat-project -p concat-api --all-targets -j 1 --no-fail-fast -- --test-threads=1` | Exit 0; 341 passed, 0 failed, 1 ignored | 59.67 s |
| task21-clippy | `cargo clippy --offline --locked -p concat-host -p concat-project -p concat-api --all-targets -j 1 -- -D warnings` | Exit 0 | 8.85 s |
| task21-format | `cargo fmt --check` | Exit 0 | 0.98 s |

## Remaining risks and unverified work

The tolerant loader's established policy can omit malformed/orphan data from a normalized document. Opening does not rewrite the source, but an eventual explicit save of normalized state is not an automatic recovery backup. No backup/quarantine policy was introduced or claimed. Cross-process save ordering, physical power loss, actual full disks, Windows filesystem/device behavior are not qualified by these local tests. Process interruption, UI notifications and controlled concurrent close behavior were subsequently verified in tasks 2.3/2.4, within the limits recorded below.

## 2.2 S1 investigation

### Guarantee at the initial checkpoint and primary sources

At the initial checkpoint, the app flushed the temporary file before rename without synchronizing the containing directory. A pre-commit write failure preserves the old manifest; successful replacement produces a readable new document. Directory-entry persistence across power loss is not confirmed by this protocol. The subsequently approved implementation is recorded below.

Linux documents that flushing a file alone does not necessarily persist its directory entry; the directory requires its own sync. This explains S1 but does not qualify a particular disk's behavior during actual power loss. [Linux fsync(2)](https://man7.org/linux/man-pages/man2/fsync.2.html). Rust 1.93.1 describes `File::sync_all` as attempting to flush content and metadata and exposing errors that file drop ignores. [Pinned Rust documentation](https://doc.rust-lang.org/1.93.1/std/fs/struct.File.html#method.sync_all).

Windows requires distinct directory-handle opening flags, and flushing has access-right requirements. These references are **not** evidence that a proposed directory-flush strategy works on Windows; native implementation and tests remain necessary. [CreateFileW directories](https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-createfilew#directories), [FlushFileBuffers](https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-flushfilebuffers).

### Disposable protocol investigation

`durability_probe.py` is an external investigation script in the audit artifact directory; it neither calls nor changes application saving. It uses a unique, automatically removed directory under `/home/turotulco/Work/clovercut-phase2-stability/`, on **Btrfs**, rather than `/tmp`, which is **tmpfs** on this host.

| Probe | Actual result |
|---|---|
| Sync file → replace manifest → sync directory | Success on the tested Btrfs directory. This is syscall/capability evidence, not power-loss simulation. |
| Replace manifest, then deliberately sync a closed positive descriptor | Injected `EBADF`; the new manifest remained visible and readable. This is a controlled protocol example, not a spontaneous Btrfs error. |
| Directory has owner write/execute permissions without read permission | Ordinary read-only directory open returned `EACCES`, while writing a temporary and replacing the file still succeeded. A mandatory directory-open precondition could therefore reject a replacement that currently works. |

The verified probe exited 0 in 0.03 s (`durability-probe-verified.json` and `durability-probe-results.json`). Its first exploratory attempt exited 1 because Python rejected a negative descriptor before any `fsync` syscall; the harness was corrected to use a closed positive descriptor. Both receipts are retained. This harness correction is not an application defect or a passing durability test.

### Required commit/outcome distinction

Before Option B, source review showed `projects::save_ordered` advanced `state.written` only after `write_manifest` returns success. `Studio::save` treats any error as unsaved/dirty; `save_and_release` stops close on any error; `Session::save` reduces the saved result to `Result<(), String>`. The original `Saved` enum distinguished only `Written` and `Superseded`.

**Implementation inference from the source plus probe:** if directory sync is appended after rename and its error follows the existing error path, the newest document may already have replaced the old one while its ticket is not recorded. A delayed older ticket could then be permitted to overwrite it. An approved change must record ordering at successful rename, even if durability confirmation later fails. It must preserve the committed new document and distinguish a pre-commit failure from a post-commit durability warning. No rollback to an older snapshot should be attempted automatically.

This is a design risk demonstrated by an isolated syscall sequence and code reasoning, **not** a claim that the unchanged application has that post-directory-sync bug: directory sync is not currently present.

### Alternatives requiring approval

| Option | Save and close behavior | Compatibility / limitations |
|---|---|---|
| **A — retain current contract** | Keep file sync + atomic replacement; document S1 and retain existing close/error behavior. | No stronger directory-entry durability guarantee. S1 remains an accepted limitation. |
| **B — attempt directory sync with explicit warning (recommended)** | Add directory sync on qualified Linux filesystems. A successful rename always advances its ticket. If directory open/sync cannot confirm durability, preserve the new manifest, return an explicit committed-but-unconfirmed outcome, surface a visible warning and allow close. Pre-commit failures still preserve the previous manifest and block ordinary close. | Preserve logical save compatibility while making reduced confirmation visible. Adapt autosave, synchronous close, `Session` and API callers rather than silently dropping a new outcome. Windows keeps its documented current contract until a native strategy is qualified. No automatic retry loop or format change is proposed. |
| **C — require durability confirmation** | Require directory access/sync and prevent ordinary close while confirmation is unavailable. Still advance the ticket once rename succeeds and report an explicit committed-but-unconfirmed outcome; never describe it as an untouched old manifest. | Stronger policy can refuse writable projects/filesystems or block close after a committed replacement. Requires explicit user-facing retry/quit behavior and platform qualification. |

**Recommendation for approval:** B, with a visible warning and close allowed after a committed-but-unconfirmed save, initially qualified on Linux. A platform/FS failure must not masquerade as complete durability confirmation. The user subsequently approved B; its implementation and verification are recorded below.

For either stronger option, the initial scope must be explicit. `projects::create` currently writes the first settings-only manifest with `std::fs::write`; creating project directories also introduces directory entries in their parents. Syncing an existing project's manifest directory alone must not be advertised as a complete new-project creation or power-loss guarantee. Network filesystems, storage hardware and Windows remain unqualified.

### Verification required after an approved implementation

- Keep existing ordered-save, concurrency, partial-write and legacy-name regressions passing.
- Inject replacement/write failure before commit and directory-open/directory-sync failures after commit separately.
- After a post-rename failure, verify the new manifest is readable, the new ticket prevents stale writes, owned temporaries are cleaned up and the result identifies unconfirmed durability.
- Verify a later retry/confirmed sync clears the warning as designed, without losing pending edits.
- Exercise autosave, API and close/switch notifications, with the approved close policy; never silently ignore the new outcome.
- Verify the declared Linux filesystem cases; document Windows/other unsupported targets and new-project creation scope independently.

## Acceptance status and stopping point (historical checkpoint before approval)

| Phase 2 acceptance criterion | Current evidence / status |
|---|---|
| Newer/corrupt documents are not opened as empty and overwritten | Tested rejection cases passed; future settings-only bypass fixed as S2. Tolerant partial loading remains an explicit policy limitation rather than a blanket corruption/recovery guarantee. |
| Stale snapshots cannot replace newer saved snapshots | Existing single-process concurrent/late-ticket tests passed; failed newer writes do not advance ordering. Cross-process ordering is not guaranteed by this protocol and was not qualified here. |
| Failed saves retain a readable document | Verified for tested pre-commit failures, including partial-write fault injection. Post-rename directory-sync behavior is a pending design, not implemented or claimed. |
| Jobs can close/cancel without stale mutation or crash | Not established: required task 2.3 and remaining 2.4 work are not started. |
| Durability guarantees are explicit | Current limit and A/B/C alternatives are documented; the stronger policy awaits approval. |

**Phase 2 is not complete and does not yet satisfy all acceptance criteria.** Task 2.1 is complete and independently verified. Task 2.2 is ready for review but not complete pending the user's durability decision. Because the user required each task to be completed before the next, work stops here; tasks 2.3/2.4 and Phase 3 are not started.

The approval gate comes directly from the user's instruction: “If stronger durability requires a design decision, document the alternatives and stop for approval before implementing that change.” No extra permission policy or skill requirement was inferred.


## 2.2 Approved Option B implementation (complete)

The user explicitly approved Option B after reviewing the alternatives above. The preceding investigation and stopping-point sections describe the earlier checkpoint; this implementation supersedes their pending-approval status. Phase 3 remains unauthorized.

- `save_ordered` retains the per-project lock through file write, file sync, rename and directory sync. Successful rename publishes the ticket before attempting directory sync. Pre-commit errors remain `Err` and preserve the prior manifest.
- Linux opens the project directory and calls `File::sync_all`. Open/sync errors return `Ok(Saved::CommittedButUnconfirmed(message))`; there is no rollback, automatic retry, or unsaved/dirty classification for this outcome. An outstanding warning is retained under the existing lock so a superseded save cannot turn it into a misleading ordinary success. A later successful save/sync clears it.
- Other platforms, including Windows, retain the existing file-sync/rename contract. `Written` there does not claim directory-sync confirmation. No Windows directory operation or new replacement/retry behavior was introduced.
- `Session::save` and template instantiation return the explicit outcome. The API forwards optional `saveWarning` in successful save/close replies and, when necessary, alongside the existing template-created view. Normal successful reply JSON remains unchanged (empty object for save/close). This is an additive response field, not a project-format change; embedded Rust callers now receive the typed outcome and all workspace callers are checked.
- Autosave/manual save show the warning through the existing visible toast and retain any edits made after capture. Close/switch show a Linux native acknowledgment dialog before release, because a toast alone would disappear on process exit. Acknowledging the warning allows close; it does not offer a refusal/retry loop. The test observes the reporter callback rather than opening a modal dialog unattended.
- Initial project creation/ancestor-directory durability is outside this improvement. No full power-loss, network filesystem, Windows native, or hardware qualification is claimed.

Focused durability tests passed (four): independent blocked replacement before commit, injected directory-open/directory-sync errors after commit, ordering/manifest/temp preservation, confirmed retry, real native Linux directory sync, and a real EACCES directory-open failure through API save/close. The actual API keeps the new manifest and permits close with a warning. The first focused build failed because its new test fixture incorrectly supplied an `AppDirs.cache` field; that harness compile error was corrected. Subsequent configured focused runs passed; all receipts are retained.

Task 2.2 completion evidence: `task22-regressions` passed 345 tests, zero failures, one ignored (49.79 s); `task22-clippy` passed (13.94 s); format passed (2.06 s). The GUI test executable was rebuilt (352.12 s), and `studio::save_durability_ui_tests::committed_warning_preserves_edits_and_allows_close` passed explicitly (one test, 1.06 s). An initial invocation referenced the earlier executable and selected zero tests; it is retained but **not counted as a test pass**. Corrected executable: `concat-84670db89054ae4f`. The reporter callback is exercised; native-dialog appearance itself is not automated. Actual host/API tests also passed with disposable Btrfs fixtures (0.47/0.35 s), in addition to the default tmpfs run. Task 2.3 began only after these checks completed.

## 2.3 Concurrent lifecycle verification (complete)

The initial graphical child-process scenario passed (one test, 1.11 s): real delayed ordered save, a real decoded frame held before delivery, a model-style background callback, actual stabilization analysis and an actual GPU-backed export overlap switching from disposable A to B. Old project results are observed being dropped by event-loop delivery (destructor signals), not just inferred from an epoch counter; none mutate B. A's latest track edit is saved, project ownership moves, playback stops, stabilization cancels and releases its slot. Export remains independent of the project switch, then production `stop_exports` cancels it at shutdown, preserves an existing disposable destination, removes owned staging and frees the slot. The child exits through production `leave`; the parent checks both reopened manifests and output preservation.

The test was then strengthened before task completion: a real blocked replacement must refuse switching without cancelling jobs/releasing A, and autosave, native decoder and background work must also already be running before B's close begins (not merely at final process exit). That stronger version was compiled and passed before task 2.3 completion (see its completion record below). Native model inference itself is not qualified: no model download is required or authorized by this fixture. The actual stabilization service plus the common project-gated delivery path are exercised; model-specific native inference shutdown remains an explicit limitation.

Existing compiled-window checks also passed: `task23-existing-close` selected 78 tests (the substring filter was broader than three close tests), including real failed-close/retry, late autosave and no-relaunch-on-failure. This is recorded as 78 tests, not as three exact-filter invocations. No additional production defect has been reproduced in 2.3 so far.

### Worker ownership and shutdown scope

| Work | Owner / delivery | Switch / shutdown policy retained |
|---|---|---|
| Autosave | Process worker pool; snapshot path and ticket captured on UI thread | Synchronous close captures the latest ticket. Old writes are superseded; project-epoch delivery suppresses callbacks after release. |
| Decoding / preview / artwork | Monitor/readers and process workers/scheduler; project-gated UI publication | Clear source/monitor/view state and advance epoch. Work already inside a native decoder need not be joined before process exit. |
| Stabilization / project analysis | Service-owned SingleFlight slot and cooperative cancel flag | Release requests cancellation; controlled stabilization test observes cancellation/slot release. Cached partial work is not deleted as recovery data. |
| Model-style result delivery | Common `spawn_in_project` guard | Late results are dropped before touching the next Studio. Actual Whisper/ONNX inference/device behavior remains unqualified by this fixture. |
| Export | Independent host-wide slot and snapshot; intentionally not project-gated | Continues across a project switch; final `stop_exports` cancels and waits up to the existing 10-second patience. Tested video-stage cancellation preserves the existing destination. |
| Process-lifetime GPU/audio/worker threads | Host/process | Existing immediate exit is retained; no broad joining or teardown refactor. A stuck operation can outlast the export patience; abrupt exit may leave staged recovery files. Synchronous audio mixing/mux cancellation latency remains the Phase 7 E2 qualification gap. |

These are existing ownership rules, not a new architecture. Lifecycle tests deliberately coordinate the concurrency so they exercise the boundary instead of relying on a fast job to happen to overlap a switch.

Task 2.3 completion: strengthened `task23-lifecycle-overlap` passed one explicit graphical test (1.22 s; actual child scenario 0.86 s). The log reports native video export cancellation/slot release in **20.10 ms** for this small controlled fixture, not a universal latency guarantee. The test independently verifies pre-commit switch refusal, retains A's ownership/jobs, retries the switch, observes three stale results dropped, overlaps another autosave/decoder/background worker with B's close, verifies late B save is superseded, cancels the export through production shutdown, and exits while native workers remain alive. Parent reopening confirms five tracks in both saved projects and unchanged existing output, with no staged export siblings remaining. No new production defect was reproduced; no lifecycle behavior was refactored. Formatting/diff checks passed. Task 2.4 starts after this independently verified completion.

## 2.4 Save-error visibility and interruption verification (complete)

No new production crash/error has been reproduced by the completed lifecycle tests. The two confirmed data-loss defects remain S2/S3, each independently reproduced/fixed/tested in 2.1; S1 is the explicitly approved durability improvement, not claimed as a reproduced physical power-loss incident.

The actual Studio warning test is extended to cover blocked pre-commit replacement: it must keep the session, edit and ownership, publish a visible error, then save/reopen successfully on retry. A separate uncommitted candidate must remain unchanged. The test also makes a real new edit after the warning snapshot is captured and checks that warning delivery does not clear that edit's dirty state. The warning starts with its durability status before the path, so the existing toast's elision cannot hide the warning behind a long project path. This remains the approved warning behavior, not a new UI redesign.

A new Linux disposable-process test observes a nonempty unique save temporary, then SIGKILLs only its exact-test child. The child validates the canonical temporary-root prefix and a fixture marker before writing. The parent always reaps the child, verifies the previous manifest is byte-for-byte preserved/readable, verifies the orphan remains, then reopens/saves and checks the orphan is still unchanged. It tests actual process interruption, not storage/power loss. The new interruption test passed in the final package run; all five persistence integration tests passed.


### Final verification after session recovery

The interrupted task 2.4 compilation was rebuilt with the repository native-library environment (`set -a; source ../.env.local; set +a`, from `src/`). An initial recovery invocation omitted that environment and was cancelled; it is not counted as a successful build. No additional application-code change was needed during recovery.

All commands below ran from `src/`. Exact command arrays, exit codes and logs are retained under `/home/turotulco/Work/clovercut-phase2-stability/` with the receipt names shown. All final receipts exited zero.

| Receipt | Command / scope | Actual result |
|---|---|---|
| task24-ui-build-configured | `cargo test --offline --locked -p concat --lib --no-run -j 1` | Compiled in 365.00 s. |
| task24-ui-warning | `timeout --kill-after=10 120 target/debug/deps/concat-84670db89054ae4f studio::save_durability_ui_tests::committed_warning_preserves_edits_and_allows_close --exact --ignored --nocapture --test-threads=1` | 1 passed, 1.00 s; blocked replacement retains session/ownership, visible error, retry/reopen, recovery candidate preservation, later edits remain dirty, visible unconfirmed warning and close allowed. |
| task24-lifecycle-final | Same executable/timeout/flags, exact test `studio::lifecycle_tests::switch_and_shutdown_with_running_work` | 1 passed, 0.86 s; measured fixture export shutdown 20.08 ms. |
| task24-ui-regressions | `timeout --kill-after=10 180 target/debug/deps/concat-84670db89054ae4f --test-threads=1` | 166 passed, 0 failed, 3 ignored, 0.73 s. Two ignored tests ran explicitly above; the pre-existing full graphical playback test remains deferred to Phase 4. |
| task24-regressions | `cargo test --offline --locked -p concat-host -p concat-project -p concat-api --all-targets -j 1 --no-fail-fast -- --test-threads=1` | 346 passed, 0 failed, 1 unrelated live-service test ignored, 67.91 s; includes 21 export tests and all 5 persistence tests, including actual SIGKILL interruption. |
| task24-clippy | `cargo clippy --offline --locked -p concat-host -p concat-project -p concat-api -p concat-server -p concat --all-targets -j 1 -- -D warnings` | Passed for selected workspace packages. Existing patched whisper-rs-sys dependency warnings are emitted under Cargo dependency lint caps, not silently treated as workspace fixes. |
| task24-format-final | `cargo fmt --check` | Passed, 1.15 s. |

`git diff --check` also passed. Test counts are per run; earlier focused reruns are not added as distinct tests. Final executions cover 514 passing tests (346 package + 166 UI + 2 explicit graphical), with the ignored coverage described above.

## Final Phase 2 acceptance and limitations

| Acceptance criterion | Final evidence |
|---|---|
| Newer/corrupt documents are not overwritten as empty edits | S2 regression and current/legacy rejection matrix pass. Established tolerant partial normalization remains explicitly documented. |
| Stale saves cannot replace newer snapshots | Ordered/concurrent saves, pre/post-commit failure tests and late lifecycle saves pass within the existing single-process ordering contract. |
| Failed saves preserve the last readable document and recoverable data | Partial-write, blocked replacement, SIGKILL interruption and recovery-candidate checks pass. Post-commit sync failure retains the committed new bytes. |
| Switch/close with jobs does not apply stale project results or crash | Controlled native decode, autosave, stabilization, background delivery and export scenario passes, including refused switch and shutdown. |
| Durability guarantees are explicit | Approved Option B is implemented: Linux directory sync, committed-but-unconfirmed warning, ticket publication at rename and close allowed; Windows contract preserved. |

**Phase 2 tasks 2.1–2.4 satisfy their local acceptance criteria and are complete within the declared qualification scope.** No new approval decision remains within Phase 2. Phase 3 is not started; wait for explicit user authorization.

Remaining limits: no physical power-loss/storage-hardware test, actual ENOSPC/full-disk test, network filesystem or native Windows qualification; new-project/ancestor-directory durability and cross-process ordering remain outside this change. Real model inference shutdown and full graphical playback/A/V qualification remain unverified. Native warning-dialog appearance was not automated (the production reporter path is exercised through its callback). Existing export audio-mix/mux cancellation latency and stuck native operations beyond shutdown patience remain later qualification work. Orphans are preserved rather than automatically promoted or deleted. No project format/dependency/lockfile change, commit or push was performed.
