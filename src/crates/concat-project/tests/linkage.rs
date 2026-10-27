// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Clovercut contributors
//! Linked time edits preserve typed placement, independent captions and locks.

use concat_project::{
    Command, DocumentSettings, Editor,
    commands::{PlayheadSide, TrimEdge, delete_to_playhead},
    model::*,
    to_document,
};
use std::sync::Arc;

fn fixture() -> Editor {
    let mut project = Project::new();
    project.media.push(serde_json::from_value(serde_json::json!({
        "id": "media", "path": "/linkage-test.mp4", "name": "Media", "kind": "video", "duration": 60.0, "hasAudio": true
    })).unwrap());
    let timeline = project.active_mut();
    timeline.linkage = true;
    timeline.tracks = vec![
        Track {
            id: "audio".into(),
            kind: TrackKind::Audio,
            ..Default::default()
        },
        Track {
            id: "main".into(),
            kind: TrackKind::Video,
            is_main: true,
            ..Default::default()
        },
        Track {
            id: "overlay".into(),
            kind: TrackKind::Video,
            ..Default::default()
        },
        Track {
            id: "captions".into(),
            kind: TrackKind::Captions,
            ..Default::default()
        },
        Track {
            id: "titles".into(),
            kind: TrackKind::Text,
            ..Default::default()
        },
        Track {
            id: "stickers".into(),
            kind: TrackKind::Sticker,
            ..Default::default()
        },
        Track {
            id: "filters".into(),
            kind: TrackKind::Filter,
            ..Default::default()
        },
    ];
    for (id, lane, kind, start, duration) in [
        ("cut", "main", ClipKind::Video, 2., 2.),
        ("tail", "main", ClipKind::Video, 5., 2.),
        ("caption-cut", "captions", ClipKind::Text, 2., 2.),
        ("caption-tail", "captions", ClipKind::Text, 5., 1.),
        ("overlay-tail", "overlay", ClipKind::Video, 5., 1.),
        ("audio-tail", "audio", ClipKind::Audio, 5., 1.),
        ("title-tail", "titles", ClipKind::Text, 5., 1.),
        ("sticker-tail", "stickers", ClipKind::Shape, 5., 1.),
        ("filter-tail", "filters", ClipKind::Layer, 5., 1.),
    ] {
        let mut clip = Clip::blank(id, lane, kind, id, start, duration);
        if matches!(kind, ClipKind::Video | ClipKind::Audio) {
            clip.media_id = "media".into();
        }
        if kind == ClipKind::Text {
            clip.text = Some(TextStyle {
                content: id.into(),
                caption_group: (lane == "captions").then(|| "captions-group".into()),
                ..Default::default()
            });
        }
        timeline.clips.push(Arc::new(clip));
    }
    from(project)
}
fn settings() -> DocumentSettings {
    DocumentSettings {
        canvas: [0, 0, 0, 255],
        name: "Linkage".into(),
        width: 640,
        height: 360,
        rate_num: 30,
        rate_den: 1,
    }
}
fn from(mut project: Project) -> Editor {
    for clip in &mut project.active_mut().clips {
        if matches!(clip.kind, ClipKind::Video | ClipKind::Audio) && clip.media_id.is_empty() {
            Arc::make_mut(clip).media_id = "media".into();
        }
    }
    Editor::from_document(&to_document(&settings(), &project)).unwrap()
}
fn at(editor: &Editor, id: &str) -> f64 {
    editor.project().active().clip(id).unwrap().start
}
fn remove(editor: &mut Editor, ids: &[&str]) {
    editor
        .apply(Command::RemoveClips {
            clip_ids: ids.iter().map(|id| id.to_string()).collect(),
            ripple: true,
        })
        .unwrap();
}

