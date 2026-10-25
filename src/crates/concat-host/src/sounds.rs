// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Jareer and Concat contributors

//! The user's own sound library: the Audio page's Mine shelf.
//!
//! A sound brought in here is copied into `<library>/audio/` (see
//! [`AppDirs::library`]), so it is there in every project and survives the
//! original being moved or deleted. Beside the copies is `library.json`,
//! the index: each sound's name, its file, how long it is and a small
//! envelope of it - sixty-four bars, read once at import - so the shelf
//! draws every card without decoding anything when it opens.
//!
//! Beside them, OpenGameArt's public CC0 audio search. Original attachments
//! are licence-checked and downloaded on first use into `<data>/sounds/`; see
//! `docs/AUDIO_LIBRARY.md`.
//!
//! Nothing here touches a project. A sound put on the timeline is copied
//! again into that project's own folder by the window, so a project never
//! points into the library and removing a sound from it breaks nothing.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::AppDirs;

/// How many bars a card's envelope has.
pub const BARS: usize = 64;

/// The extensions the library takes: sound files, the way the bin's
/// import names them. A video's soundtrack is not a library sound.
pub const EXTENSIONS: [&str; 9] = [
    "mp3", "wav", "aac", "m4a", "flac", "ogg", "opus", "aif", "aiff",
];

/// One sound in the library.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Sound {
    /// Stable for as long as the sound is in the library; what a drag
    /// carries and what the index is keyed by.
    pub id: String,
    /// What the card says: the file's name without its extension, at
    /// first.
    pub name: String,
    /// The copy's file name inside the library folder.
    pub file: String,
    /// Seconds.
    pub duration: f64,
    /// A shelf of the user's choosing; empty for none. Kept for the day
    /// the shelf offers categories of one's own.
    #[serde(default)]
    pub category: String,
    /// The envelope: [`BARS`] peaks, each `0..=1`, quantised to hundredths.
    #[serde(default)]
    pub bars: Vec<f32>,
    /// When it came in, as Unix seconds: the shelf's order, newest first.
    #[serde(default)]
    pub added: u64,
}

/// The index on disk.
#[derive(Serialize, Deserialize, Default)]
struct Index {
    #[serde(default)]
    sounds: Vec<Sound>,
}

/// The library: its folder and what is in it.
#[derive(Clone, Debug, Default)]
pub struct Library {
    /// The folder: the copies and the index beside them.
    pub dir: PathBuf,
    /// Newest first.
    pub sounds: Vec<Sound>,
}

impl Library {
    /// Where the library lives on this machine.
    pub fn dir_in(dirs: &AppDirs) -> PathBuf {
        dirs.library().join("audio")
    }

    /// The library as the index has it. A missing or unreadable index is
    /// an empty library, and a sound whose copy has gone is left out.
    pub fn open(dirs: &AppDirs) -> Library {
        Library::at(Self::dir_in(dirs))
    }

    /// The library in `dir`; see [`Library::open`].
    pub fn at(dir: PathBuf) -> Library {
        let index: Index = std::fs::read(dir.join("library.json"))
            .ok()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .unwrap_or_default();
        let sounds = index
            .sounds
            .into_iter()
            .filter(|sound| is_plain_name(&sound.file) && dir.join(&sound.file).is_file())
            .collect();
        Library { dir, sounds }
    }

    /// The copy of `sound` on disk.
    pub fn path_of(&self, sound: &Sound) -> PathBuf {
        self.dir.join(&sound.file)
    }

    /// The sound by its id.
    pub fn get(&self, id: &str) -> Option<&Sound> {
        self.sounds.iter().find(|sound| sound.id == id)
    }

    /// Writes the index, through a temporary file and a rename, so a crash
    /// mid-write leaves the last one whole.
    fn save(&self) -> Result<(), String> {
        std::fs::create_dir_all(&self.dir)
            .map_err(|error| format!("could not create {}: {error}", self.dir.display()))?;
        let index = Index {
            sounds: self.sounds.clone(),
        };
        let bytes = serde_json::to_vec_pretty(&index).map_err(|error| error.to_string())?;
        let path = self.dir.join("library.json");
        let partial = self.dir.join("library.json.partial");
        std::fs::write(&partial, bytes)
            .and_then(|()| std::fs::rename(&partial, &path))
            .map_err(|error| format!("could not write {}: {error}", path.display()))
    }

