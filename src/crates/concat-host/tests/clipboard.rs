// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Clovercut contributors

//! Durable clipboard files, cancellation, deduplication and cleanup regressions.

use concat_host::clipboard::prepare;
use concat_project::model::{Clip, ClipKind, CustomFont, MediaItem, TextStyle};
use concat_project::{Command, Editor, Project, clipboard::Clipboard};
use std::path::Path;
use std::sync::{Arc, atomic::AtomicBool};

fn clipboard(path: &Path) -> Clipboard {
    let item: MediaItem = serde_json::from_value(serde_json::json!({"id":"m1", "path":path, "name":"Generated", "kind":"video", "duration":3.0, "hasAudio":true})).unwrap();
    let mut p = Project::new();
    p.media.push(item);
    let lane = p
        .active()
        .tracks
        .iter()
        .find(|t| t.is_main)
        .unwrap_or(&p.active().tracks[0])
        .id
        .clone();
    let mut c = Clip::blank("c1", lane, ClipKind::Video, "Generated", 0.0, 2.0);
    c.media_id = "m1".into();
    p.active_mut().clips.push(Arc::new(c));
    Clipboard::capture(&p, &["c1".into()])
}

fn accept(editor: &mut Editor, prepared: &mut concat_host::clipboard::PreparedClipboard) {
    editor
        .apply(Command::PasteClips {
            clipboard: Box::new(std::mem::take(&mut prepared.clipboard)),
            start: 0.0,
            shift: 0,
            locked_tracks: vec![],
        })
        .unwrap();
}

#[test]
fn owned_files_survive_source_removal_undo_redo_and_repeated_paste_reuses_them() {
    let source = tempfile::tempdir().unwrap();
    let target = tempfile::tempdir().unwrap();
    let path = source.path().join("enhanced.mp4");
    std::fs::write(&path, b"owned media fixture").unwrap();
    let held = clipboard(&path);
    let cancel = AtomicBool::new(false);
    let mut editor = Editor::new();
    let mut prepared = prepare(
        held.clone(),
        source.path(),
        target.path(),
        editor.project(),
        &cancel,
    )
    .unwrap();
    let imported = prepared.clipboard.media[0].path.clone();
    assert!(Path::new(&imported).starts_with(target.path()));
    accept(&mut editor, &mut prepared);
    prepared.commit();
    let repeated = prepare(
        held,
        source.path(),
        target.path(),
        editor.project(),
        &cancel,
    )
    .unwrap();
    assert_eq!(repeated.clipboard.media[0].path, imported);
    assert_eq!(
        std::fs::read_dir(target.path().join("media/pasted"))
            .unwrap()
            .count(),
        1
    );
    drop(repeated);
    std::fs::remove_file(path).unwrap();
    assert_eq!(std::fs::read(&imported).unwrap(), b"owned media fixture");
    assert!(editor.undo());
    assert!(editor.project().media.is_empty());
    assert!(editor.redo());
    assert_eq!(editor.project().media[0].path, imported);
    assert!(Path::new(&imported).is_file());
}

#[test]
fn dropping_uncommitted_results_removes_only_their_owned_resources() {
    let source = tempfile::tempdir().unwrap();
    let target = tempfile::tempdir().unwrap();
    let path = source.path().join("reverse.mp4");
    std::fs::write(&path, b"reverse").unwrap();
    let unrelated = target.path().join("keep.txt");
    std::fs::write(&unrelated, b"keep").unwrap();
    let prepared = prepare(
        clipboard(&path),
        source.path(),
        target.path(),
        &Project::new(),
        &AtomicBool::new(false),
    )
    .unwrap();
    let staged = prepared.clipboard.media[0].path.clone();
    assert!(Path::new(&staged).is_file());
    drop(prepared);
    assert!(!Path::new(&staged).exists());
    assert_eq!(std::fs::read(&path).unwrap(), b"reverse");
    assert_eq!(std::fs::read(&unrelated).unwrap(), b"keep");
}