#[test]
fn selected_main_and_caption_intervals_are_counted_once_with_history() {
    for ids in [
        vec!["cut"],
        vec!["cut", "caption-cut"],
        vec!["caption-cut", "cut"],
    ] {
        let mut editor = fixture();
        let before = to_document(&settings(), editor.project());
        remove(&mut editor, &ids);
        for id in [
            "tail",
            "caption-tail",
            "overlay-tail",
            "audio-tail",
            "title-tail",
            "sticker-tail",
            "filter-tail",
        ] {
            assert_eq!(at(&editor, id), 3., "{id} follows exactly two seconds");
        }
        assert!(editor.project().active().clip("caption-cut").is_none());
        let after = to_document(&settings(), editor.project());
        assert!(editor.undo());
        assert_eq!(to_document(&settings(), editor.project()), before);
        assert!(editor.redo());
        assert_eq!(to_document(&settings(), editor.project()), after);
        let reopened = Editor::from_document(&after).unwrap();
        assert_eq!(at(&reopened, "caption-tail"), 3.);
        assert!(reopened.project().active().linkage);
    }
}

#[test]
fn independent_lane_deletion_and_disabled_linkage_keep_existing_behavior() {
    let mut editor = fixture();
    remove(&mut editor, &["caption-cut"]);
    assert_eq!(at(&editor, "caption-tail"), 3.);
    assert_eq!(at(&editor, "tail"), 5.);
    assert_eq!(at(&editor, "audio-tail"), 5.);
    let mut editor = fixture();
    let timeline_id = editor.project().active().id.clone();
    editor
        .apply(Command::SetTimelineLinkage {
            timeline_id,
            on: false,
        })
        .unwrap();
    remove(&mut editor, &["cut"]);
    assert_eq!(at(&editor, "tail"), 3.);
    assert_eq!(at(&editor, "caption-tail"), 5.);
    assert!(editor.project().active().clip("caption-cut").is_some());
}

#[test]
fn locked_followers_are_neither_moved_nor_deleted_and_locked_main_is_unchanged() {
    let mut editor = fixture();
    let locked = vec!["captions".into(), "audio".into()];
    editor
        .apply_within_locked(
            None,
            Command::RemoveClips {
                clip_ids: vec!["cut".into(), "caption-cut".into()],
                ripple: true,
            },
            &locked,
        )
        .unwrap();
    assert_eq!(at(&editor, "caption-cut"), 2.);
    assert_eq!(at(&editor, "caption-tail"), 5.);
    assert_eq!(at(&editor, "audio-tail"), 5.);
    assert_eq!(at(&editor, "overlay-tail"), 3.);
    let before = to_document(&settings(), editor.project());
    assert!(
        !editor
            .apply_within_locked(
                None,
                Command::TrimClip {
                    clip_id: "tail".into(),
                    edge: TrimEdge::End,
                    delta: -0.5,
                    ripple: true
                },
                &["main".into()]
            )
            .unwrap()
            .applied
    );
    assert_eq!(to_document(&settings(), editor.project()), before);
}

#[test]
fn crossing_music_bed_stays_and_following_audio_does_not_overlap_it() {
    let mut project = fixture().project().clone();
    let timeline = project.active_mut();
    timeline.clips.retain(|clip| clip.id != "audio-tail");
    timeline.clips.push(Arc::new(Clip::blank(
        "bed",
        "audio",
        ClipKind::Audio,
        "Bed",
        0.,
        6.,
    )));
    timeline.clips.push(Arc::new(Clip::blank(
        "after-bed",
        "audio",
        ClipKind::Audio,
        "Next",
        7.,
        2.,
    )));
    let mut editor = from(project);
    remove(&mut editor, &["cut"]);
    assert_eq!(at(&editor, "bed"), 0.);
    assert_eq!(at(&editor, "after-bed"), 6.);
    assert_eq!(editor.project().active().clip("bed").unwrap().duration, 6.);
}

