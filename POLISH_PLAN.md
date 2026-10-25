# Clovercut polish — completed programme

The eight-phase polish programme is complete. Its superseded investigation logs,
phase reports and audit screenshots are available in Git history. This file does
not impose historical stop/approval gates on current development.

Maintained documentation is indexed in [docs/README.md](docs/README.md).
Architecture and timing boundaries are described in [src/README.md](src/README.md).

Regression coverage remains in the source tree:

- Project durability, ownership, atomic editing and clipboard tests in `concat-project` and `concat-host`.
- Timeline precision and interaction regressions in `phase3_timeline_tests.rs`.
- Playback clock, recovery and preview/export parity suites in the `phase4_*` modules.
- Isolated graphical walkthrough in `phase5_ui_walkthrough.rs`.
- Performance and lifecycle budgets in `phase6_performance_tests.rs`.
- Export precision, cancellation and destination preservation in `phase7_export_reliability_tests.rs`.

Keep these tests and the applicable CI checks when changing the corresponding
behaviour. Machine-specific historical timings are not current performance or
cross-platform guarantees. Consult [Concat adaptations](docs/UPSTREAM.md) for
selectively imported changes and deliberate Clovercut differences.
