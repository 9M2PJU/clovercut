// SPDX-License-Identifier: AGPL-3.0-or-later

//! Persistence verification uses only unique disposable project directories.

use std::path::Path;

use concat_host::{Session, projects};
use concat_project::{Command, DocumentSettings, Editor};
use serde_json::json;

fn settings() -> DocumentSettings {
    DocumentSettings {
        name: "Disposable stability fixture".to_owned(),
        width: 1280,
        height: 720,
        rate_num: 25,
        rate_den: 1,
    }
}

fn open(root: &Path) -> Session {
    Session::open(root.to_str().unwrap(), settings()).expect("opens the disposable fixture")
}

#[test]
fn rejected_documents_keep_their_original_bytes_under_both_manifest_names() {
    let mut future = Editor::new().to_document(&settings());
    future["version"] = json!(concat_project::DOCUMENT_VERSION + 1);
    let fixtures = [
        b"{\"version\":1,\"timelines\":[".to_vec(),
        b"null".to_vec(),
        b"[]".to_vec(),
        br#"{"timelines":"garbage"}"#.to_vec(),
        serde_json::to_vec_pretty(&future).unwrap(),
    ];
    for name in ["concat.json", "wolfcut.json"] {
        for before in &fixtures {
            let scratch = tempfile::tempdir().unwrap();
            let manifest = scratch.path().join(name);
            std::fs::write(&manifest, before).unwrap();
            assert!(Session::open(scratch.path().to_str().unwrap(), settings()).is_err());
            assert_eq!(std::fs::read(&manifest).unwrap(), *before);
            assert_eq!(
                std::fs::read_dir(scratch.path()).unwrap().count(),
                1,
                "a rejected open must not replace the manifest or create a new one"
            );
        }
    }
    let scratch = tempfile::tempdir().unwrap();
    let manifest = scratch.path().join("concat.json");
    std::fs::create_dir(&manifest).unwrap();
    std::fs::write(manifest.join("kept"), b"recoverable bytes").unwrap();
    assert!(Session::open(scratch.path().to_str().unwrap(), settings()).is_err());
    assert_eq!(
        std::fs::read(manifest.join("kept")).unwrap(),
        b"recoverable bytes"
    );
}

#[test]
fn an_uncommitted_recovery_candidate_is_preserved_without_replacing_the_manifest() {
    let scratch = tempfile::tempdir().unwrap();
    let root = scratch.path();
    let mut original = open(root);
    original.save(None).unwrap();
    let manifest = projects::manifest_path(root);
    let before = std::fs::read(&manifest).unwrap();
    let original_tracks = original.project().active().tracks.len();
    original
        .apply(Command::AddTrack {
            bottom: false,
            kind: None,
            insert_at: None,
        })
        .unwrap();
    let candidate = serde_json::to_vec_pretty(&original.document()).unwrap();
    let orphan = root.join("concat.json.saving.interrupted-fixture");
    std::fs::write(&orphan, &candidate).unwrap();

    let mut reopened = open(root);
    assert_eq!(
        reopened.project().active().tracks.len(),
        original_tracks,
        "an uncommitted candidate is not automatically promoted"
    );
    assert_eq!(std::fs::read(&manifest).unwrap(), before);
    assert_eq!(std::fs::read(&orphan).unwrap(), candidate);
    reopened.save(None).unwrap();
    assert_eq!(std::fs::read(&manifest).unwrap(), before);
    assert_eq!(
        std::fs::read(&orphan).unwrap(),
        candidate,
        "a later save must leave the recovery candidate intact"
    );
}

