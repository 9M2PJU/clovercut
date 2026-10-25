# Desktop editing

These notes describe current development behaviour. See the
[release notes](releases/) for the features included in a downloaded package.

## Timeline

Click a clip to select it and open its inspector without moving the playhead.
Click or drag the time ruler or the reserved strip beneath its labels to seek.
Clicking empty track space or making a selection box changes selection only.
The default playhead is white; an explicit custom colour remains respected.

The timeline toolbar's plus button adds a timeline. Tabs appear when there is
more than one timeline; existing rename, close and reorder actions remain.
Automatic track height is compact; manual Small, Medium and Large sizes remain
available. Inspector width is adjustable, with its minimum derived from the
active controls. A transition indicator spans both clips at the cut.

## Text and colour

Colour adjustment names, groups, wheels and curve channels follow the selected
language without changing project parameter IDs or keyframes. Groups can be
folded and reset independently. Text supports bold, italic, underline and
vertical writing. The development branch also adds strikethrough and display-only
uppercase, lowercase and sentence case; selecting the active case again restores
the original spelling. Captions retain their source text and group editing.

## Copy, paste and duplicate

Copy a selection, open another project in the same window and paste at the
playhead. This is an in-app clipboard, shared across projects in that window.
The complete clip settings, effect/keyframe state, caption membership, selected
audio streams and generated-media references are retained. Pasted caption groups
are independent of the source; neighbour transitions and detached-audio links
are retained when their companion clips are copied too.

External media remain references. Source-project-owned files are copied into
the destination before paste is accepted; verified resources and fonts are
reused. Conflicting font families do not silently replace the destination font.
Resource copying is cancellable. Closing/switching projects or deleting the
paste anchor prevents stale worker deliveries from changing another timeline.
Paste respects current destination track types and locks. Duplicate uses the
same group operation after the selected span, as one undo step.

## Content libraries

See [audio sources](AUDIO_LIBRARY.md), [GL Transitions](GL_TRANSITIONS.md) and
[favourites and usage lists](LIBRARY_ORGANIZATION.md). Category arrows page
through groups of library icons; wheel scrolling remains available.