    /// Takes in a sound [`prepare`] made, and writes the index.
    pub fn add(&mut self, sound: Sound) -> Result<(), String> {
        self.sounds.retain(|held| held.id != sound.id);
        self.sounds.insert(0, sound);
        self.save()
    }

    /// Deletes the copy and forgets the sound. A copy already gone is not
    /// an error: what the person asked for is that it be gone.
    pub fn remove(&mut self, id: &str) -> Result<(), String> {
        let Some(at) = self.sounds.iter().position(|sound| sound.id == id) else {
            return Ok(());
        };
        let sound = self.sounds.remove(at);
        let path = self.path_of(&sound);
        if let Err(error) = std::fs::remove_file(&path)
            && error.kind() != std::io::ErrorKind::NotFound
        {
            self.sounds.insert(at, sound);
            return Err(format!("could not delete {}: {error}", path.display()));
        }
        self.save()
    }
}

/// Whether a path is a sound the library takes, by its extension.
pub fn is_sound(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            EXTENSIONS
                .iter()
                .any(|known| known.eq_ignore_ascii_case(extension))
        })
}

/// A file name with nothing in it that could walk out of a folder.
fn is_plain_name(name: &str) -> bool {
    !name.is_empty()
        && name != "."
        && name != ".."
        && !name.contains(['/', '\\', ':'])
        && Path::new(name).file_name().is_some_and(|file| file == name)
}

/// Copies `source` into the library folder `dir` and reads what its card
/// needs: how long it is, and its envelope. Slow - a decode of the whole
/// file - so a worker's; [`Library::add`] then takes the result in on the
/// window's thread.
///
/// The copy keeps the file's own name, numbered when the folder already
/// has one by it, and a file that is not sound is refused before anything
/// is copied.
pub fn prepare(dir: &Path, source: &Path) -> Result<Sound, String> {
    if !is_sound(source) {
        return Err(format!("{} is not a sound file", source.display()));
    }
    let summary = crate::media::probe(&source.to_string_lossy())?;
    if summary.audio.is_none() {
        return Err(format!("{} has no sound in it", source.display()));
    }
    std::fs::create_dir_all(dir)
        .map_err(|error| format!("could not create {}: {error}", dir.display()))?;
    let stem = source
        .file_stem()
        .map(|stem| stem.to_string_lossy().into_owned())
        .unwrap_or_else(|| "Sound".to_owned());
    let extension = source
        .extension()
        .map(|extension| extension.to_string_lossy().to_ascii_lowercase())
        .unwrap_or_default();
    let file = free_name(dir, &stem, &extension);
    let target = dir.join(&file);
    std::fs::copy(source, &target)
        .map_err(|error| format!("could not copy {}: {error}", source.display()))?;
    let duration = summary.duration.unwrap_or(0.0);
    let bars = envelope(&target, duration).unwrap_or_default();
    let added = now();
    Ok(Sound {
        id: format!(
            "{:012x}",
            crate::media::fnv1a(format!("{file}|{added}").as_bytes()) & 0xffff_ffff_ffff
        ),
        name: stem,
        file,
        duration,
        category: String::new(),
        bars,
        added,
    })
}

/// `stem.extension` in `dir`, or `stem 2.extension` and on, whichever is
/// free, with anything that could not be a plain file name taken out.
fn free_name(dir: &Path, stem: &str, extension: &str) -> String {
    let clean: String = stem
        .chars()
        .map(|c| {
            if c.is_control() || matches!(c, '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|') {
                '_'
            } else {
                c
            }
        })
        .collect();
    let clean = clean.trim().trim_matches('.');
    let clean = if clean.is_empty() { "Sound" } else { clean };
    let named = |n: u32| {
        let stem = if n == 1 {
            clean.to_owned()
        } else {
            format!("{clean} {n}")
        };
        if extension.is_empty() {
            stem
        } else {
            format!("{stem}.{extension}")
        }
    };
    (1..)
        .map(named)
        .find(|name| name != "library.json" && !dir.join(name).exists())
        .unwrap_or_else(|| "Sound".to_owned())
}

