// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Clovercut contributors

//! Prepare cross-project clipboard files off the UI thread, with owned staging.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

use concat_project::{Project, clipboard::Clipboard};
use sha2::{Digest, Sha256};

const SOURCE: &str = "clovercutClipboardSource";

/// Prepared resources stay temporary until the document accepts the whole paste.
/// Dropping a cancelled, failed or stale result removes only its own files.
pub struct PreparedClipboard {
    /// Clipboard whose project-owned references now belong to the destination.
    pub clipboard: Clipboard,
    directory: Option<tempfile::TempDir>,
}

impl PreparedClipboard {
    /// Retain staged files after their references enter the project/undo history.
    pub fn commit(self) {
        if let Some(directory) = self.directory {
            let _ = directory.keep();
        }
    }
}

#[derive(Clone)]
struct Resolved {
    path: String,
    source: PathBuf,
    digest: Option<String>,
}

struct Stage<'a> {
    source: PathBuf,
    destination: PathBuf,
    cancel: &'a AtomicBool,
    directory: Option<tempfile::TempDir>,
    paths: HashMap<PathBuf, Resolved>,
    buffer: Vec<u8>,
}

impl Stage<'_> {
    fn resolve(&mut self, name: &str) -> Result<Resolved, String> {
        self.check_cancel()?;
        let original = Path::new(name);
        let absolute = if original.is_absolute() {
            original.to_path_buf()
        } else {
            self.source.join(original)
        };
        let absolute = absolute.canonicalize().unwrap_or(absolute);
        if let Some(path) = self.paths.get(&absolute) {
            return Ok(path.clone());
        }
        let owned = self.source != self.destination && absolute.starts_with(&self.source);
        let (path, digest) = if owned {
            if self.directory.is_none() {
                let parent = self.destination.join("media").join("pasted");
                fs::create_dir_all(&parent).map_err(|e| e.to_string())?;
                self.directory = Some(
                    tempfile::Builder::new()
                        .prefix("paste-")
                        .tempdir_in(parent)
                        .map_err(|e| e.to_string())?,
                );
            }
            let extension = absolute
                .extension()
                .and_then(|e| e.to_str())
                .filter(|s| s.len() <= 12 && s.chars().all(|c| c.is_ascii_alphanumeric()))
                .unwrap_or("bin");
            let target = self.directory.as_ref().unwrap().path().join(format!(
                "{}.{}",
                self.paths.len(),
                extension
            ));
            let mut input =
                fs::File::open(&absolute).map_err(|e| format!("{}: {e}", absolute.display()))?;
            let before = input.metadata().map_err(|e| e.to_string())?;
            let mut output = fs::OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(&target)
                .map_err(|e| e.to_string())?;
            let mut digest = Sha256::new();
            loop {
                self.check_cancel()?;
                let bytes = input.read(&mut self.buffer).map_err(|e| e.to_string())?;
                if bytes == 0 {
                    break;
                }
                digest.update(&self.buffer[..bytes]);
                output
                    .write_all(&self.buffer[..bytes])
                    .map_err(|e| e.to_string())?;
            }
            let after = input.metadata().map_err(|e| e.to_string())?;
            if before.len() != after.len() || before.modified().ok() != after.modified().ok() {
                return Err(
                    "A copied file changed while it was being read. Please copy it again.".into(),
                );
            }
            output.sync_all().map_err(|e| e.to_string())?;
            (target, Some(format!("{:x}", digest.finalize())))
        } else {
            (absolute.clone(), None)
        };
        let resolved = Resolved {
            path: path.to_string_lossy().into_owned(),
            source: absolute.clone(),
            digest,
        };
        self.paths.insert(absolute, resolved.clone());
        Ok(resolved)
    }

    fn check_cancel(&self) -> Result<(), String> {
        if self.cancel.load(Ordering::Relaxed) {
            Err("Paste cancelled.".into())
        } else {
            Ok(())
        }
    }

    fn digest(&mut self, path: &Path) -> Result<String, String> {
        let mut file = fs::File::open(path).map_err(|e| e.to_string())?;
        let mut digest = Sha256::new();
        loop {
            self.check_cancel()?;
            let bytes = file.read(&mut self.buffer).map_err(|e| e.to_string())?;
            if bytes == 0 {
                break;
            }
            digest.update(&self.buffer[..bytes]);
        }
        Ok(format!("{:x}", digest.finalize()))
    }
}

