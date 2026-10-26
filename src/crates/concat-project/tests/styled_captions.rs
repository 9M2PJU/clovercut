// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Clovercut contributors

//! Word styles remain local to one caption through history and persistence.

use concat_project::{
    Command, DocumentSettings, Editor,
    model::{RunStyle, TextStyle},
    to_document,
};

#[test]
fn word_styles_stay_local_to_a_caption_and_survive_history_and_reopen() {
    let mut editor = Editor::new();
    let mut ids = Vec::new();
    for (start, content) in [(0.0, "Olá mundo"), (3.0, "Outra legenda")] {
        ids.push(
            editor
                .apply(Command::AddTextClip {
                    above: false,
                    track_id: None,
                    start,
                    style: Some(TextStyle {
                        content: content.into(),
                        caption_group: Some("shared-caption-collection".into()),
                        vertical: true,
                        ..Default::default()
                    }),
                    duration: Some(2.0),
                    offset_y: None,
                })
                .unwrap()
                .created_id
                .unwrap(),
        );
    }
    let before = editor.project().active().clip(&ids[1]).unwrap().clone();
    editor
        .apply(Command::StyleTextRange {
            clip_id: ids[0].clone(),
            start: 4,
            end: 9,
            patch: RunStyle {
                color: Some("#00ff00".into()),
                font_scale: Some(1.5),
                ..Default::default()
            },
        })
        .unwrap();
    assert_eq!(editor.project().active().clip(&ids[1]).unwrap(), &before);
    assert!(editor.undo());
    assert!(
        editor
            .project()
            .active()
            .clip(&ids[0])
            .unwrap()
            .text
            .as_ref()
            .unwrap()
            .runs
            .is_empty()
    );
    assert!(editor.redo());
    let styled = editor.project().active().clip(&ids[0]).unwrap().clone();
    let settings = DocumentSettings {
        canvas: [0, 0, 0, 255],
        name: "Captions".into(),
        width: 640,
        height: 360,
        rate_num: 30,
        rate_den: 1,
    };
    let reopened = Editor::from_document(&to_document(&settings, editor.project())).unwrap();
    assert_eq!(reopened.project().active().clip(&ids[0]).unwrap(), &styled);
    assert_eq!(reopened.project().active().clip(&ids[1]).unwrap(), &before);
}
