// SPDX-License-Identifier: AGPL-3.0-or-later
//! Canvas persistence, compatibility and timeline ownership regressions.
use serde_json::json;
/// A timeline's canvas colour is written as `#rrggbb` (`#rrggbbaa` when
/// it is see-through) and read back;
/// black is not written at all, so a document without one is the
/// document it always was, and junk reads as black (#456).
#[test]
fn a_canvas_colour_is_written_only_when_it_is_not_black() {
    use concat_project::model::VideoSettings;
    let black = serde_json::to_value(VideoSettings::default()).unwrap();
    assert!(black.get("canvas").is_none(), "{black}");
    let green = VideoSettings {
        canvas: [0, 177, 64, 255],
        ..VideoSettings::default()
    };
    let written = serde_json::to_value(green).unwrap();
    assert_eq!(written["canvas"], "#00b140");
    let read: VideoSettings = serde_json::from_value(written).unwrap();
    assert_eq!(read, green);
    let junk: VideoSettings = serde_json::from_value(json!({
        "width": 1920, "height": 1080, "rateNum": 30, "rateDen": 1, "canvas": "teal"
    }))
    .unwrap();
    assert_eq!(junk.canvas, [0, 0, 0, 255]);
    // A see-through canvas carries its alpha, and an opaque one written
    // with an `ff` alpha reads as the same colour.
    let clear = VideoSettings {
        canvas: [0, 0, 0, 0],
        ..VideoSettings::default()
    };
    let written = serde_json::to_value(clear).unwrap();
    assert_eq!(written["canvas"], "#00000000");
    let read: VideoSettings = serde_json::from_value(written).unwrap();
    assert_eq!(read.canvas, [0, 0, 0, 0]);
    let opaque: VideoSettings = serde_json::from_value(json!({
        "width": 1920, "height": 1080, "rateNum": 30, "rateDen": 1, "canvas": "#00b140ff"
    }))
    .unwrap();
    assert_eq!(opaque.canvas, [0, 177, 64, 255]);
}

#[test]
fn each_timeline_keeps_its_canvas_through_history_and_document_round_trip() {
    use concat_project::model::{Background, OPAQUE_BLACK, VideoSettings};
    use concat_project::{Command, DocumentSettings, Editor};
    let mut editor = Editor::new();
    let first = editor.project().active_timeline_id.clone();
    let green = VideoSettings {
        canvas: [0, 177, 64, 255],
        ..VideoSettings::default()
    };
    editor
        .apply(Command::SetTimelineVideo {
            timeline_id: first.clone(),
            video: green,
        })
        .unwrap();
    assert!(editor.undo());
    assert_eq!(editor.project().active().video.canvas, OPAQUE_BLACK);
    assert!(editor.redo());
    let second = editor
        .apply(Command::AddTimeline)
        .unwrap()
        .created_id
        .unwrap();
    assert_eq!(editor.project().active().video.canvas, green.canvas);
    let clear = VideoSettings {
        canvas: [0, 0, 0, 0],
        ..green
    };
    editor
        .apply(Command::SetTimelineVideo {
            timeline_id: second.clone(),
            video: clear,
        })
        .unwrap();
    let settings = DocumentSettings {
        name: "Canvas regression".into(),
        width: 1920,
        height: 1080,
        rate_num: 30,
        rate_den: 1,
        canvas: clear.canvas,
    };
    let document = editor.to_document(&settings);
    assert_eq!(
        document["video"]["canvas"], "#00000000",
        "legacy active mirror follows its timeline"
    );
    let reopened = Editor::from_document(&document).unwrap();
    for (id, canvas) in [(&first, green.canvas), (&second, clear.canvas)] {
        let timeline = reopened
            .project()
            .timelines
            .iter()
            .find(|timeline| &timeline.id == id)
            .unwrap();
        assert_eq!(timeline.video.canvas, canvas);
        assert_eq!(
            timeline.background,
            Background::None,
            "the existing per-clip backdrop is independent"
        );
        assert!(
            timeline.tracks.iter().any(|track| track.is_main),
            "typed main track is retained"
        );
    }
}

#[test]
fn legacy_documents_and_invalid_colours_do_not_become_transparent_or_panic() {
    use concat_project::model::{OPAQUE_BLACK, VideoSettings};
    use concat_project::{DocumentSettings, Editor};
    for value in [
        json!(null),
        json!([1, 2]),
        json!("#é0000"),
        json!("#000000gg"),
        json!(42),
    ] {
        let read: VideoSettings = serde_json::from_value(
            json!({ "width": 1920, "height": 1080, "rateNum": 30, "rateDen": 1, "canvas": value }),
        )
        .unwrap();
        assert_eq!(read.canvas, OPAQUE_BLACK);
    }
    let settings = DocumentSettings {
        name: "Old".into(),
        width: 1920,
        height: 1080,
        rate_num: 30,
        rate_den: 1,
        canvas: OPAQUE_BLACK,
    };
    let document = Editor::new().to_document(&settings);
    assert!(document["video"].get("canvas").is_none());
    assert_eq!(
        Editor::from_document(&document)
            .unwrap()
            .project()
            .active()
            .video
            .canvas,
        OPAQUE_BLACK
    );
}
