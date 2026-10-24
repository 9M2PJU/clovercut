// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Clovercut contributors

//! Complete clip snapshots and atomic, typed placement of copied groups.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use crate::commands::{CommandError, IdMint, Outcome};
use crate::model::{Background, Clip, ClipKind, CustomFont, MediaItem, Project, Track, TrackKind};

/// One copied clip and its original lane position.
#[derive(Clone, Debug, PartialEq)]
pub struct CopiedClip {
    /// Position of its source lane, from the bottom upwards.
    pub row: usize,
    /// The full clip snapshot, including settings added in future versions.
    pub clip: Clip,
    /// The designated lane type, including sticker images and captions.
    pub track_kind: TrackKind,
}

/// An in-process clipboard. It is deliberately not accepted over the command API.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Clipboard {
    /// Copied clips, earliest first.
    pub clips: Vec<CopiedClip>,
    /// Only media referenced by the copied clips, including auxiliary pictures.
    pub media: Vec<MediaItem>,
    /// Custom fonts used by the copied titles.
    pub fonts: Vec<CustomFont>,
}

impl Clipboard {
    /// Capture the named clips in the active timeline and their dependencies.
    pub fn capture(project: &Project, ids: &[String]) -> Self {
        let clips = ids.iter().filter_map(|id| {
            let clip = project.active().clip(id)?.clone();
            let row = project
                .active()
                .tracks
                .iter()
                .position(|t| t.id == clip.track_id)?;
            Some((row, clip))
        });
        Self::from_clips(project, clips)
    }

    /// Capture supplied snapshots, also used for a duplicate at a new position.
    pub fn from_clips(project: &Project, clips: impl IntoIterator<Item = (usize, Clip)>) -> Self {
        let mut needed = HashSet::new();
        let mut families = HashSet::new();
        let mut clips: Vec<_> = clips
            .into_iter()
            .map(|(row, clip)| {
                for id in media_references(&clip) {
                    needed.insert(id.to_owned());
                }
                if let Some(text) = &clip.text {
                    families.insert(text.font_family.clone());
                }
                let mut kind =
                    TrackKind::from_clip_kind(clip.kind, crate::captions::is_caption(&clip));
                if matches!(clip.kind, ClipKind::Image | ClipKind::Video)
                    && project
                        .active()
                        .tracks
                        .get(row)
                        .is_some_and(|t| t.kind == TrackKind::Sticker)
                {
                    kind = TrackKind::Sticker;
                }
                CopiedClip {
                    row,
                    clip,
                    track_kind: kind,
                }
            })
            .collect();
        let mut seen = HashSet::new();
        clips.retain(|c| seen.insert(c.clip.id.clone()));
        clips.sort_by(|a, b| {
            a.clip
                .start
                .total_cmp(&b.clip.start)
                .then(a.row.cmp(&b.row))
        });
        Self {
            clips,
            media: project
                .media
                .iter()
                .filter(|m| needed.contains(m.id.as_str()))
                .cloned()
                .collect(),
            fonts: project
                .fonts
                .iter()
                .filter(|f| {
                    families.iter().any(|family| {
                        family.split(',').any(|part| {
                            part.trim()
                                .trim_matches(['\'', '"'])
                                .eq_ignore_ascii_case(f.family.trim_matches(['\'', '"']))
                        })
                    })
                })
                .cloned()
                .collect(),
        }
    }

    /// Whether the clipboard has any clips to place.
    pub fn is_empty(&self) -> bool {
        self.clips.is_empty()
    }
}

fn media_references(clip: &Clip) -> Vec<&str> {
    let mut ids = Vec::new();
    if !clip.media_id.is_empty() {
        ids.push(clip.media_id.as_str());
    }
    if let Some(enhanced) = &clip.enhanced {
        ids.extend([enhanced.original.as_str(), enhanced.copy.as_str()]);
    }
    if let Some(Background::Image { media_id }) = &clip.background {
        ids.push(media_id.as_str());
    }
    ids
}

