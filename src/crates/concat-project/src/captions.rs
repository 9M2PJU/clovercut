// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Clovercut contributors

//! Caption membership, independent of ordinary titles and timeline placement.

use crate::model::{Clip, ClipKind, Timeline};

/// The default caption collection within a timeline.
pub const DEFAULT_GROUP: &str = "captions";

/// Whether a clip belongs to a caption collection.
pub fn is_caption(clip: &Clip) -> bool {
    clip.kind == ClipKind::Text && group(clip).is_some()
}

fn group(clip: &Clip) -> Option<&str> {
    clip.text
        .as_ref()?
        .caption_group
        .as_deref()
        .filter(|id| !id.is_empty())
}

/// Apply a caption inspector edit to the collection or just the selected clips.
pub fn scoped_targets(timeline: &Timeline, selected: &[String], all: bool) -> Vec<String> {
    if all {
        style_targets(timeline, selected)
    } else {
        selected.to_vec()
    }
}

/// Expand styling targets to their caption collections, preserving selection
/// order. Text content, timing, and ordinary titles are never grouped by this.
pub fn style_targets(timeline: &Timeline, selected: &[String]) -> Vec<String> {
    let groups: Vec<&str> = selected
        .iter()
        .filter_map(|id| timeline.clip(id).and_then(group))
        .collect();
    let mut ids = selected.to_vec();
    for clip in &timeline.clips {
        if group(clip).is_some_and(|id| groups.contains(&id)) && !ids.contains(&clip.id) {
            ids.push(clip.id.clone());
        }
    }
    ids
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Command, DocumentSettings, Editor,
        commands::ClipPatch,
        model::{Project, TextStyle},
        to_document,
    };

    #[test]
    fn caption_scope_and_animated_paint_survive_save_and_undo() {
        use crate::model::{KeyEase, KeyProperty};
        let mut project = Project::new();
        for id in ["a", "b"] {
            let mut clip = Clip::blank(id, "T1", ClipKind::Text, id, 0.0, 2.0);
            clip.text = Some(TextStyle {
                content: id.into(),
                caption_group: Some(DEFAULT_GROUP.into()),
                underline: true,
                vertical: true,
                vertical_align: 2,
                color: "#ff0000".into(),
                ..Default::default()
            });
            project.active_mut().clips.push(clip.into());
        }
        let selected = vec!["a".to_owned()];
        assert_eq!(scoped_targets(project.active(), &selected, false), selected);
        assert_eq!(
            scoped_targets(project.active(), &selected, true),
            vec!["a", "b"]
        );
        let settings = DocumentSettings {
            name: "Caption keys".into(),
            width: 640,
            height: 360,
            rate_num: 30,
            rate_den: 1,
        };
        let mut editor = Editor::from_document(&to_document(&settings, &project)).unwrap();
        let mut commands = Vec::new();
        for id in scoped_targets(editor.project().active(), &selected, true) {
            for (property, first, last) in [
                (KeyProperty::TextRed, 1.0, 0.0),
                (KeyProperty::TextBlue, 0.0, 1.0),
                (KeyProperty::StrokeWidth, 0.0, 0.01),
                (KeyProperty::StrokeOpacity, 0.0, 1.0),
            ] {
                for (at, value) in [(0.0, first), (1.0, last)] {
                    commands.push(Command::SetClipKey {
                        clip_id: id.clone(),
                        property,
                        at,
                        value,
                        ease: KeyEase::LINEAR,
                    });
                }
            }
        }
        editor.apply(Command::Batch { commands }).unwrap();
        let clip = editor.project().active().clip("a").unwrap();
        let middle = clip.text_at(0.5);
        assert_eq!(middle.color, "#800080");
        assert!((middle.stroke_width - 0.005).abs() < 1e-6);
        assert!(middle.stroke_color.ends_with("80"));
        assert!(middle.underline);
        assert!(middle.vertical);
        assert_eq!(middle.vertical_align, 2);
        assert_eq!(middle.content, "a");
        let reopened = Editor::from_document(&to_document(&settings, editor.project())).unwrap();
        assert_eq!(
            reopened
                .project()
                .active()
                .clip("b")
                .unwrap()
                .text_at(0.5)
                .color,
            "#800080"
        );
        assert!(editor.undo());
        assert!(editor.project().active().clip("a").unwrap().keys.is_empty());
        assert!(editor.project().active().clip("b").unwrap().keys.is_empty());
    }

    #[test]
    fn caption_styles_round_trip_and_undo_together_without_touching_titles_or_words() {
        let mut project = Project::new();
        for (id, group) in [
            ("a", Some("captions")),
            ("b", Some("captions")),
            ("title", None),
            ("translation", Some("translation")),
        ] {
            let mut clip = Clip::blank(id, "T1", ClipKind::Text, id, 0.0, 2.0);
            clip.text = Some(TextStyle {
                content: id.into(),
                caption_group: group.map(str::to_owned),
                ..Default::default()
            });
            project.active_mut().clips.push(clip.into());
        }
        let settings = DocumentSettings {
            name: "captions".into(),
            width: 1920,
            height: 1080,
            rate_num: 25,
            rate_den: 1,
        };
        let mut editor = Editor::from_document(&to_document(&settings, &project)).unwrap();
        let ids = style_targets(editor.project().active(), &["a".into()]);
        assert_eq!(ids, ["a", "b"]);
        let commands = ids
            .iter()
            .map(|id| {
                let mut style = editor
                    .project()
                    .active()
                    .clip(id)
                    .unwrap()
                    .text
                    .clone()
                    .unwrap();
                style.font_size = 0.08;
                Command::UpdateClip {
                    clip_id: id.clone(),
                    patch: ClipPatch {
                        text: Some(Some(style)),
                        ..Default::default()
                    },
                }
            })
            .collect();
        editor.apply(Command::Batch { commands }).unwrap();
        for id in ["a", "b"] {
            let clip = editor.project().active().clip(id).unwrap();
            assert_eq!(clip.text.as_ref().unwrap().font_size, 0.08);
            assert_eq!(clip.text.as_ref().unwrap().content, id);
            assert_eq!((clip.start, clip.duration), (0.0, 2.0));
        }
        for id in ["title", "translation"] {
            assert_eq!(
                editor
                    .project()
                    .active()
                    .clip(id)
                    .unwrap()
                    .text
                    .as_ref()
                    .unwrap()
                    .font_size,
                TextStyle::default().font_size
            );
        }
        let reopened = Editor::from_document(&to_document(&settings, editor.project())).unwrap();
        assert_eq!(
            style_targets(reopened.project().active(), &["a".into()]),
            ids
        );
        assert!(editor.undo());
        assert!(!editor.can_undo());
        assert_eq!(
            editor
                .project()
                .active()
                .clip("b")
                .unwrap()
                .text
                .as_ref()
                .unwrap()
                .font_size,
            TextStyle::default().font_size
        );
        assert!(editor.redo());
    }

    #[test]
    fn legacy_titles_are_not_implicitly_captions() {
        let style: crate::model::TextStyle =
            serde_json::from_str(r#"{"content":"Title"}"#).unwrap();
        assert!(style.caption_group.is_none());
        assert!(!style.vertical, "existing text stays horizontal");
        assert!(
            !serde_json::to_value(style)
                .unwrap()
                .as_object()
                .unwrap()
                .contains_key("captionGroup")
        );
    }
}