#[test]
fn tolerant_loading_preserves_the_original_file_and_unknown_fields() {
    let scratch = tempfile::tempdir().unwrap();
    let mut document = Editor::new().to_document(&settings());
    document["pluginMetadata"] = json!({"keep": [1, 2, 3]});
    document["timelines"][0]["clips"] = json!([
        {"id":"kept", "trackId":"T1", "kind":"text", "duration":3.0},
        {"id":"orphan", "trackId":"missing", "kind":"text"},
        {"id":"invalid", "trackId":"T1", "kind":"text", "start":"soon"}
    ]);
    let before = serde_json::to_vec_pretty(&document).unwrap();
    let manifest = scratch.path().join("concat.json");
    std::fs::write(&manifest, &before).unwrap();
    let loaded = open(scratch.path());
    let ids: Vec<_> = loaded
        .project()
        .active()
        .clips
        .iter()
        .map(|c| c.id.as_str())
        .collect();
    assert_eq!(
        ids,
        ["kept"],
        "existing tolerant-reader policy filters malformed/orphan entries"
    );
    assert_eq!(
        loaded.document()["pluginMetadata"],
        document["pluginMetadata"]
    );
    assert_eq!(
        std::fs::read(&manifest).unwrap(),
        before,
        "inspection must preserve the source bytes; this test does not resave normalized entries"
    );
}

#[cfg(target_os = "linux")]
#[test]
fn a_partial_write_failure_preserves_the_manifest_and_does_not_advance_save_order() {
    const CHILD_ROOT: &str = "CLOVERCUT_STABILITY_PARTIAL_WRITE_ROOT";
    const MARKER: &str = ".disposable-stability-fixture";
    if let Some(path) = std::env::var_os(CHILD_ROOT) {
        let root = std::fs::canonicalize(path).unwrap();
        assert!(root.starts_with(std::fs::canonicalize(std::env::temp_dir()).unwrap()));
        assert!(
            root.file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .starts_with("clovercut-phase2-fault-")
        );
        assert_eq!(
            std::fs::read(root.join(MARKER)).unwrap(),
            b"disposable fixture only"
        );
        let manifest = projects::manifest_path(&root);
        let before = std::fs::read(&manifest).unwrap();
        assert!(before.len() > 256);
        let mut candidate = projects::read_document(root.to_str().unwrap())
            .unwrap()
            .unwrap();
        candidate["retryMarker"] = json!("preserved after a partial write");
        let older = projects::ticket();
        let newer = projects::ticket();
        // The resource limit and signal disposition affect only this exact-test
        // subprocess, never the test runner, desktop or another project.
        let (previous_limit, previous_signal) = unsafe {
            let mut previous = std::mem::zeroed::<libc::rlimit>();
            assert_eq!(libc::getrlimit(libc::RLIMIT_FSIZE, &mut previous), 0);
            let signal = libc::signal(libc::SIGXFSZ, libc::SIG_IGN);
            assert_ne!(signal, libc::SIG_ERR);
            let limited = libc::rlimit {
                rlim_cur: 256,
                rlim_max: previous.rlim_max,
            };
            assert_eq!(libc::setrlimit(libc::RLIMIT_FSIZE, &limited), 0);
            (previous, signal)
        };
        let failed = projects::save_ordered(root.to_str().unwrap(), &candidate, newer);
        unsafe {
            assert_eq!(libc::setrlimit(libc::RLIMIT_FSIZE, &previous_limit), 0);
            assert_ne!(libc::signal(libc::SIGXFSZ, previous_signal), libc::SIG_ERR);
        }
        let error = failed.expect_err("a 256-byte file limit forces a partial-write failure");
        assert!(error.contains("could not write"), "{error}");
        assert_eq!(std::fs::read(&manifest).unwrap(), before);
        assert!(
            !std::fs::read_dir(&root)
                .unwrap()
                .filter_map(Result::ok)
                .any(|e| e
                    .file_name()
                    .to_string_lossy()
                    .starts_with("concat.json.saving"))
        );
        assert_eq!(
            projects::save_ordered(root.to_str().unwrap(), &candidate, older).unwrap(),
            projects::Saved::Written,
            "a failed newer ticket must not suppress a valid retry"
        );
        assert_eq!(open(&root).project().active().tracks.len(), 5);
        println!("partial-write injection and ordered retry: ok");
        return;
    }

    let scratch = tempfile::Builder::new()
        .prefix("clovercut-phase2-fault-")
        .tempdir()
        .unwrap();
    std::fs::write(scratch.path().join(MARKER), b"disposable fixture only").unwrap();
    let mut session = open(scratch.path());
    session
        .apply(Command::AddTrack {
            bottom: false,
            kind: None,
            insert_at: None,
        })
        .unwrap();
    session.save(None).unwrap();
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "a_partial_write_failure_preserves_the_manifest_and_does_not_advance_save_order",
            "--exact",
            "--nocapture",
            "--test-threads=1",
        ])
        .env(CHILD_ROOT, scratch.path())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .contains("partial-write injection and ordered retry: ok")
    );
    assert_eq!(
        projects::read_document(scratch.path().to_str().unwrap())
            .unwrap()
            .unwrap()["retryMarker"],
        "preserved after a partial write"
    );
    assert_eq!(open(scratch.path()).project().active().tracks.len(), 5);
}

