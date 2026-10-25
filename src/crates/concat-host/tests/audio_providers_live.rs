// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Clovercut contributors
//! Opt-in real-provider audio search, download and cache verification.

use concat_host::{AppDirs, sounds};
use std::sync::atomic::AtomicBool;

#[test]
#[ignore = "downloads real provider audio; requires network, connected YouTube Audio API and CLOVERCUT_FREESOUND_KEY"]
fn search_download_probe_and_reuse_both_audio_providers() {
    let key = std::env::var("CLOVERCUT_FREESOUND_KEY").expect("provide a Freesound API key");
    let root = std::env::temp_dir().join(format!("clovercut-live-audio-{}", std::process::id()));
    let dirs = AppDirs::under(&root);
    for (provider, query) in [(1, "Mess Call"), (2, "rain")] {
        let catalogue = sounds::search_catalogue_provider(&dirs, provider, query, 0, 0, &key, "")
            .expect("provider search")
            .expect("catalogue");
        let sound = catalogue.sounds.first().expect("a playable search result");
        assert!(sound.is_offered());
        let path = sounds::fetch_sound(&dirs, sound, &AtomicBool::new(false), &mut |_, _| {})
            .expect("download playable audio");
        let summary = concat_host::media::probe(path.to_str().unwrap()).expect("probe audio");
        assert!(summary.audio.is_some());
        assert!(summary.duration.is_some_and(|d| d > 0.0));
        let known = sounds::resolved_sound(&dirs, sound).expect("cached metadata");
        assert_eq!(known.sha256.len(), 64);
        assert!(known.duration > 0.0);
        let reused = sounds::fetch_sound(&dirs, sound, &AtomicBool::new(false), &mut |_, _| {
            panic!("cached audio should not download again")
        })
        .expect("reuse cached audio");
        assert_eq!(path, reused);
    }
    std::fs::remove_dir_all(root).unwrap();
}
