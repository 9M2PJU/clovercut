# Desktop timeline and library layout — 25 October 2026

Development work for 0.1.6. The published desktop release remains 0.1.5.

## Behavior

The timeline toolbar's plus button creates a timeline through the existing
creation callback. With one timeline, the tab strip is omitted and the pane
switcher lives in the toolbar. With multiple timelines, tabs and their existing
selection, rename, close and reorder actions remain available; the pane switcher
returns to the tab strip. The old tab-strip plus button is removed.

Automatic track height is 52 logical pixels. Manual Small (44), Medium (80) and
Large (108) preferences keep their existing heights. Library category icons are
20 pixels with more spacing. Sidebar categories use boxed backgrounds and the
theme accent for selection; the sidebar scrolls when its content exceeds the
available height.

## Transition indicator regression

The indicator used to be a child of the incoming clip. If the outgoing clip
appeared later in the document's clip list, it painted over the indicator's left
half, including its duration handle. Project insertion order therefore affected
both the appearance and access to the controls.

TransitionOverlay now owns the existing transition wash, cut button, handles and
duration tooltip. Lanes renders its overlay repeater after the complete clip
repeater. Selection and duration gestures still address the incoming clip ID and
use the existing press, drag, release and cancellation callbacks. Track locking
still disables the transition's touch areas. The clip model retains its original
order, avoiding model-index changes during a drag.

Reproduction: create two adjacent eight-second video clips A and B, with a
two-second incoming cross-fade on B. Save otherwise identical projects with clip
arrays `[A, B]` and `[B, A]`. Before the fix, the second order hides the left half
of the indicator. After the fix, both projects must show the complete indicator
above A and B. Also check one versus multiple timelines, compact track height,
and category access in a short window.

## Validation

The initial layout build completed in 18m27s and passed the Vulkan/Whisper smoke
check. Isolated graphical launches with one and two timelines confirmed the
conditional tab strip, accessible pane switcher, compact tracks and larger boxed
library categories. That check exposed clipped categories in a short sidebar;
the scroll wrapper was added afterward.

Rust formatting, `git diff --check` and locale key validation pass. The final
combined build containing sidebar scrolling and the transition overlay did not
complete: systemd-oomd killed two attempts for memory pressure (18m16s and
18m37s), with peaks of 11.6/11.8 GiB RAM and 12.3/12.7 GiB swap. Both used the
existing quick profile, nice 15, I/O priority 7 and a temporary 24 GiB disk swap
at priority 150; the existing zram and OOM protections were unchanged. Neither
attempt produced a compiler diagnostic.

The transition regression was reproduced graphically before the change in both
clip insertion orders. A later complete build with the inspector follow-up succeeded in 15m42s and
passed the Vulkan/Whisper smoke check. Isolated graphical launches confirmed the
complete transition indicator above both clips for both insertion orders, and
one versus two timeline layouts. No release has been created.


## Inspector follow-up

- Parameter rows share a label, slider, fixed numeric field and keyframe column
  on desktop panels at least 400 logical pixels wide. Narrow panels and touch
  input keep the stacked layout. Resizing retains the same control instances,
  units, numeric stepping and double-click reset behavior.
- Inspector navigation uses accent underlines rather than nested boxed wells.
  All segmented controls expose the complete option name through a tooltip.
- LUT selection has the available row width; `.cube` import is a secondary icon
  with its existing label and tooltip. Multiple applied LUTs and their independent
  toggles, removals and intensity controls are preserved. The empty note is shorter.
- Scalar color groups can be folded independently and expose a labelled reset
  action. Reset uses existing parameter defaults, respects animation at the
  playhead, and commits the group together. It does nothing when already at defaults.
- Each inspector reports a stable content minimum from its active navigation.
  The dock applies these minimums per seat, ignores them on other pane kinds and
  proportionally fits panes when the window cannot accommodate every minimum.
  Numeric edits and hover do not change the minimum. Application is deferred to
  the event loop to avoid borrowing Studio during model publication.

The complete Slint component tree compiles with the existing build-script
compiler. Five dock tests pass, including independent inspector minimums and
narrow-window behavior, through a lightweight harness using the actual dock
source and stand-in generated geometry types. Rust formatting, locale key checks
and diff checks pass. This is not a full application or graphical test result;
the first complete application build subsequently passed in 15m42s and passed
the Vulkan/Whisper smoke check. Graphical inspection confirmed inline parameter
rows in a wide inspector, the LUT layout, underlined navigation and folding a
color group. Group reset/undo and narrow selected-inspector interactions have not been verified graphically.


## Clovercut default accent

With no saved accent choice, startup uses the brand green `#24d47a`, also used
by the logo and website. Existing named and custom choices remain authoritative;
palette indices are unchanged. This uses the existing custom-color tokens,
including hover, selection wash and foreground contrast. The startup-only build passed in 7m04s. A subsequent build with the library underline removed and refined group reset passed in 19m06s, followed by a successful Vulkan/Whisper smoke check.


## Library paging and translation follow-up

Library arrows now advance approximately one viewport, aligned to complete
category boundaries and clear of the arrow overlays. They preserve the selected
category, continuous wheel scrolling, overflow menu and resize clamping. The
complete Slint tree compiles with the existing build-script compiler.

All 25 shipped locale inventories (1,027 keys each) pass strict checking. Pending feature strings
were translated across all 24 non-English languages. Exact cognates and established
technical terms are recorded with their reviewed values in
`scripts/locale-identical.json`; changing the English does not inherit an exemption.
The checker now rejects empty/nontext entries and missing, extra or repeated
numbered placeholders. Six checker regression tests pass and CI runs them before
the strict inventory check. The two hardcoded audio-card categories now use the
existing translated category labels. Source metadata and provider names stay intact.

The combined application build passed in 19m53s. A follow-up for audio-card labels
passed in 12m01s. Graphical inspection then found hardcoded saved/unsaved status
and caption grouping menu text; these now use translated keys in every language.
The final incremental build passed in 8m55s and the Vulkan/Whisper smoke passed.
The final executable is `src/target/quick/clovercut` (391,827,000 bytes), SHA-256
`2653bfe25537bbcfae0719ad94feca40f00afd9ea9d221a5c22e4833097c45d5`.

Isolated Portuguese and German launches confirmed the library labels and layout.
A final Portuguese launch confirmed the translated saved badge. Automated mouse
interaction was inconclusive (the original method did not deliver clicks, and the
replacement lost QA-window focus), so this is not a claimed end-to-end paging,
group reset or undo test. The full Slint tree and application compile successfully.
This follow-up does not publish a release.
