# Concat follow-up review — 25 October 2026

Reviewed `jub0t/Concat@2429c0ca` against the previous review point `971ad1d9`:
11 new commits. This is a selective review, not an upstream merge.

| Commit | Change | Clovercut assessment |
|---|---|---|
| `638b0432` | Write Save Audio As WAV as PCM | Priority candidate: reproduced locally. Current mixer unconditionally selects AAC and float-planar samples, even for `.wav`. The resulting file reports AAC and did not decode an audible tone in Clovercut. Adapt to our cancellation-aware mixer, preserving fades, volume keys, tempo and stream selection; do not transplant upstream's newer audio-bus mixer. |
| `b5042b5c` | Seek trimmed audio backwards on its own stream | Our existing one-second preroll already passes all four new MPEG-PS/TS regression cases, both in preview and saved M4A: tone onset 0.000125 seconds after the cut. Keep the current solution for now. Stream-specific seeking may still improve robustness/performance in other containers, but these fixtures do not demonstrate a remaining Clovercut bug. |
| `24f05885` | Bottom-aligned waveform bars | Optional visual improvement compatible with the CapCut-inspired direction. Would need coordinated path, hot-level overlay and silence-baseline changes, preserving our waveform caching and clip sizing. |
| `20dcd9aa`, `bc05ea3c`, `758da5a8` | Typography case, decoration and hover labels | Bold, italic, underline and translated hover labels already exist here. Text case and strikethrough would be additions. Adapt only missing behaviour, preserving vertical text, captions and current controls; documentation-only pieces need no separate port. |
| `64b25e90` | Per-word rich text | Potentially useful, but changes text model, rendering, editing, caches and MCP. Requires deliberate compatibility and copy/paste/caption tests; not a small inspector patch. |
| `643bbe1f` | Project-wide frame background colour | Distinct from existing clip background controls. Useful future feature, but touches document schema, preview, export and new/project dialogs. Requires renderer/export parity coverage. |
| `4e207179` | Put wheels/curves after scalar adjustments | Conflicts with our deliberately compact Color/Curves/Wheels navigation and collapsible groups. Preserve our layout. |
| `b44bbf5e` | Remove the one-card Titles shelf | Library redesign, not a correctness fix. Review against our own text categories and quick title insertion before considering it. |
| `2429c0ca` | Formatting and canvas callback wiring | Relevant wiring belongs to the new canvas feature above; Clovercut does not have that schema yet. No standalone change needed. |

## Reproduction

A small external harness in
`/home/turotulco/Work/clovercut-upstream-review-2026-10-25` links the actual
unchanged Clovercut media crate, using the existing quick profile/cache. It
loads upstream's `sound-behind-the-picture.mpg` and `keyframes-3s-apart.ts`
fixtures at cuts 1/3.5 and 1/4 seconds respectively. All preview/M4A cases
start the tone at 0.000125 seconds; every WAV round-trip reports no tone.
FFprobe confirms `codec_name=aac` in the WAV and reports an AAC decode error.
Log: `/tmp/clovercut-upstream-audio-review.log`.

No upstream production changes were applied by this review. The timeline
selection, white playhead and grading translations are Clovercut-specific work.
