# Development rules

Read `docs/AGENT_HANDOFF.md`, `POLISH_PLAN.md` and the relevant phase report before changing code. These rules apply to any editor or agent.

## Scope and Git

- Preserve the existing working tree, including untracked regression tests. Inspect status and diffs before editing.
- Follow the user's current authorization. Phase 3 is approved within its documented local scope; Phase 4 requires separate explicit authorization.
- Do not commit, push, reset, clean, discard changes or replace this working tree without explicit user approval. A proposed command is not approval to execute it.
- Fix reproduced defects in small changes with meaningful regressions. Record evidence, actual commands/results and limits in the plan and phase report. Avoid broad controller or architecture rewrites.

## Correctness and data

- Use disposable projects and isolated configuration/data/cache directories for failure, save/recovery and graphical tests. Never inject failures against real user projects.
- Preserve save-ticket ordering, unique temporary files and atomic replacement. Rename commits the new snapshot and advances ordering even when subsequent directory synchronization fails. Report committed-but-unconfirmed durability visibly; do not restore stale data or silently discard recovery candidates. Preserve the approved Windows behavior.
- Preserve version-1 document compatibility, unknown fields and rejection of unsupported newer documents. A format change needs an explicit compatibility strategy.
- Keep authoritative edit times/origins in f64 and frame picks on the exact project rate. Slint timeline coordinates are local to the last published origin; do not roundtrip absolute state through f32. Preserve continuous free-edit semantics.
- Running timeline playback selects the containing frame interval; paused picks retain nearest-frame preview semantics. Keep sample-based audio timing and the original audio clock.
- Cancellation discards gesture preview without a history entry; a completed gesture retains exact undo/redo behavior. Preserve group spacing and track-lock handling.
- Preserve GPU queue/thread ownership, project-epoch guards, transactional export staging and SingleFlight ownership. Do not replace intentional native-process shutdown behavior casually.

## Environment and verification

- Run Cargo from `src/` with the toolchain pinned in `src/rust-toolchain.toml`. Preserve crate dependency direction; core must not depend on media, host or UI.
- Respect the lockfile and local native prerequisites. Use `-j 1` for heavy builds on this machine and avoid concurrent heavy build processes.
- Never print, copy into documentation, stage or commit `.env.local`, credentials or environment dumps. Load trusted local configuration privately when needed.
- Run focused tests for each change and the relevant regressions. Ignored GUI/device tests require explicit execution with their prerequisites; zero matched tests, skips and missing prerequisites are not passes.
- Preserve failed receipts as well as successful ones. Distinguish local verification from platform, physical A/V and release qualification. Do not download models or claim device/network coverage implicitly.
