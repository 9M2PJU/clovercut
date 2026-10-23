// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Jareer and Concat contributors

//! Lanes: adding, removing and muting or hiding them.
//!
//! One arm per command, exactly as [`super::apply`] routes them here;
//! everything these arms share lives in the parent module.

use super::*;

/// Applies one of this module's commands. Any other is a routing error.
pub(super) fn apply(
    project: &mut Project,
    mint: &mut IdMint,
    command: Command,
) -> Result<Outcome, CommandError> {
    match command {
        Command::AddTrack {
            bottom,
            kind,
            insert_at,
        } => {
            let timeline = project.active_mut();
            let id = mint.next("t");
            let track = Track {
                id: id.clone(),
                visible: true,
                muted: false,
                kind: kind.unwrap_or(if bottom {
                    TrackKind::Audio
                } else {
                    TrackKind::Video
                }),
                is_main: false,
                extra: Default::default(),
            };
            // The stack is stored bottom first: the last track is the lane
            // drawn on top. Audio tracks stay strictly below the main video track;
            // visual tracks (video overlays, adjustment layers, text, stickers) stay strictly above.
            let main_idx = timeline.main_track_index();
            let is_audio = track.kind == TrackKind::Audio;
            if let Some(idx) = insert_at {
                let clamp_idx = if is_audio {
                    idx.min(main_idx)
                } else {
                    idx.max((main_idx + 1).min(timeline.tracks.len()))
                        .min(timeline.tracks.len())
                };
                timeline.tracks.insert(clamp_idx, track);
            } else if bottom || is_audio {
                let insert_idx = if is_audio {
                    0
                } else {
                    (main_idx + 1).min(timeline.tracks.len())
                };
                timeline.tracks.insert(insert_idx, track);
            } else {
                timeline.tracks.push(track);
            }
            Ok(Outcome {
                created_id: Some(id),
                applied: true,
            })
        }

        Command::RemoveTrack { track_id } => {
            let timeline = project.active_mut();
            if timeline.tracks.len() <= 1 {
                return Err(CommandError::LastTrack);
            }
            if timeline.track(&track_id).is_some_and(|t| t.is_main) {
                return Err(CommandError::LastTrack);
            }
            let track_count = timeline.tracks.len();
            timeline.tracks.retain(|track| track.id != track_id);
            timeline.clips.retain(|clip| clip.track_id != track_id);
            // Clips only ever sit on existing tracks, so an unknown id - the
            // tolerated no-op - removes neither.
            let applied = timeline.tracks.len() != track_count;
            Ok(Outcome {
                created_id: None,
                applied,
            })
        }

        Command::SetTrackFlag {
            track_id,
            flag,
            value,
        } => {
            let timeline = project.active_mut();
            let applied = timeline
                .tracks
                .iter_mut()
                .find(|track| track.id == track_id)
                .is_some_and(|track| match flag {
                    TrackFlag::Visible => assign(&mut track.visible, value),
                    TrackFlag::Muted => assign(&mut track.muted, value),
                });
            Ok(Outcome {
                created_id: None,
                applied,
            })
        }

        _ => unreachable!("commands::apply routes only this module's commands here"),
    }
}
