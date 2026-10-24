# Clovercut upstream follow-up — 24 October 2026

Follow-up to `UPSTREAM_ADAPTATION_2026-10-24.md`, reviewed against
`jub0t/Concat@971ad1d9`. These changes follow the already-tagged desktop 0.1.4
release; its tag was not moved.

## Adapted changes

- **`e353fe57`: unnamed WAV channels.** Assign a conventional layout only when a
  decoded frame has an unspecified channel order. Apply this to both preview
  decoding and export mixing, preserving pitch/tempo curves, stream origins,
  preroll and cancellation. Bump the PCM cache revision so filtered samples
  produced before this correction are regenerated.
- **`41ca63a4`: update metadata, MCP and VAD.** Fetch updater metadata using only
  the distributed TLS roots; keep the existing shared/native-root clients and
  redirect policies for providers and downloads. Clovercut's audio catalogue
  already queries OpenGameArt, YouTube and Freesound directly, so the Concat
  sound-manifest path was not restored. Advertise MCP `preview_frame` as capable
  of writing/replacing a file. Publish the embedded VAD model through unique
  process-and-call temporary names and validate its contents, not only its size.
- **`e854f821` (remaining relevant part).** Compile CHATTERBOX_STEP only with the
  Chatterbox feature. The previous next_back/rustfmt fixes were already present.
- **`3e651f4f` (font-picker portion).** Allow Escape to reach the containing popup.
  Retain online searches' submitted callback followed by focus release on Enter.
- **`2606aff1` (visual-layer copy correction only).** Create copied visual layers
  with AddLayerClip instead of the file-based AddClip command, then reuse the
  existing copy commands for their effects, parameter keys, strength and fades.
  Keep gesture grouping, locked-track checks and visual-track boundaries. The
  commands live in a small window-independent module so they can be tested
  against the real editor without compiling the Slint test executable.

No global audio buses/SoundLayer schema or Android renderer changes were added.
The existing eye control and caption/style/animation tabs were preserved.

## Desktop focus

README and the public site now describe Clovercut as desktop-first, with Linux,
Windows and macOS driving development and release priorities. Keyboard/mouse,
multiple tracks and full-size preview/inspector are the primary workflow.
Mobile support is described as secondary with a separate feature/release schedule.
Android remains available as a secondary previous-release link, rather than a
primary download card or mobile-specific main call to action. macOS visitors no
longer receive a Linux download from OS detection.

The current known-good Linux/Windows download URLs were retained while the next
desktop release is building; macOS points to the release list for availability.
The site's incorrect reference to an NSIS installer was also corrected.

## Validation

- Regression reproduces unnamed stereo WAV filtering in both preview and actual
  AAC export mixing: the unfiltered tone remains audible and the low-pass output
  falls below the expected threshold.
- Eight simultaneous VAD writers repair a same-sized corrupt model, publish the
  exact embedded bytes and leave no temporary parts. Actual VAD load/run on
  silence also passes.
- Real editor copy tests preserve visual-layer effects, bypass state, parameter
  keyframes, opacity, fades and empty chains; paste undo/redo works. Audio,
  below-main and missing destination tracks are refused.
- Public MCP tools/list regression checks the file-output annotations.
- 309 component library tests passed with GPU/Chatterbox enabled; five explicitly
  ignored tests require additional hardware or external inputs. Another 26 tests
  passed for layer copying, editor preferences and phase-7 export reliability.
- Strict component Clippy passed with all targets and GPU/Chatterbox enabled.
- Slint generation passed; locale/model checks and formatting passed.
- HTML parsing/unique IDs and JavaScript OS-detection checks passed for Linux,
  Windows, macOS, Android and iPhone. No browser connection was available for a
  visual screenshot review. GitHub Pages deployment succeeded and the published
  HTML was fetched and checked for the new desktop-first messages.
- Full local build and isolated smoke: pending completion; results will be
  appended when the executable is ready.
