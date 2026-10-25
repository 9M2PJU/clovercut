# Timeline selection and grading labels — 25 October 2026

Desktop clip presses select the clip and expose its inspector without seeking.
An empty-lane click or selection box changes selection without moving the
playhead. Seeking remains on the time ruler and its reserved strip. The desktop
ruler is 44 pixels high, with labels in the first 28 pixels and a 16-pixel strip
below. Track headers and meters use the same boundary; phone-specific ruler
height and gestures remain unchanged.

The default playhead is white in both palettes. A subtle dark edge preserves
contrast over bright clips and the light theme. Explicit custom playhead colours
remain respected.

Standard colour controls are localized at the display boundary: 20 scalar,
wheel and curve names, plus the Lightness heading. Other headings reuse existing
translations. Parameter keys, group identifiers, keyframes and group-reset
callbacks stay unchanged. Labels react to language changes without rewriting
project data. The locale checker compares grading keys with the built-in Adjust
manifest so new parameters cannot silently bypass translation again.

## Validation

- Slint interface generation passed after all changes.
- Strict validation passed for 1052 keys in 25 languages.
- Nine locale-checker regressions passed, including missing/new and removed
  grading parameter coverage.
- Existing quick build with nice15/ionice7 and temporary 24 GiB Btrfs swap at
  priority150 passed in 21m51s. Profiles, jobs and CGUs were unchanged.
- Isolated startup smoke passed with Radeon RX6600XT Vulkan and Whisper.
- An isolated synthetic-project window confirmed the white playhead and larger
  ruler. Pointer interactions were reviewed in the handlers, not claimed as
  automated GUI click tests. The new app was opened for user testing after the
  prior app instance had already closed.

Binary: `src/target/quick/clovercut`, 392448408 bytes, version0.1.6.
SHA-256: `b92b2fb6e1023b3bb1df4ba01dad6df7f16143a267f1b29709db043a6b4a3be8`.
Logs: `/tmp/clovercut-ruler-grading-build.log` and
`/tmp/clovercut-ruler-grading-smoke.log`.

See [the upstream review](UPSTREAM_REVIEW_2026-10-25.md) for the eleven new
Concat commits and the audio regression results; no upstream production changes
were included in this UI patch.