#[test]
fn trim_and_speed_follow_the_fixed_main_not_the_lowest_occupied_video_lane() {
    for command in [
        Command::TrimClip {
            clip_id: "cut".into(),
            edge: TrimEdge::End,
            delta: -1.,
            ripple: true,
        },
        Command::TrimClip {
            clip_id: "cut".into(),
            edge: TrimEdge::Start,
            delta: 1.,
            ripple: true,
        },
        Command::SetClipSpeed {
            clip_id: "cut".into(),
            speed: 2.,
            ripple: true,
        },
    ] {
        let mut editor = fixture();
        editor.apply(command).unwrap();
        assert_eq!(at(&editor, "tail"), 4.);
        assert_eq!(at(&editor, "caption-tail"), 4.);
        assert_eq!(at(&editor, "audio-tail"), 4.);
    }
    let mut project = fixture().project().clone();
    project
        .active_mut()
        .clips
        .retain(|clip| clip.track_id != "main");
    let mut editor = from(project);
    remove(&mut editor, &["overlay-tail"]);
    assert_eq!(at(&editor, "caption-tail"), 5.);
}

#[test]
fn multi_lane_delete_to_playhead_trims_once_in_original_coordinates() {
    for side in [PlayheadSide::Left, PlayheadSide::Right] {
        for ids in [
            vec!["cut".into(), "caption-cut".into()],
            vec!["caption-cut".into(), "cut".into()],
        ] {
            let mut editor = fixture();
            let command =
                delete_to_playhead(editor.project().active(), &ids, 3., side, true).unwrap();
            editor.apply(command).unwrap();
            assert_eq!(at(&editor, "tail"), 4.);
            assert_eq!(at(&editor, "caption-tail"), 4.);
            assert_eq!(at(&editor, "audio-tail"), 4.);
            assert_eq!(editor.project().active().clip("cut").unwrap().duration, 1.);
            assert_eq!(
                editor
                    .project()
                    .active()
                    .clip("caption-cut")
                    .unwrap()
                    .duration,
                1.
            );
        }
    }
}

#[test]
fn new_and_legacy_documents_are_opt_in_and_toggling_is_undoable() {
    let mut editor = Editor::new();
    assert!(!editor.project().active().linkage);
    let before = to_document(&settings(), editor.project());
    let timeline_id = editor.project().active().id.clone();
    editor
        .apply(Command::SetTimelineLinkage {
            timeline_id,
            on: true,
        })
        .unwrap();
    assert!(editor.project().active().linkage);
    assert!(editor.undo());
    assert_eq!(to_document(&settings(), editor.project()), before);
    assert!(editor.redo());
    let reopened = Editor::from_document(&to_document(&settings(), editor.project())).unwrap();
    assert!(reopened.project().active().linkage);
    editor.apply(Command::AddTimeline).unwrap();
    assert!(
        editor.project().active().linkage,
        "new cut inherits its predecessor's choice"
    );
}

#[test]
fn magnetic_insertion_and_freeze_move_followers_but_plain_placement_does_not() {
    for ripple in [false, true] {
        let mut editor = fixture();
        editor
            .apply(Command::AddClip {
                media_id: "media".into(),
                track_id: "main".into(),
                start: 2.,
                ripple,
            })
            .unwrap();
        assert_eq!(at(&editor, "caption-tail"), if ripple { 65. } else { 5. });
    }
    let mut project = fixture().project().clone();
    project.media[0].kind = MediaKind::Image;
    project.active_mut().clip_mut("cut").unwrap().kind = ClipKind::Image;
    let mut editor = from(project);
    editor
        .apply(Command::FreezeFrame {
            clip_id: "cut".into(),
            time: 3.,
            duration: Some(1.),
            still: None,
        })
        .unwrap();
    assert_eq!(at(&editor, "caption-tail"), 6.);
    assert_eq!(at(&editor, "tail"), 6.);
}

#[test]
fn overlapping_and_adjacent_removed_spans_form_one_union_across_tracks() {
    let mut project = fixture().project().clone();
    let timeline = project.active_mut();
    timeline.clips.push(Arc::new(Clip::blank(
        "cut-2",
        "main",
        ClipKind::Video,
        "Second cut",
        4.,
        1.,
    )));
    timeline.clips.push(Arc::new(Clip::blank(
        "caption-extra",
        "captions",
        ClipKind::Text,
        "Extra",
        1.,
        3.,
    )));
    let mut editor = from(project);
    remove(
        &mut editor,
        &["cut", "cut-2", "caption-cut", "caption-extra"],
    );
    assert_eq!(at(&editor, "tail"), 2.);
    assert_eq!(
        at(&editor, "caption-tail"),
        1.,
        "union [1,5], not its summed lengths"
    );
    assert_eq!(at(&editor, "audio-tail"), 2.);
}