fn refused(reason: &str) -> CommandError {
    CommandError::CannotPaste {
        reason: reason.to_owned(),
    }
}

/// Place a clipboard group on a staged document, committing only on success.
/// `shift` is relative to source rows; locked destination lanes are never used.
pub(crate) fn paste(
    project: &mut Project,
    mint: &mut IdMint,
    clipboard: Clipboard,
    start: f64,
    shift: isize,
    locked: &[String],
) -> Result<Outcome, CommandError> {
    if clipboard.is_empty() {
        return Ok(Outcome::default());
    }
    let mut staged = project.clone();
    let mut staged_mint = mint.clone();
    let outcome = paste_into(
        &mut staged,
        &mut staged_mint,
        clipboard,
        start,
        shift,
        locked,
    )?;
    *project = staged;
    *mint = staged_mint;
    Ok(outcome)
}

fn paste_into(
    project: &mut Project,
    mint: &mut IdMint,
    clipboard: Clipboard,
    start: f64,
    shift: isize,
    locked: &[String],
) -> Result<Outcome, CommandError> {
    if !start.is_finite()
        || clipboard.clips.iter().any(|c| {
            !c.clip.start.is_finite()
                || !c.clip.duration.is_finite()
                || c.clip.duration <= 0.0
                || !c.clip.source_start.is_finite()
                || !c.clip.speed.is_finite()
        })
    {
        return Err(CommandError::NotANumber);
    }
    let mut media_map = HashMap::new();
    for item in &clipboard.media {
        if item.path.is_empty() && !item.placeholder {
            return Err(CommandError::MediaGone);
        }
        let id = if let Some(existing) = project.media.iter().find(|m| {
            !item.path.is_empty() && m.path == item.path && m.placeholder == item.placeholder
        }) {
            existing.id.clone()
        } else {
            let mut imported = item.clone();
            imported.id = mint.next("m");
            let id = imported.id.clone();
            project.media.push(imported.tidy());
            id
        };
        media_map.insert(item.id.clone(), id);
    }
    // Missing snapshots must never fall back to a coincidentally equal target ID.
    for held in &clipboard.clips {
        if media_references(&held.clip)
            .iter()
            .any(|id| !media_map.contains_key(*id))
        {
            return Err(CommandError::MediaGone);
        }
    }
    for font in &clipboard.fonts {
        if let Some(existing) = project.fonts.iter().find(|f| f.family == font.family) {
            if existing.path != font.path {
                return Err(refused(
                    "A font with the same family but a different file is already in this project.",
                ));
            }
        } else {
            project.fonts.push(font.clone());
        }
    }
    let mut rows: Vec<_> = clipboard
        .clips
        .iter()
        .map(|c| (c.row, c.track_kind))
        .collect();
    rows.sort_by_key(|(row, kind)| (*row, kind_order(*kind)));
    rows.dedup();
    let mut destinations = Vec::new();
    let mut floor = 0;
    let mut assigned = HashSet::new();
    let timeline = project.active_mut();
    for (source_row, kind) in &rows {
        let preferred = source_row.saturating_add_signed(shift).max(floor);
        let existing = (floor..timeline.tracks.len())
            .filter(|&row| {
                let track = &timeline.tracks[row];
                track.kind == *kind
                    && !locked.contains(&track.id)
                    && !assigned.contains(&track.id)
                    && (*kind != TrackKind::Filter || row >= timeline.main_track_index())
                    && (!matches!(
                        kind,
                        TrackKind::Text | TrackKind::Captions | TrackKind::Sticker
                    ) || row > timeline.main_track_index())
            })
            .min_by_key(|row| row.abs_diff(preferred));
        let row = existing.unwrap_or_else(|| {
            let row = if *kind == TrackKind::Audio {
                floor.min(timeline.main_track_index())
            } else {
                timeline.tracks.len()
            };
            timeline.tracks.insert(
                row,
                Track {
                    id: mint.next("t"),
                    kind: *kind,
                    visible: true,
                    ..Track::default()
                },
            );
            // Insertion can shift earlier mappings: keep IDs, never row indices.
            row
        });
        assigned.insert(timeline.tracks[row].id.clone());
        destinations.push(timeline.tracks[row].id.clone());
        floor = row + 1;
    }
    let origin = clipboard
        .clips
        .iter()
        .map(|c| c.clip.start)
        .fold(f64::INFINITY, f64::min);
    let mut base = start.max(0.0);
    // Shift the group as a whole to the first gap that fits all its lanes.
    loop {
        let mut next = base;
        for held in &clipboard.clips {
            let row = rows
                .iter()
                .position(|r| *r == (held.row, held.track_kind))
                .unwrap();
            let offset = held.clip.start - origin;
            let at = base + offset;
            for old in timeline
                .clips
                .iter()
                .filter(|c| c.track_id == destinations[row])
            {
                if at < old.start + old.duration - 1e-8
                    && old.start < at + held.clip.duration - 1e-8
                {
                    next = next.max(old.start + old.duration - offset);
                }
            }
        }
        if next <= base {
            break;
        }
        if !next.is_finite() {
            return Err(CommandError::NotANumber);
        }
        base = next;
    }
    let ids: HashMap<_, _> = clipboard
        .clips
        .iter()
        .map(|h| (h.clip.id.clone(), mint.next("c")))
        .collect();
    let mut groups = HashMap::new();
    let mut used_groups: HashSet<_> = timeline
        .clips
        .iter()
        .filter_map(|c| c.text.as_ref()?.caption_group.clone())
        .collect();
    let mut created = None;
    for held in &clipboard.clips {
        let mut clip = held.clip.clone();
        let row = rows
            .iter()
            .position(|r| *r == (held.row, held.track_kind))
            .unwrap();
        clip.id = ids[&clip.id].clone();
        clip.track_id = destinations[row].clone();
        clip.start = base + (clip.start - origin);
        if !clip.media_id.is_empty() {
            clip.media_id = media_map[&clip.media_id].clone();
        }
        if let Some(enhanced) = &mut clip.enhanced {
            enhanced.original = media_map[&enhanced.original].clone();
            enhanced.copy = media_map[&enhanced.copy].clone();
        }
        if let Some(Background::Image { media_id }) = &mut clip.background {
            *media_id = media_map[media_id].clone();
        }
        clip.detached_from = clip
            .detached_from
            .as_ref()
            .and_then(|old| ids.get(old))
            .cloned();
        // Only preserve a transition if its source neighbour is copied too.
        let previous = clipboard.clips.iter().any(|other| {
            other.clip.id != held.clip.id
                && other.row == held.row
                && (other.clip.start + other.clip.duration - held.clip.start).abs() < 1e-6
        });
        if !previous {
            clip.transition_in = None;
        }
        if let Some(style) = &mut clip.text
            && let Some(group) = &mut style.caption_group
            && !group.is_empty()
        {
            *group = groups
                .entry(group.clone())
                .or_insert_with(|| {
                    loop {
                        let id = mint.next("cg");
                        if used_groups.insert(id.clone()) {
                            break id;
                        }
                    }
                })
                .clone();
        }
        if clip.text.is_none() {
            clip.name = format!("{} copy", clip.name);
        }
        created = Some(clip.id.clone());
        timeline.clips.push(Arc::new(clip.tidy()));
    }
    Ok(Outcome {
        created_id: created,
        applied: true,
    })
}

fn kind_order(kind: TrackKind) -> u8 {
    match kind {
        TrackKind::Audio => 0,
        TrackKind::Video => 1,
        TrackKind::Text => 2,
        TrackKind::Captions => 3,
        TrackKind::Sticker => 4,
        TrackKind::Filter => 5,
    }
}
