// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Jareer and Concat contributors
//! Commands shared by clip copying and duplication, independent of the window.

use concat_project::commands::{ClipMove, ClipPatch, Command, TrimEdge};
use concat_project::model::{Clip, ClipKind, Timeline, TrackKind};

/// A visual layer has no media file: create another layer before copying its chain.
pub(crate) fn layer_copy_command(
    source: &Clip,
    timeline: &Timeline,
    track_id: &str,
    start: f64,
) -> Option<Command> {
    let row = timeline
        .tracks
        .iter()
        .position(|track| track.id == track_id)?;
    let track = &timeline.tracks[row];
    if source.kind != ClipKind::Layer
        || track.kind == TrackKind::Audio
        || row < timeline.main_track_index()
    {
        return None;
    }
    Some(Command::AddLayerClip {
        track_id: Some(track_id.to_owned()),
        start,
        duration: Some(source.duration),
        effect_id: source
            .video_effects
            .first()
            .map(|link| link.id.clone())
            .unwrap_or_default(),
        name: source.name.clone(),
    })
}

/// The edits that make `placed` - the whole of `source`'s file, just laid
/// on a free stretch of the lane - into a copy of `source` at `start` on
/// `track_id`: its frames, speed, placement, keys and mix.
///
/// The footage is trimmed at normal speed to what the source covers, then
/// retimed - the speed command keeps the footage covered and recomputes
/// the length - and only then moved to `start`, so neighbours there never
/// push the whole file aside and shift the frames the trims cut.
pub(crate) fn copy_commands(
    source: &Clip,
    placed: &Clip,
    track_id: &str,
    start: f64,
) -> Vec<Command> {
    let created = &placed.id;
    let mut commands = Vec::new();
    let head = source.source_start - placed.source_start;
    if head.abs() > 1e-6 {
        commands.push(Command::TrimClip {
            clip_id: created.clone(),
            edge: TrimEdge::Start,
            delta: head,
            ripple: false,
        });
    }
    // The footage the source covers, at normal speed.
    let covered = source.duration * source.speed;
    let tail = covered - (placed.duration - head);
    if tail.abs() > 1e-6 {
        commands.push(Command::TrimClip {
            clip_id: created.clone(),
            edge: TrimEdge::End,
            delta: tail,
            ripple: false,
        });
    }
    if (source.speed - 1.0).abs() > 1e-9 {
        commands.push(Command::SetClipSpeed {
            clip_id: created.clone(),
            ripple: false,
            speed: source.speed,
        });
    }
    if source.speed_curve.is_some() {
        commands.push(Command::SetClipSpeedCurve {
            clip_id: created.clone(),
            ripple: false,
            curve: source.speed_curve.clone(),
        });
    }
    commands.push(Command::MoveClips {
        moves: vec![ClipMove {
            clip_id: created.clone(),
            start,
            track_id: track_id.to_owned(),
        }],
    });
    commands.push(Command::SetClipTransform {
        clip_id: created.clone(),
        scale: Some(source.scale),
        offset_x: Some(source.offset_x),
        offset_y: Some(source.offset_y),
        rotation: Some(source.rotation),
        stretch_x: Some(source.stretch_x),
        stretch_y: Some(source.stretch_y),
    });
    for key in &source.keys {
        commands.push(Command::SetClipKey {
            clip_id: created.clone(),
            property: key.property,
            at: key.at,
            value: key.value,
            ease: key.ease,
        });
    }
    commands.push(Command::UpdateClip {
        clip_id: created.clone(),
        patch: ClipPatch {
            name: Some(format!("{} copy", source.name)),
            volume: Some(source.volume),
            fade_in: Some(source.fade_in),
            fade_out: Some(source.fade_out),
            visual_motion: Some(source.visual_motion),
            opacity: Some(source.opacity),
            preserve_pitch: Some(source.preserve_pitch),
            muted: source.muted,
            flip_h: Some(source.flip_h),
            flip_v: Some(source.flip_v),
            blend: Some(source.blend.clone()),
            crop: Some(source.crop),
            filters: Some(source.filters.clone()),
            video_effects: Some(source.video_effects.clone()),
            ..ClipPatch::default()
        },
    });
    commands
}