/// The envelope a card draws: [`BARS`] peaks across the whole file.
pub fn envelope(path: &Path, duration: f64) -> Option<Vec<f32>> {
    // A hundred buckets a second is plenty for sixty-four bars, and a
    // tenth of the work of the timeline's resolution.
    let peaks = concat_media::peaks::extract(path, 100, None).ok()?;
    let span = if duration > 0.0 {
        duration as f32
    } else {
        peaks.len() as f32 / peaks.buckets_per_second.max(1.0)
    };
    if span <= 0.0 {
        return None;
    }
    let pitch = span / BARS as f32;
    Some(
        (0..BARS)
            .map(|bar| {
                let (low, high) = peaks.extremes(bar as f32 * pitch, (bar + 1) as f32 * pitch);
                (high.max(-low).clamp(0.0, 1.0) * 100.0).round() / 100.0
            })
            .collect(),
    )
}

fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |since| since.as_secs())
}

// ── OpenGameArt sounds ─────────────────────────────────────────────────

/// Cached public search results; see `docs/AUDIO_LIBRARY.md`.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct Catalogue {
    /// The cache format: 2 for OpenGameArt results.
    #[serde(default)]
    pub version: u32,
    /// Search text that produced these cached results.
    #[serde(default)]
    pub query: String,
    /// The selected audio category, zero meaning all audio.
    #[serde(default)]
    pub category: usize,
    /// Zero-based result page.
    #[serde(default)]
    pub page: usize,
    /// The source offers another page of results.
    #[serde(default)]
    pub more: bool,
    /// The sounds, in the order the shelf shows them.
    #[serde(default)]
    pub sounds: Vec<Listed>,
}

/// Source-provided credits retained alongside cached audio.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct Credits {
    /// Source attribution text or a credit assembled from the source metadata.
    pub text: String,
    /// None means that the source licence has not declared an attribution rule.
    pub required: Option<bool>,
}

/// One public audio result.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct Listed {
    /// Stable for ever: `[a-z0-9-]`.
    pub id: String,
    /// What the row shows.
    pub name: String,
    /// Whoosh, Impact, Riser, Ambience or Music.
    #[serde(default)]
    pub category: String,
    /// The local cache file name, pending until the original is resolved.
    pub file: String,
    /// Seconds.
    #[serde(default)]
    pub duration: f64,
    /// Its size, for the progress bar; 0 when not given.
    #[serde(default)]
    pub bytes: u64,
    /// The source licence, preserved independently for each provider.
    #[serde(default)]
    pub license: String,
    /// Where it was found.
    #[serde(default)]
    pub source: String,
    /// Who made it, when known.
    #[serde(default)]
    pub author: String,
    /// Locally observed SHA-256, used to detect cache corruption.
    #[serde(default)]
    pub sha256: String,
    /// Original file URL, resolved and licence-checked from OpenGameArt.
    #[serde(default)]
    pub download_url: String,
    /// Credit metadata retained when the audio is downloaded for offline use.
    #[serde(default)]
    pub credits: Credits,
}

