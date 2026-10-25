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
clip insertion orders. The final overlay and sidebar scroll still require a
completed build and graphical validation. The existing executable contains the
initial layout changes, **not** the transition fix or sidebar scroll. No release
has been created.
