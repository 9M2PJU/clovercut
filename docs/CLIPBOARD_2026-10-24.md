# Clovercut clipboard integration — 24 October 2026

Adapts the intent of Concat `1d04d147` to Clovercut's typed tracks, caption
collections, embedded subtitle library and project-owned generated media.
Global audio layers (`2606aff1`) remain separate work.

## Behaviour

- Copy, paste and duplicate use a complete Clip snapshot. Masks, cutout strokes,
  stabilization, Enhance references, backgrounds, audio streams, visual motion,
  transforms, speed curves, keys, disabled effect chains and unknown extra
  fields are retained. The old hand-maintained copy_commands list is removed.
- The clipboard captures only referenced media, including Enhance's original
  and processed media and image backgrounds, plus referenced project fonts.
  A source media ID is never interpreted directly in a different project's bin.
- Paste uses one internal, non-serializable command. It stages both Project
  and IdMint; imports, lane creation and clips enter the document atomically
  and undo/redo together. No project format or public API command is added.
- Lanes must be compatible and unlocked. Distinct copied source lanes remain
  distinct. Captions, stickers, titles and layers respect the main-track
  boundary. The whole group's timing shifts together to avoid existing clips.
- Each pasted caption collection has a fresh group ID. Copied members stay
  grouped; they cannot style captions already in the destination. Ordinary
  titles stay ordinary. Placed captions carry their own text/style and do not
  require copying unrelated SRT library assets.
- Detached audio remaps its video link only when the video is also copied.
  A transition is retained only when its neighbouring source clip is copied.
- Duplicating a selection uses that same atomic group operation after the
  selected span, preserving timing, caption membership and detached links.

## Files and asynchronous delivery

External media remain references. Files owned by the source project are copied
on a worker to destination-owned `media/pasted/paste-*/` directories before the
paste is accepted. Copying and hashing are cancellable and use a fixed 1 MiB
buffer. The staged directory is removed on failure or when a stale result is
dropped; it is retained only after document acceptance, including undo history.

Repeated imports reuse verified destination resources using source-path/content
provenance. Identical custom fonts are reused; a conflicting family does not
silently replace the destination's font. Native path canonicalization reuses
existing external bin paths, including Windows path-prefix/symlink differences.
File contents are synchronized; resource directories reuse the existing
platform-specific project synchronization policy before document publication.

The window accepts only one pending resource paste. The project epoch drops
old deliveries on close/switch, cancellation stops file work, and a changed
timeline or removed anchor prevents a paste into an unintended timeline. If an
anchor moves during preparation, placement uses its current lane and end time.
Source rows are mapped against current destination locks when the result arrives.
These are in-process clipboard operations: copy in one project, open another
project in the same window, and paste. They do not introduce an OS clipboard.

## Validation record

The initial integration build completed locally in 6m55s using the established
`cargo build --profile quick -p concat` command and temporary 24 GiB disk swap.
Final-source validation is recorded below.

Tests cover full-state copying for all six clip kinds, conflicting IDs, auxiliary
media, caption independence, typed locked lanes, neighbour links, source coverage
with speed curves, group collision placement, template slots, API exclusion,
resource ownership/deduplication, cancellation, cleanup and font conflicts.
An explicit isolated graphical test exercises Studio's real cross-project
worker delivery and group duplication, with actual disposable project files.

The pre-change regression reproduced six settings lost by the old copy helper:
mask, cutout, stabilization, background, enhanced-media references and selected
audio stream. The full-state regression now covers those attributes as well as
typed animated stickers and empty caption-group titles.

- Final engine suite: **983 passed, 9 skipped**, with GPU required, using the
  existing `quick` profile and excluding only the graphical and Android crates.
- Final UI library suite: **175 passed, 13 skipped**.
- The explicit isolated graphical clipboard test passed in **1.160 s**, including
  deletion of the destination anchor before asynchronous delivery. The test
  binary rebuilt in 4m56s after correcting the removal-command assertion.
- Production UI library Clippy with `-D warnings`: passed (9m32s).
- Component Clippy: the desktop workspace excluding the graphical and Android
  crates, all targets, GPU/Chatterbox features, `-D warnings`: passed.
- Locale inventories, model manifest and workflow Actionlint: passed.

Local validation runs one heavy graphical Rust target at a time. Checking or
compiling the normal graphical library and its test variant concurrently creates
avoidable memory pressure on this 16 GiB machine, even with temporary disk swap.
The application build keeps the existing profile, job settings and codegen units.

The final regular application build passed in **13m33s**, from source commit
`7a46bfa5`, using `cargo build --profile quick -p concat` after silently loading
`.env.local`. The isolated native startup smoke test passed on the Radeon RX
6600 XT Vulkan adapter, including Whisper startup. `--version` reports 0.1.6.

Binary: `src/target/quick/clovercut`, 389448096 bytes. SHA-256:
`709b0e13af98b12a7400bbf316eb68a1112147b9708055437d4306663fb8faba`.

The first normal-build attempt was stopped when system memory pressure became
excessive. Zram had reached 15.2 GiB before disk swap began carrying significant
data; no OOM kill was recorded. The successful attempt prioritized the temporary
24 GiB disk swap at 150 while leaving zram at 100. Cargo's profile, jobs and
codegen units were unchanged. The compiler threads also received lower CPU and
I/O scheduling priority during this attempt to favour interactive work. The
service recorded an 11.1 GiB memory peak and 8.5 GiB swap peak. This build remains
memory intensive on a 16 GiB desktop; disk swap trades build time for headroom.
