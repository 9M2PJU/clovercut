# Personal library lists

Effects, filters, transitions (including online GL Transitions) and GIPHY stickers
have a star on each card and four views: **All**, **Favourites**, **Most used** and
**Recent**. Audio effect shelves use the same controls.

Stars and usage are local preferences shared across projects and remembered
between launches. Existing starred package IDs are retained. Search works within
the selected personal view; use All to return to normal catalogue browsing.
Most used sorts by successful application count, breaking ties by recency. Recent
shows the latest 50 distinct used items. Previews, failed applications, parameter
adjustments and undo/redo do not add uses.

GIPHY personal lists persist only provider IDs and usage numbers. Selecting a
personal sticker list resolves those IDs through the provider's Get GIFs by ID
endpoint with current metadata and thumbnails. The regular search keeps GIPHY's
result ordering. Personal lists are ordered locally, support paging and search,
and require the configured GIPHY key and internet access. Leaving a pending
personal lookup invalidates its results. Only an original explicitly added to a
project becomes persistent project media.

# Content library settings

**Settings → Content libraries** groups services into **Audio**, **Stickers** and
**Transitions**. Each provider card contains its activation switch, explanation,
configuration status and, where required, a masked API-key field and save action.
Saved keys remain local and are not shown in plain text.

# Timeline insertion feedback

Moving a clip uses the original pointer grab position, rather than the track's
top edge. While the pointer remains inside the source track, a move cannot
propose creating another track. Outside it, insertion has a narrow 5 logical-pixel
boundary zone, preserving existing visual/audio track constraints. This prevents
a horizontal move or a small adjustment near the clip edge from creating tracks.