#[test]
fn missing_owned_resources_and_cancellation_leave_no_temporary_files() {
    let source = tempfile::tempdir().unwrap();
    let target = tempfile::tempdir().unwrap();
    let path = source.path().join("good.mp4");
    std::fs::write(&path, b"good").unwrap();
    let mut held = clipboard(&path);
    let mut missing = held.media[0].clone();
    missing.id = "missing".into();
    missing.path = source
        .path()
        .join("missing.mp4")
        .to_string_lossy()
        .into_owned();
    held.media.push(missing);
    assert!(
        prepare(
            held,
            source.path(),
            target.path(),
            &Project::new(),
            &AtomicBool::new(false)
        )
        .is_err()
    );
    assert_eq!(
        std::fs::read_dir(target.path().join("media/pasted"))
            .unwrap()
            .count(),
        0
    );
    assert!(
        prepare(
            clipboard(&path),
            source.path(),
            target.path(),
            &Project::new(),
            &AtomicBool::new(true)
        )
        .is_err()
    );
    assert_eq!(
        std::fs::read_dir(target.path().join("media/pasted"))
            .unwrap()
            .count(),
        0
    );
}

#[test]
fn external_files_stay_references_and_relative_source_paths_resolve_correctly() {
    let source = tempfile::tempdir().unwrap();
    let target = tempfile::tempdir().unwrap();
    let external = tempfile::tempdir().unwrap();
    let path = external.path().join("large.mp4");
    std::fs::write(&path, b"external").unwrap();
    let prepared = prepare(
        clipboard(&path),
        source.path(),
        target.path(),
        &Project::new(),
        &AtomicBool::new(false),
    )
    .unwrap();
    assert_eq!(Path::new(&prepared.clipboard.media[0].path), path);
    assert!(!target.path().join("media").exists());
    let local = source.path().join("relative.mp4");
    std::fs::write(local, b"relative").unwrap();
    let prepared = prepare(
        clipboard(Path::new("relative.mp4")),
        source.path(),
        target.path(),
        &Project::new(),
        &AtomicBool::new(false),
    )
    .unwrap();
    assert_eq!(
        std::fs::read(&prepared.clipboard.media[0].path).unwrap(),
        b"relative"
    );
}

#[test]
fn identical_fonts_are_reused_and_conflicting_fonts_do_not_leave_files() {
    let source = tempfile::tempdir().unwrap();
    let target = tempfile::tempdir().unwrap();
    let font = source.path().join("custom.ttf");
    let existing = target.path().join("existing.ttf");
    std::fs::write(&font, b"same font").unwrap();
    std::fs::write(&existing, b"same font").unwrap();
    let mut held = Clipboard::default();
    held.fonts.push(CustomFont {
        family: "Custom".into(),
        path: font.to_string_lossy().into_owned(),
    });
    let mut p = Project::new();
    let mut c = Clip::blank(
        "title",
        p.active().tracks[0].id.clone(),
        ClipKind::Text,
        "Title",
        0.0,
        2.0,
    );
    c.text = Some(TextStyle {
        font_family: "Custom".into(),
        ..Default::default()
    });
    held.clips = Clipboard::from_clips(&p, [(0, c)]).clips;
    p.fonts.push(CustomFont {
        family: "Custom".into(),
        path: existing.to_string_lossy().into_owned(),
    });
    let prepared = prepare(
        held.clone(),
        source.path(),
        target.path(),
        &p,
        &AtomicBool::new(false),
    )
    .unwrap();
    assert_eq!(prepared.clipboard.fonts, p.fonts);
    drop(prepared);
    std::fs::write(&font, b"different font").unwrap();
    let mut prepared = prepare(
        held,
        source.path(),
        target.path(),
        &p,
        &AtomicBool::new(false),
    )
    .unwrap();
    let staged = prepared.clipboard.fonts[0].path.clone();
    let mut editor = Editor::from_document(&concat_project::to_document(
        &concat_project::DocumentSettings {
            canvas: [0, 0, 0, 255],
            name: "Target".into(),
            width: 640,
            height: 360,
            rate_num: 30,
            rate_den: 1,
        },
        &p,
    ))
    .unwrap();
    assert!(
        editor
            .apply(Command::PasteClips {
                clipboard: Box::new(std::mem::take(&mut prepared.clipboard)),
                start: 0.0,
                shift: 0,
                locked_tracks: vec![]
            })
            .is_err()
    );
    drop(prepared);
    assert!(!Path::new(&staged).exists());
    assert_eq!(std::fs::read(&existing).unwrap(), b"same font");
}