impl Listed {
    /// Whether the entry can be offered: a plain file name, an id of the
    /// shape the provider uses, a trusted download host, and its licence.
    pub fn is_offered(&self) -> bool {
        let digest_ok = self.sha256.is_empty()
            || (self.sha256.len() == 64 && self.sha256.chars().all(|c| c.is_ascii_hexdigit()));
        if !is_plain_name(&self.file) || !digest_ok {
            return false;
        }
        if let Some(id) = self.id.strip_prefix("ytcc-") {
            return self.file == format!("{id}.mp3")
                && self.source == crate::youtube_audio::SOURCE
                && !self.license.is_empty()
                && crate::youtube_audio::is_track_endpoint(&self.download_url, id);
        }
        if let Some(id) = self.id.strip_prefix("ytal-") {
            return !id.is_empty()
                && id
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_'))
                && self.file == format!("{id}.mp3")
                && self.source == "https://www.youtube.com/audiolibrary"
                && crate::youtube_audio::is_audio(&self.download_url);
        }
        if let Some(id) = self.id.strip_prefix("fs-") {
            return !id.is_empty()
                && id.chars().all(|c| c.is_ascii_digit())
                && self.file == format!("{id}.mp3")
                && self.source == format!("https://freesound.org/s/{id}/")
                && crate::freesound::is_audio(&self.download_url)
                && !self.license.is_empty();
        }
        let license = self.license.to_ascii_uppercase();
        !self.id.is_empty()
            && self
                .id
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
            && ((self.sha256.len() == 64)
                || (self.id.starts_with("oga-")
                    && crate::opengameart::is_source(&self.source)
                    && (self.download_url.is_empty()
                        || crate::opengameart::is_audio(&self.download_url))))
            && (license == "CC0" || license == "CC0-1.0")
    }
}

impl Catalogue {
    /// Read from the manifest's bytes, keeping only the entries that can be
    /// offered.
    pub fn parse(bytes: &[u8]) -> Result<Catalogue, String> {
        let mut catalogue: Catalogue = serde_json::from_slice(bytes)
            .map_err(|error| format!("the sound list is not readable: {error}"))?;
        catalogue.sounds.retain(Listed::is_offered);
        for sound in &mut catalogue.sounds {
            sound.sha256.make_ascii_lowercase();
        }
        Ok(catalogue)
    }

    /// The sound by its id.
    pub fn get(&self, id: &str) -> Option<&Listed> {
        self.sounds.iter().find(|sound| sound.id == id)
    }
}

/// Where Concat's sounds and their manifest are cached: `<data>/sounds/`.
pub fn cache_dir(dirs: &AppDirs) -> PathBuf {
    dirs.data.join("sounds")
}

/// The manifest as last fetched, when it has been.
pub fn cached_catalogue(dirs: &AppDirs) -> Option<Catalogue> {
    let bytes = std::fs::read(cache_dir(dirs).join("manifest.json")).ok()?;
    let mut catalogue = Catalogue::parse(&bytes).ok()?;
    if catalogue.version != 2 {
        return None;
    }
    for sound in &mut catalogue.sounds {
        if let Some(known) = resolved_sound(dirs, sound) {
            *sound = known;
        }
    }
    Some(catalogue)
}

/// Search an audio catalogue provider directly, caching metadata.
pub fn search_catalogue_provider(
    dirs: &AppDirs,
    provider: usize,
    query: &str,
    category: usize,
    page: usize,
    freesound_key: &str,
    youtube_api: &str,
) -> Result<Option<Catalogue>, String> {
    let mut catalogue = match provider {
        1 => crate::youtube_audio::search(dirs, query, category, page, youtube_api)?,
        2 => crate::freesound::search(dirs, query, category, page, freesound_key)?,
        _ => return search_catalogue(dirs, query, category, page),
    };
    catalogue.sounds.retain(Listed::is_offered);
    for sound in &mut catalogue.sounds {
        if let Some(known) = resolved_sound(dirs, sound) {
            *sound = known;
        }
    }
    Ok(Some(catalogue))
}

/// Search OpenGameArt directly, and cache only the metadata locally.
pub fn search_catalogue(
    dirs: &AppDirs,
    query: &str,
    category: usize,
    page: usize,
) -> Result<Option<Catalogue>, String> {
    let mut catalogue = crate::opengameart::search(query, category, page)?;
    for sound in &mut catalogue.sounds {
        if let Some(mut known) = resolved_sound(dirs, sound) {
            known.category = sound.category.clone();
            *sound = known;
        }
    }
    let dir = cache_dir(dirs);
    if std::fs::create_dir_all(&dir).is_ok() {
        let path = dir.join("manifest.json");
        let partial = dir.join("manifest.json.partial");
        if let Ok(bytes) = serde_json::to_vec(&catalogue)
            && std::fs::write(&partial, bytes).is_ok()
        {
            let _ = std::fs::rename(&partial, &path);
        }
    }
    Ok(Some(catalogue))
}

