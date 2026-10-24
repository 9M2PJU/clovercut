// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Jareer and Concat contributors
//! Exercise the same copy commands used by the desktop without compiling Slint.

#[path = "../../concat/src/clip_copy.rs"]
mod clip_copy;

use concat_project::Editor;
use concat_project::commands::Command;
use concat_project::model::{AppliedFilter, ClipKind, KeyEase, Track, TrackKind};

#[test]
fn a_media_less_layer_copies_its_chain_keys_and_strength_and_undoes_as_a_paste() {
    let mut editor = Editor::new();
    let source_id = editor
        .apply(Command::AddLayerClip {
            track_id: None,
            start: 0.0,
            duration: Some(4.0),
            effect_id: "concat.glow".into(),
            name: "Look".into(),
        })
        .unwrap()
        .created_id
        .unwrap();
    let mut source = editor.project().active().clip(&source_id).unwrap().clone();
    let mut link = AppliedFilter::new("concat.glow");
    link.enabled = false;
    link.params.insert("amount".into(), 0.4);
    link.set_key("amount", 0.5, 0.7, KeyEase::LINEAR);
    source.video_effects = vec![link, AppliedFilter::new("concat.blur")];
    source.opacity = 0.35;
    source.fade_in = 0.3;
    source.fade_out = 0.5;
    let lane = source.track_id.clone();
    for chain in [source.video_effects.clone(), Vec::new()] {
        source.video_effects = chain;
        let add =
            clip_copy::layer_copy_command(&source, editor.project().active(), &lane, 5.0).unwrap();
        let id = editor
            .apply_within(Some("copy-layer"), add)
            .unwrap()
            .created_id
            .unwrap();
        let placed = editor.project().active().clip(&id).unwrap().clone();
        let edits = clip_copy::copy_commands(&source, &placed, &lane, 5.0);
        editor
            .apply_within(Some("copy-layer"), Command::Batch { commands: edits })
            .unwrap();
        editor.end_gesture();
        let copy = editor.project().active().clip(&id).unwrap();
        assert_eq!(copy.kind, ClipKind::Layer);
        assert!(copy.media_id.is_empty());
        assert_eq!(copy.video_effects, source.video_effects);
        assert_eq!((copy.start, copy.duration, copy.opacity), (5.0, 4.0, 0.35));
        assert_eq!((copy.fade_in, copy.fade_out), (0.3, 0.5));
        assert!(editor.undo());
        assert!(editor.project().active().clip(&id).is_none());
        assert!(editor.redo());
        assert_eq!(
            editor.project().active().clip(&id).unwrap().video_effects,
            source.video_effects
        );
        assert!(editor.undo());
    }
}

#[test]
fn visual_layers_cannot_be_copied_into_audio_or_below_the_main_track() {
    let mut editor = Editor::new();
    let id = editor
        .apply(Command::AddLayerClip {
            track_id: None,
            start: 0.0,
            duration: None,
            effect_id: "concat.glow".into(),
            name: "Look".into(),
        })
        .unwrap()
        .created_id
        .unwrap();
    let source = editor.project().active().clip(&id).unwrap().clone();
    let mut timeline = editor.project().active().clone();
    timeline.tracks.insert(
        0,
        Track {
            id: "audio".into(),
            kind: TrackKind::Audio,
            ..Default::default()
        },
    );
    assert!(clip_copy::layer_copy_command(&source, &timeline, "audio", 6.0).is_none());
    timeline.tracks[0].kind = TrackKind::Video;
    assert!(clip_copy::layer_copy_command(&source, &timeline, "audio", 6.0).is_none());
    assert!(clip_copy::layer_copy_command(&source, &timeline, "missing", 6.0).is_none());
}
