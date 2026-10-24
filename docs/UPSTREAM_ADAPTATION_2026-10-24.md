# Concat fixes adapted for Clovercut — 24 October 2026

Baseline: Clovercut `8e60c0cd`. Reviewed upstream snapshot:
`jub0t/Concat@7dc91a924ca5657bca2523d42c256912ecb21e17`.
These are selective adaptations, not an upstream merge. The release remains paused.

## Imported changes and provenance

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

## Clovercut adaptations

- Kept typed tracks, main-track policy, asymmetric transition geometry, generated sources, caption motion, GPU recovery, cancellable audio mixing and atomic final output replacement.
- Kept continuous Rubber Band speed curves and MP3 seek preroll. Source-offset silence is inserted before retiming; a new regression checks both origin directions at 2× speed and with a cut.
- Invalidated cached PCM and waveform peaks generated against the old stream origin, and cached artwork generated before display-aspect correction. Existing files are not deleted eagerly.
- Kept online GL transitions, transition hover previews, audio/GIPHY source previews, all four personal-library modes and their translations. The upstream deletion of three inspector translation keys was excluded.
- Preserved the caption collection's default Style tab and its animation tab. Generic Effects is appended to those tabs.
- Kept cancellation checks around voice detection and Whisper. The bundled detector loads locally; no additional model download is required.

The audio-origin regression was run **before** applying its fix. For
`picture-starts-late.mov`, the tone began at 1.000 s instead of 0.500 s.
The polish playback-clock tests offset both streams equally, so they did not
cover this distinct input-file condition. The whole-file and cut regressions
pass after the fix in both the sample decoder and export mix.

## Validation

- Engine library tests: 335 passed, five explicitly ignored tests requiring external inputs/hardware.
- Application feature configuration (GPU and Chatterbox): export 48, host 137 and speech 23 passed; includes a rendered logo over a transition chain and a real load/run of the embedded VAD on silence.
- GPU integration suites: 46 passed across export, phase-7 reliability, timeline precision and editor preferences; includes cancellation and preservation of the previous destination.
- Strict Clippy (`-D warnings`) passed for media, host, export, speech and MCP, all targets, with GPU and Chatterbox enabled.
- Slint interface generation passed. Locale inventory has no missing or stale entries in the 25 bundled languages; existing untranslated English strings remain.
- Complete local build: `cargo build --profile quick -p concat` passed in 12m29s, with the existing profile and 24 GiB temporary disk swap.
- Isolated `clovercut --smoke` passed (exit 0): Radeon RX 6600 XT / Vulkan and Whisper CPU backend detected.
- Executable: `src/target/quick/clovercut`, 390,869,592 bytes, built 24 October 2026 at 10:51:59 Europe/Lisbon.

No Android device was available. The encoder state machine and retry policy
have automated coverage, but native MediaCodec recovery is not device-qualified.
Caption timing has deterministic span tests and an actual detector load/run;
perceptual quality on real multilingual speech remains a manual validation item.

## Deliberately excluded

No wholesale inspector redesign, Kotlin spike, replacement of the clip/layer
effect modes, duplicate drag-to-layer feature, or replacement of Clovercut's
library cards and hover previews. Cross-project paste and Android magnetic
policy remain separate work requiring their own Clovercut-specific design.
No CI trigger changes, version bump, tag or release were made.