/// The default public CC0 audio search.
pub fn fetch_catalogue(dirs: &AppDirs) -> Result<Option<Catalogue>, String> {
    search_catalogue(dirs, "", 0, 0)
}

/// A previously resolved first-party attachment, including its observed digest.
pub fn resolved_sound(dirs: &AppDirs, sound: &Listed) -> Option<Listed> {
    if !sound.is_offered() {
        return None;
    }
    let known: Listed = serde_json::from_slice(
        &std::fs::read(cache_dir(dirs).join(format!("{}.json", sound.id))).ok()?,
    )
    .ok()?;
    (known.id == sound.id
        && known.source == sound.source
        && known.is_offered()
        && known.sha256.len() == 64
        && (if sound.id.starts_with("ytcc-") {
            true
        } else if sound.id.starts_with("oga-") {
            crate::opengameart::is_audio(&known.download_url)
        } else {
            known.download_url == sound.download_url
        }))
    .then(|| {
        if sound.id.starts_with("ytcc-") {
            let mut current = sound.clone();
            current.sha256 = known.sha256;
            current.bytes = known.bytes;
            current.duration = known.duration;
            current
        } else {
            known
        }
    })
}

/// The source-derived cache path of a downloaded original attachment.
pub fn cached_file(dirs: &AppDirs, sound: &Listed) -> PathBuf {
    if sound.download_url.is_empty()
        && sound.id.starts_with("oga-")
        && let Some(known) = resolved_sound(dirs, sound)
    {
        return cached_file(dirs, &known);
    }
    use sha2::{Digest, Sha256};
    let key = if sound.id.starts_with("oga-") {
        format!("{:x}", Sha256::digest(sound.source.as_bytes()))
    } else if sound.id.starts_with("ytal-")
        || sound.id.starts_with("ytcc-")
        || sound.id.starts_with("fs-")
    {
        sound.id.clone()
    } else {
        sound.sha256.clone()
    };
    cache_dir(dirs).join(format!("{}-{}", &key[..key.len().min(12)], sound.file))
}