/// SIGKILL affects only an exact-test child writing a marked, unique fixture.
/// Observing a nonempty temporary coordinates a real interrupted write without
/// a production fault switch. This is process loss, not physical power loss.
#[cfg(target_os = "linux")]
#[test]
fn an_interrupted_save_keeps_the_committed_document_and_recovery_candidate() {
    use std::os::unix::process::ExitStatusExt;
    use std::time::{Duration, Instant};
    const CHILD: &str = "CLOVERCUT_INTERRUPTED_SAVE_FIXTURE";
    const MARKER: &str = ".disposable-stability-fixture";
    if let Some(root) = std::env::var_os(CHILD) {
        let root = std::fs::canonicalize(root).unwrap();
        let temporary_root = std::fs::canonicalize(std::env::temp_dir()).unwrap();
        assert!(root.starts_with(temporary_root));
        assert!(
            root.file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("clovercut-phase2-interrupt-")
        );
        assert_eq!(
            std::fs::read(root.join(MARKER)).unwrap(),
            b"interrupted-save-fixture"
        );
        let mut document = projects::read_document(root.to_str().unwrap())
            .unwrap()
            .unwrap();
        document["pluginMetadata"] = json!({"interrupted": "x".repeat(64 * 1024 * 1024)});
        projects::save(root.to_str().unwrap(), &document).unwrap();
        panic!("the parent must interrupt the marked temporary write");
    }
    let scratch = tempfile::Builder::new()
        .prefix("clovercut-phase2-interrupt-")
        .tempdir()
        .unwrap();
    let root = scratch.path();
    std::fs::write(root.join(MARKER), b"interrupted-save-fixture").unwrap();
    let mut baseline = Editor::new().to_document(&settings());
    baseline["pluginMetadata"] = json!({"committed": "keep"});
    projects::save(root.to_str().unwrap(), &baseline).unwrap();
    let original = std::fs::read(projects::manifest_path(root)).unwrap();
    let mut child = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "an_interrupted_save_keeps_the_committed_document_and_recovery_candidate",
            "--exact",
            "--nocapture",
            "--test-threads=1",
        ])
        .env(CHILD, root)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .unwrap();
    let until = Instant::now() + Duration::from_secs(20);
    let candidate = loop {
        let candidate = std::fs::read_dir(root)
            .unwrap()
            .filter_map(Result::ok)
            .find_map(|entry| {
                (entry.file_name().to_string_lossy().contains(".saving.")
                    && entry.metadata().is_ok_and(|meta| meta.len() > 0))
                .then(|| entry.path())
            });
        if let Some(candidate) = candidate {
            break Some(candidate);
        }
        if child.try_wait().unwrap().is_some() || Instant::now() >= until {
            break None;
        }
        std::thread::yield_now();
    };
    // Always reap the child, including a failed harness synchronization.
    let _ = child.kill();
    let status = child.wait().unwrap();
    let candidate = candidate.expect("observes a real partially written temporary");
    assert_eq!(status.signal(), Some(9));
    assert_eq!(
        std::fs::read(projects::manifest_path(root)).unwrap(),
        original
    );
    assert_eq!(
        projects::read_document(root.to_str().unwrap()).unwrap(),
        Some(baseline)
    );
    let partial =
        std::fs::read(&candidate).expect("the interrupted process leaves its own candidate");
    assert!(!partial.is_empty());
    let mut session = open(root);
    assert_eq!(session.project().active().tracks.len(), 4);
    session.save(None).unwrap();
    assert_eq!(
        std::fs::read(candidate).unwrap(),
        partial,
        "opening and saving do not silently remove crash recovery data"
    );
}
