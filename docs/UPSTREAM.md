# Concat adaptations

Clovercut selectively adapts fixes from [Concat](https://github.com/jub0t/Concat).
A commit appearing in the upstream-only log may already have an equivalent
Clovercut implementation. Preserve typed tracks, caption collections, generated
media ownership, cancellation, atomic exports and Clovercut's existing UI.

## Current review

Reviewed through `94f4255f` (26 October 2026), following the previous review at
`2429c0ca`. Adaptations below are newer than the source used by the published
0.1.6 release and require a newer app build.

### Latest batch

| Commit | Change | Clovercut assessment |
|---|---|---|
| `c63c7ecf` | Direct3D effect sampling | Adapted: effect and transition helpers use explicit mip level zero, avoiding implicit derivatives in parameter-controlled loops. Keeps the existing shader contracts and single-mip pictures. |
| `23bc6899` (decoder only) | ProRes colour matrix | Adapted independently of alpha export: untagged ProRes frames inherit the stream's matrix before colour conversion. Other codecs and explicitly tagged frames retain their behaviour. |
| `e6bfd3fc` | New-project notice | Adapted in all locales, referring only to Clovercut's editable frame size and frame rate; does not promise the upstream canvas control. |
| `7db37f2a` | Enhance only the used source span; reuse identical frames | Adapted with a one-second margin, span-specific cache identity, original/copy offsets, preview alignment and rejection when a clip changes source while the job runs. Keeps cancellation-aware audio mixing/muxing, bounded decoder/encoder threads and atomic publication. Old enhanced links default to offset zero; clipboard retains the offset. |
| `f55451e6` | Default title font | Already equivalent: Clovercut's model and UI use bundled Hanken Grotesk, with regression coverage. |
| `cdbdf074`, `b613ba62` | Stable inspector labels during playback | Clovercut already derives tab choice through cached kind/batch properties and has additional caption/animation tabs. Reproduce any remaining model churn before adapting a held label model; never copy upstream's smaller tab list. |
| `aa38b9d5`, `84c0e625`, `94f4255f` | GPU Enhance and runtime engine selection | Deferred to a separate measured integration. Review the final series together: the initial CoreML preference regressed in-app performance and was replaced by runtime timing. Preserve low resource limits and CPU fallback; published Mac timings are not Clovercut measurements. |
| `fa2c5168`, `13e67ea2` | Import saved voices with metadata and automatic sample placement | Useful next feature, extending the existing saved-voice library. Adapt the modal and shelf to Clovercut and retain feature-gated speech engines. |
| `870290a0`, `23bc6899`, `470391e7` | Transparent canvas and ProRes 4444/VP9 alpha export | Defer as a coordinated feature after the previously reviewed project canvas work. Requires schema compatibility, preview/export alpha parity, codec availability and audio/container tests. The independent ProRes decoder fix above is included. |
| `b237687c` | Concat application identifier migration | Not applicable: retain Clovercut's identifier, portable mode and storage paths. |

The span regression exercises the real decode/encode/audio pipeline with a small
deterministic picture processor, so it needs no Enhance model download. It checks
picture and sound start times, output duration, exact-frame reuse and cancellation
cleanup. Separate tests cover ProRes colour, old enhanced links, toggling copies
and preservation through Clovercut's typed clipboard.

### Previous batch

| Commit | Change | Clovercut assessment |
|---|---|---|
| `638b0432` | Write Save Audio As WAV as PCM | Adapted: `.wav` (case-insensitive) now uses packed 16-bit PCM, with 1024-sample blocks and an exact final sample count. AAC remains unchanged for other destinations. Keeps Clovercut’s cancellation-aware mixer, fades, volume keys, tempo and stream selection; does not transplant upstream’s audio-bus mixer. |
| `b5042b5c` | Seek trimmed audio backwards on its own stream | Our existing one-second preroll already passes all four new MPEG-PS/TS regression cases, both in preview and saved M4A: tone onset 0.000125 seconds after the cut. Keep the current solution for now. Stream-specific seeking may still improve robustness/performance in other containers, but these fixtures do not demonstrate a remaining Clovercut bug. |
| `24f05885` | Bottom-aligned waveform bars | Adapted: bottom-aligned waveform bars and one upper hot-level band across the shared Wave component. Keeps existing peak caches, zoom/trim sampling, gain-only layout updates, compact track sizes and the existing bottom baseline. |
| `20dcd9aa`, `bc05ea3c`, `758da5a8` | Typography case, decoration and hover labels | Adapted only missing case and strikethrough controls. Reuses Clovercut’s accessible B/I/U controls and translated hover labels; preserves vertical text, caption-group/batch targeting and existing underline rendering. Case changes are display-only and reversible. The five new labels are translated in all 25 locales; no upstream inspector replacement. |
| `64b25e90` | Per-word rich text | Potentially useful, but changes text model, rendering, editing, caches and MCP. Requires deliberate compatibility and copy/paste/caption tests; not a small inspector patch. |
| `643bbe1f` | Project-wide frame background colour | Distinct from existing clip background controls. Useful future feature, but touches document schema, preview, export and new/project dialogs. Requires renderer/export parity coverage. |
| `4e207179` | Put wheels/curves after scalar adjustments | Conflicts with our deliberately compact Color/Curves/Wheels navigation and collapsible groups. Preserve our layout. |
| `b44bbf5e` | Remove the one-card Titles shelf | Library redesign, not a correctness fix. Review against our own text categories and quick title insertion before considering it. |
| `2429c0ca` | Formatting and canvas callback wiring | Relevant wiring belongs to the new canvas feature above; Clovercut does not have that schema yet. No standalone change needed. |

## Integration details

WAV uses packed PCM S16LE in 1024-sample blocks with an exact final sample count;
M4A keeps AAC. The existing mix graph retains trim, speed, gain, fades and
cancellation. Clovercut's one-second decoder preroll already passes upstream's
four new MPEG-PS/TS trimmed-audio examples.

Waveforms rise from the floor with one upper hot-level band. Peak caches,
zoom/trim sampling and gain-only layout updates are unchanged. Text casing is
applied at the shared preview/export boundary without rewriting the source or
caption transcript. Strike and case participate in caption-group/batch edits,
copy/paste, presets and the serialized title cache key. Old projects default to
Original/no strike; unknown case values fall back to Original. The existing
vertical text and underline implementation is preserved.

Regression coverage lives in `concat-media` audio tests, `concat-project` model
and clipboard tests, `concat-text` rendering tests and `concat-host` title tests.
UI formatting, strict Clippy and all 25 locale inventories are checked separately.
Hardware-specific and live-provider tests retain their explicit prerequisites.

## Earlier adaptations

| Area | Concat commits | Result |
| --- | --- | --- |
| Reader pool | `e8c546ac`, `2b6eb230` | Separate rolling readers for two cuts from one file, without opening extra readers for neighbouring prefetch requests. |
| Transition stacking | `76988c49` | A chain of transitions stays beneath logos and tracks above it, regardless of clip list order. |
| Export frame grid | `fe2be741` | Export and audio mix end on the timeline's quantised frame grid. |
| Frame plan borrowing | `167a7331` | Avoid cloning complete plans when treatments do not need them. |
| Camera/container seeking | `d02b7565`, `39ebb75e` | Recover from overshooting MPEG-TS seeks and account for coarse container timestamp precision. |
| Non-square pixels | `b8337561` | Display HDV/AVCHD/DV at their aspect-correct dimensions before rotation. |
| Variable frame rate | `1dac19b7`, `bac31a3f` | Hold the preceding picture across pauses and index moving frames by capture cadence. |
| Damaged packets | `7c113230` | Tolerate a bounded sequence of corrupt packets; still fail on a broken stream. |
| Common media origin | `93c18b72` | Preview, transcription and export preserve offsets between picture and sound. |
| Import artwork | `7b2d30ea`, `ccc6131c`, `7dc91a92` | Prioritise posters, then waveforms and strips; use suitable keyframes with an exact fallback. |
| Captions | `f1a5f4d3`, `2922614c` | Fix consecutive Whisper timestamps and tighten caption spans to detected speech using the bundled Silero VAD model. |
| Library default | `6466b9f7` | Open on All while retaining Clovercut's favourites, most-used and recent modes. |
| Search focus | `0a45c5a5` (functional part) | Enter still submits online searches, then releases focus; Escape releases focus. |
| Filter preview | `599157e7` | Label the audition in the monitor and allow dismissal by its close button, Escape or selecting a clip. |
| Text effects | `0acb767b` | Add the generic effects chain beside Clovercut's caption, style and animation tabs. |
| Encoder stalls | `0c86f62b` | A hardware encoder stall restarts the render once in software, resets progress/ETA and reports the new stage. |

Other selectively adapted changes include unnamed WAV channel layouts
(`e353fe57`), updater TLS roots/MCP annotations/VAD cache publication (`41ca63a4`),
feature-gated Chatterbox configuration (`e854f821`), font-picker Escape handling
(`3e651f4f`) and the visual-layer copying correction from `2606aff1`.
Cross-project paste adapts `1d04d147` through Clovercut's own typed-track and
project-owned resource implementation. Global SoundLayer/audio-bus schema and
Android renderer changes were not imported.

Detailed investigation logs and superseded audit reports remain available in
Git history; they are not current setup or release instructions.
