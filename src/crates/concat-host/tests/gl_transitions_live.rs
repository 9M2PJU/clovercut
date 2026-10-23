// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Clovercut contributors
//! Opt-in official GL catalogue, package cache and GPU rendering verification.

use concat_effects::{Catalogue, Package};
use concat_host::{
    cards::{self, Card, Painter},
    export::WgpuCompositor,
    gl_transitions,
};

#[test]
#[ignore = "uses the official online GL catalogue and shader downloads"]
fn downloads_validates_and_reuses_a_transition_package() {
    let root = tempfile::tempdir().unwrap();
    let entries = gl_transitions::catalogue(&root.path().join("cache")).unwrap();
    assert_eq!(entries.len(), 123);
    let entry = entries.iter().find(|e| e.name == "GridFlip").unwrap();
    let effects = root.path().join("effects");
    gl_transitions::install(entry, &effects).unwrap();
    let folder = effects.join(entry.id());
    let shader = folder.join("effect.wgsl");
    let before = std::fs::metadata(&shader).unwrap().modified().unwrap();
    gl_transitions::install(entry, &effects).unwrap();
    assert_eq!(
        std::fs::metadata(&shader).unwrap().modified().unwrap(),
        before
    );
    assert!(folder.join("LICENSE").is_file());
    let package = Package::from_folder(&folder).unwrap();
    assert!(!package.manifest.params.is_empty());
    assert_eq!(
        package
            .manifest
            .params
            .iter()
            .find(|p| p.key == "p_size_0")
            .unwrap()
            .default,
        4.0
    );
    assert_eq!(
        gl_transitions::catalogue(&root.path().join("cache"))
            .unwrap()
            .len(),
        entries.len()
    );
}

#[test]
#[ignore = "requires official fixture files and a hardware GPU"]
fn official_shaders_render_on_gpu() {
    let Some(gpu) = WgpuCompositor::new() else {
        panic!("GPU required");
    };
    assert!(!gpu.on_software_adapter());
    let entries: Vec<gl_transitions::Entry> = serde_json::from_str(
        &std::fs::read_to_string(std::env::var("CLOVERCUT_GL_INDEX").unwrap()).unwrap(),
    )
    .unwrap();
    let fixture_dir = std::env::var("CLOVERCUT_GL_FIXTURES").unwrap();
    let root = tempfile::tempdir().unwrap();
    let still = cards::still_in(root.path()).unwrap();
    let city = cards::city_still_in(root.path()).unwrap();
    let mut painter = Painter::with_city(gpu, &still, Some(&city), 64, 36).unwrap();
    let mut catalogue = Catalogue::new();
    for entry in entries.iter().filter(|e| e.supported()) {
        let source = std::fs::read_to_string(
            std::path::Path::new(&fixture_dir).join(format!("{}.wgsl", entry.name)),
        )
        .unwrap();
        let (manifest, shader) = gl_transitions::adapt(entry, &source).unwrap();
        catalogue
            .add(Package::from_sources(&manifest, None, Some(&shader)).unwrap())
            .unwrap();
    }
    let mut count = 0;
    for package in catalogue.packages() {
        let card = Card::of(&catalogue, package, root.path()).unwrap();
        let start = painter.transition_frame(&card, 0.0).unwrap();
        let middle = painter.transition_frame(&card, 0.5).unwrap();
        let end = painter.transition_frame(&card, 1.0).unwrap();
        assert_ne!(start.pixels(), end.pixels(), "{} endpoints", package.id());
        assert!(
            middle.pixels().chunks_exact(4).any(|p| p[3] > 0),
            "{} middle",
            package.id()
        );
        count += 1;
    }
    assert_eq!(count, 123);
}