/// Download original audio directly from the source provider, after checking its own
/// licence and parameters.
pub fn fetch_sound(
    dirs: &AppDirs,
    sound: &Listed,
    cancel: &std::sync::atomic::AtomicBool,
    progress: &mut dyn FnMut(u64, u64),
) -> Result<PathBuf, String> {
    use sha2::{Digest, Sha256};
    use std::io::Read;
    if !sound.is_offered() {
        return Err("invalid audio library entry".into());
    }

    let known = resolved_sound(dirs, sound);
    let sound = known.as_ref().unwrap_or(sound);
    let (mut sound, urls) = if sound.id.starts_with("oga-") {
        let resolved = match resolved_sound(dirs, sound) {
            Some(known) => known,
            None => crate::opengameart::resolve(sound)?,
        };
        if !crate::opengameart::is_audio(&resolved.download_url) {
            return Err("invalid original audio URL".into());
        }
        let url = resolved.download_url.clone();
        (resolved, vec![url])
    } else if sound.id.starts_with("ytcc-") {
        let target = cached_file(dirs, sound);
        if target.is_file()
            && (sound.sha256.is_empty() || crate::models::verify(&target, &sound.sha256).is_ok())
        {
            return Ok(target);
        }
        let url = crate::youtube_audio::resolve_audio(sound)?;
        (sound.clone(), vec![url])
    } else if sound.id.starts_with("ytal-") {
        let drive_id = sound.id.trim_start_matches("ytal-");
        (
            sound.clone(),
            vec![
                sound.download_url.clone(),
                format!("https://docs.google.com/uc?export=download&id={drive_id}"),
            ],
        )
    } else if sound.id.starts_with("fs-") {
        if sound.download_url.is_empty() {
            return Err("Freesound sound has no preview URL".into());
        }
        (sound.clone(), vec![sound.download_url.clone()])
    } else {
        return Err("unsupported audio source".into());
    };

    let target = cached_file(dirs, &sound);
    if target.is_file()
        && (sound.sha256.is_empty() || crate::models::verify(&target, &sound.sha256).is_ok())
    {
        return Ok(target);
    }
    let dir = cache_dir(dirs);
    std::fs::create_dir_all(&dir).map_err(|error| error.to_string())?;
    let partial = target.with_extension("partial");
    let mut last = String::new();
    for url in urls {
        match crate::models::download(&url, &partial, sound.bytes, cancel, "cancelled", progress) {
            Ok(_) => {
                if !sound.sha256.is_empty()
                    && let Err(error) = crate::models::verify(&partial, &sound.sha256)
                {
                    let _ = std::fs::remove_file(&partial);
                    last = error;
                    continue;
                }
                let summary =
                    match crate::media::probe(partial.to_str().ok_or("invalid audio path")?) {
                        Ok(summary)
                            if summary.audio.is_some()
                                && summary.duration.is_some_and(|d| d > 0.0) =>
                        {
                            summary
                        }
                        _ => {
                            let _ = std::fs::remove_file(&partial);
                            last = "The source did not provide playable audio".into();
                            continue;
                        }
                    };
                sound.duration = summary.duration.unwrap_or(0.0);
                sound.bytes = std::fs::metadata(&partial)
                    .map_err(|error| error.to_string())?
                    .len();
                let mut hash = Sha256::new();
                let mut file = std::fs::File::open(&partial).map_err(|error| error.to_string())?;
                let mut buffer = [0u8; 65536];
                loop {
                    let n = file.read(&mut buffer).map_err(|error| error.to_string())?;
                    if n == 0 {
                        break;
                    }
                    hash.update(&buffer[..n]);
                }
                sound.sha256 = format!("{:x}", hash.finalize());
                std::fs::rename(&partial, &target).map_err(|error| error.to_string())?;
                {
                    let path = dir.join(format!("{}.json", sound.id));
                    let metadata = serde_json::to_vec(&sound).map_err(|error| error.to_string())?;
                    let temporary = path.with_extension("json.partial");
                    std::fs::write(&temporary, metadata)
                        .and_then(|()| std::fs::rename(&temporary, &path))
                        .map_err(|error| error.to_string())?;
                }
                return Ok(target);
            }
            Err(error) if error == "cancelled" => return Err(error),
            Err(error) => last = error,
        }
    }
    Err(last)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_audio_entries_validate_without_weakening_legacy_catalogues() {
        let fs = Listed {
            id: "fs-123".into(),
            file: "123.mp3".into(),
            name: "Rain".into(),
            category: "Effects".into(),
            duration: 1.0,
            bytes: 0,
            license: "https://creativecommons.org/licenses/by/4.0/".into(),
            source: "https://freesound.org/s/123/".into(),
            author: "Artist".into(),
            sha256: String::new(),
            download_url: "https://cdn.freesound.org/previews/123/123-hq.mp3".into(),
            credits: Credits::default(),
        };
        assert!(fs.is_offered());
        let mut forged = fs.clone();
        forged.download_url = "https://evilfreesound.org/audio.mp3".into();
        assert!(!forged.is_offered());
        forged = fs.clone();
        forged.file = "../123.mp3".into();
        assert!(!forged.is_offered());
        let mut yt = fs;
        yt.id = "ytal-AbC_123-x".into();
        yt.file = "AbC_123-x.mp3".into();
        yt.source = "https://www.youtube.com/audiolibrary".into();
        yt.download_url =
            "https://drive.usercontent.google.com/download?id=AbC_123-x&export=download".into();
        assert!(yt.is_offered());
        yt.download_url = "https://drive.usercontent.google.com.evil.test/download".into();
        assert!(!yt.is_offered());
    }

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("concat-sounds-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn a_name_is_numbered_rather_than_overwritten() {
        let dir = scratch("names");
        assert_eq!(free_name(&dir, "Whoosh", "wav"), "Whoosh.wav");
        std::fs::write(dir.join("Whoosh.wav"), b"x").unwrap();
        assert_eq!(free_name(&dir, "Whoosh", "wav"), "Whoosh 2.wav");
        assert_eq!(free_name(&dir, "../up", "wav"), "_up.wav");
        assert_eq!(free_name(&dir, "  ", "mp3"), "Sound.mp3");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn only_sound_files_are_taken() {
        assert!(is_sound(Path::new("/a/Rain.WAV")));
        assert!(is_sound(Path::new("b.m4a")));
        assert!(!is_sound(Path::new("clip.mp4")));
        assert!(!is_sound(Path::new("noextension")));
    }

    /// The index keeps what is added, newest first, survives a reopen, and
    /// forgets a sound whose copy has gone; a removal deletes the copy.
    #[test]
    fn the_index_round_trips_and_a_removal_deletes_the_copy() {
        let dir = scratch("index");
        let mut library = Library::at(dir.clone());
        assert!(library.sounds.is_empty());
        for (id, file) in [("a", "One.wav"), ("b", "Two.wav")] {
            std::fs::write(dir.join(file), b"riff").unwrap();
            library
                .add(Sound {
                    id: id.into(),
                    name: file.trim_end_matches(".wav").into(),
                    file: file.into(),
                    duration: 1.5,
                    category: String::new(),
                    bars: vec![0.5; BARS],
                    added: 1,
                })
                .unwrap();
        }
        let reopened = Library::at(dir.clone());
        assert_eq!(
            reopened
                .sounds
                .iter()
                .map(|s| s.id.as_str())
                .collect::<Vec<_>>(),
            ["b", "a"]
        );
        assert_eq!(reopened.get("a").unwrap().bars.len(), BARS);

        library.remove("b").unwrap();
        assert!(!dir.join("Two.wav").exists());
        std::fs::remove_file(dir.join("One.wav")).unwrap();
        assert!(
            Library::at(dir.clone()).sounds.is_empty(),
            "a missing copy is left out"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Only CC0 entries with a digest and a plain file name are offered.
    #[test]
    fn a_manifest_offers_only_what_it_can_check() {
        let digest = "A".repeat(64);
        let bytes = format!(
            r#"{{"version":1,"sounds":[
                {{"id":"whoosh-1","name":"Whoosh","category":"Whoosh","file":"w.ogg","duration":1.2,"license":"CC0-1.0","source":"s","sha256":"{digest}"}},
                {{"id":"bad","name":"No digest","file":"x.ogg","license":"CC0"}},
                {{"id":"paid","name":"Not CC0","file":"y.ogg","license":"CC-BY-4.0","sha256":"{digest}"}},
                {{"id":"up","name":"Escapes","file":"../z.ogg","license":"CC0","sha256":"{digest}"}}
            ]}}"#
        );
        let catalogue = Catalogue::parse(bytes.as_bytes()).expect("parses");
        assert_eq!(catalogue.sounds.len(), 1);
        let sound = catalogue.get("whoosh-1").unwrap();
        assert_eq!(sound.sha256, "a".repeat(64), "digests compare in lowercase");
        let dirs = AppDirs::under(Path::new("/cache"));
        assert_eq!(
            cached_file(&dirs, sound),
            Path::new("/cache/sounds/aaaaaaaaaaaa-w.ogg")
        );
        assert!(Catalogue::parse(b"not json").is_err());
    }

    /// An index that names a file outside the folder is not believed.
    #[test]
    fn an_index_cannot_point_out_of_the_folder() {
        let dir = scratch("escape");
        std::fs::write(
            dir.join("library.json"),
            br#"{"sounds":[{"id":"x","name":"x","file":"../x.wav","duration":1}]}"#,
        )
        .unwrap();
        assert!(Library::at(dir.clone()).sounds.is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