#[test]
fn linked_multi_lane_growth_is_one_shared_insertion() {
    for ids in [["cut", "caption-cut"], ["caption-cut", "cut"]] {
        let mut editor = fixture();
        editor
            .apply(Command::Batch {
                commands: ids
                    .iter()
                    .map(|id| Command::TrimClip {
                        clip_id: id.to_string(),
                        edge: TrimEdge::End,
                        delta: 1.,
                        ripple: true,
                    })
                    .collect(),
            })
            .unwrap();
        assert_eq!(at(&editor, "tail"), 6.);
        assert_eq!(at(&editor, "caption-tail"), 6.);
        assert_eq!(at(&editor, "audio-tail"), 6.);
    }
}

#[test]
fn speed_curve_ripple_and_locked_followers_share_constant_speed_policy() {
    let mut editor = fixture();
    editor
        .apply_within_locked(
            None,
            Command::SetClipSpeedCurve {
                clip_id: "cut".into(),
                curve: Some(vec![
                    SpeedPoint {
                        at: 0.,
                        speed: 2.,
                        slope: None,
                    },
                    SpeedPoint {
                        at: 1.,
                        speed: 2.,
                        slope: None,
                    },
                ]),
                ripple: true,
            },
            &["audio".into()],
        )
        .unwrap();
    assert_eq!(at(&editor, "tail"), 4.);
    assert_eq!(at(&editor, "caption-tail"), 4.);
    assert_eq!(at(&editor, "audio-tail"), 5.);
}

#[test]
fn separate_remove_commands_in_one_batch_share_the_same_removed_time() {
    for ids in [["cut", "caption-cut"], ["caption-cut", "cut"]] {
        let mut editor = fixture();
        editor
            .apply(Command::Batch {
                commands: ids
                    .iter()
                    .map(|id| Command::RemoveClips {
                        clip_ids: vec![id.to_string()],
                        ripple: true,
                    })
                    .collect(),
            })
            .unwrap();
        assert_eq!(at(&editor, "tail"), 3.);
        assert_eq!(at(&editor, "caption-tail"), 3.);
    }
}

#[test]
fn multi_lane_speed_changes_do_not_repeat_the_main_ripple() {
    let mut project = fixture().project().clone();
    let mut clip = Clip::blank("audio-cut", "audio", ClipKind::Audio, "Audio", 2., 2.);
    clip.media_id = "media".into();
    project.active_mut().clips.push(Arc::new(clip));
    let mut editor = from(project);
    editor
        .apply(Command::Batch {
            commands: ["audio-cut", "cut"]
                .iter()
                .map(|id| Command::SetClipSpeed {
                    clip_id: id.to_string(),
                    speed: 2.,
                    ripple: true,
                })
                .collect(),
        })
        .unwrap();
    assert_eq!(at(&editor, "audio-tail"), 4.);
    assert_eq!(at(&editor, "tail"), 4.);
    assert_eq!(at(&editor, "caption-tail"), 4.);
    assert_eq!(
        editor
            .project()
            .active()
            .clip("audio-cut")
            .unwrap()
            .duration,
        1.
    );
}

#[test]
fn preview_and_committed_trim_share_positions_durations_keys_and_locks() {
    for edge in [TrimEdge::Start, TrimEdge::End] {
        let mut editor = fixture();
        let mut preview = editor.project().clone();
        let command = Command::TrimClip {
            clip_id: "cut".into(),
            edge,
            delta: if edge == TrimEdge::Start { 0.5 } else { -0.5 },
            ripple: true,
        };
        let locked = vec!["audio".into()];
        concat_project::commands::apply_with_locks(
            &mut preview,
            &mut Default::default(),
            command.clone(),
            &locked,
        )
        .unwrap();
        editor.apply_within_locked(None, command, &locked).unwrap();
        assert_eq!(
            to_document(&settings(), &preview),
            to_document(&settings(), editor.project())
        );
    }
}