/// Resolve references and copy resources owned by a different source project.
/// External media remain references. Repeated imports reuse verified resources.
/// Call from a worker; copying and hashing are cancellable.
pub fn prepare(
    mut clipboard: Clipboard,
    source: &Path,
    destination: &Path,
    target: &Project,
    cancel: &AtomicBool,
) -> Result<PreparedClipboard, String> {
    let mut stage = Stage {
        source: source
            .canonicalize()
            .unwrap_or_else(|_| source.to_path_buf()),
        destination: destination.canonicalize().map_err(|e| e.to_string())?,
        cancel,
        directory: None,
        paths: HashMap::new(),
        buffer: vec![0; 1024 * 1024],
    };
    for item in &mut clipboard.media {
        if item.path.is_empty() && item.placeholder {
            continue;
        }
        let resolved = stage.resolve(&item.path)?;
        item.path = resolved.path;
        if resolved.digest.is_none() {
            // canonicalize can add a Windows verbatim-path prefix or resolve a
            // symlink. Keep an existing bin path for that same physical file.
            if let Some(existing) = target
                .media
                .iter()
                .find(|m| Path::new(&m.path).canonicalize().ok().as_ref() == Some(&resolved.source))
            {
                item.path = existing.path.clone();
            }
        }
        if let Some(digest) = resolved.digest {
            let provenance =
                serde_json::json!({ "path": resolved.source.to_string_lossy(), "sha256": digest });
            for existing in &target.media {
                if existing.extra.get(SOURCE) == Some(&provenance)
                    && stage.digest(Path::new(&existing.path)).ok().as_deref() == Some(&digest)
                {
                    item.path = existing.path.clone();
                    break;
                }
            }
            item.extra.insert(SOURCE.into(), provenance);
        }
    }
    for font in &mut clipboard.fonts {
        let resolved = stage.resolve(&font.path)?;
        font.path = resolved.path;
        if let Some(existing) = target
            .fonts
            .iter()
            .find(|f| f.family == font.family && f.path != font.path)
        {
            let digest = match resolved.digest {
                Some(digest) => digest,
                None => stage.digest(Path::new(&font.path))?,
            };
            if stage.digest(Path::new(&existing.path)).ok().as_deref() == Some(&digest) {
                font.path = existing.path.clone();
            }
        }
    }
    stage.check_cancel()?;
    // Deduplication can make temporary copies unnecessary. Delete only files
    // owned by this stage, never files already referenced by the destination.
    if let Some(directory) = &stage.directory {
        let used: HashSet<_> = clipboard
            .media
            .iter()
            .map(|m| m.path.as_str())
            .chain(clipboard.fonts.iter().map(|f| f.path.as_str()))
            .collect();
        for file in fs::read_dir(directory.path()).map_err(|e| e.to_string())? {
            let file = file.map_err(|e| e.to_string())?.path();
            if !used.contains(file.to_string_lossy().as_ref()) {
                fs::remove_file(file).map_err(|e| e.to_string())?;
            }
        }
        if fs::read_dir(directory.path())
            .map_err(|e| e.to_string())?
            .next()
            .is_none()
        {
            stage.directory.take();
        }
    }
    if let Some(directory) = &stage.directory {
        // Resource names must reach disk before a saved manifest can point at
        // them. Reuse the project's platform-specific directory-sync policy.
        crate::projects::sync_directory(directory.path())?;
        crate::projects::sync_directory(directory.path().parent().unwrap())?;
        crate::projects::sync_directory(&stage.destination.join("media"))?;
        crate::projects::sync_directory(&stage.destination)?;
    }
    Ok(PreparedClipboard {
        clipboard,
        directory: stage.directory,
    })
}
