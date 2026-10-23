// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Clovercut contributors

//! Direct access to the YouTube Free Audio Library catalog, using the public
//! community index (ThibaultJanBeyer/YouTube-Free-Audio-Library-API).
//! Audio is fetched directly from Google Drive / Google UserContent endpoints.

use std::io::Read;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use serde::Deserialize;
use url::Url;

use crate::dirs::AppDirs;
use crate::sounds::{Catalogue, Listed};

const API_URL: &str = "https://raw.githubusercontent.com/ThibaultJanBeyer/YouTube-Free-Audio-Library-API/master/api.json";
const BACKUP_API_URL: &str = "https://thibaultjanbeyer.github.io/YouTube-Free-Audio-Library-API/api.json";

#[derive(Deserialize, Clone, Debug)]
struct ApiEntry {
    #[serde(default)]
    id: String,
    #[serde(default)]
    name: String,
}

#[derive(Deserialize, Clone, Debug)]
#[allow(dead_code)]
struct ApiResponse {
    #[serde(default)]
    count: usize,
    #[serde(default)]
    all: Vec<ApiEntry>,
}

static CACHED_ENTRIES: Mutex<Option<(Instant, Vec<ApiEntry>)>> = Mutex::new(None);

fn clean_name(raw: &str) -> String {
    let without_ext = raw.strip_suffix(".mp3").unwrap_or(raw);
    let spaced = without_ext.replace('_', " ");
    spaced.trim().to_string()
}

/// Whether the given URL points to a Google Drive / Google UserContent stream.
pub fn is_audio(value: &str) -> bool {
    Url::parse(value).is_ok_and(|url| {
        (url.host_str() == Some("drive.usercontent.google.com")
            || url.host_str() == Some("docs.google.com"))
            && (url.scheme() == "https" || url.scheme() == "http")
    })
}

fn download_url(id: &str) -> String {
    format!("https://drive.usercontent.google.com/download?id={id}&export=download")
}

fn load_entries(dirs: &AppDirs) -> Result<Vec<ApiEntry>, String> {
    let mut lock = CACHED_ENTRIES.lock().map_err(|e| e.to_string())?;
    if let Some((when, ref entries)) = *lock {
        if when.elapsed() < Duration::from_secs(3600 * 24) && !entries.is_empty() {
            return Ok(entries.clone());
        }
    }

    let cache_dir = crate::sounds::cache_dir(dirs);
    let cache_file = cache_dir.join("youtube_audio_catalog.json");
    if let Ok(content) = std::fs::read_to_string(&cache_file) {
        if let Ok(parsed) = serde_json::from_str::<ApiResponse>(&content) {
            if !parsed.all.is_empty() {
                *lock = Some((Instant::now(), parsed.all.clone()));
                return Ok(parsed.all);
            }
        }
    }

    // Fetch from API
    let agent = crate::net::agent(Duration::from_secs(15), Duration::from_secs(30));
    let resp = agent
        .get(API_URL)
        .set("User-Agent", "Mozilla/5.0 (compatible; Clovercut/0.1)")
        .call()
        .or_else(|_| {
            agent
                .get(BACKUP_API_URL)
                .set("User-Agent", "Mozilla/5.0 (compatible; Clovercut/0.1)")
                .call()
        });

    let response = match resp {
        Ok(r) => r,
        Err(e) => return Err(format!("Could not reach YouTube Audio Library index: {e}")),
    };

    let mut body = String::new();
    response
        .into_reader()
        .read_to_string(&mut body)
        .map_err(|e| e.to_string())?;

    let parsed: ApiResponse = serde_json::from_str(&body)
        .map_err(|e| format!("Invalid YouTube Audio catalog: {e}"))?;
    if parsed.all.is_empty() {
        return Err("YouTube Audio Library catalog is empty".into());
    }

    let _ = std::fs::create_dir_all(&cache_dir);
    let _ = std::fs::write(&cache_file, &body);

    *lock = Some((Instant::now(), parsed.all.clone()));
    Ok(parsed.all)
}

/// Search the YouTube Audio Library index, matching query against song names.
pub fn search(
    dirs: &AppDirs,
    query: &str,
    category: usize,
    page: usize,
) -> Result<Catalogue, String> {
    let entries = load_entries(dirs)?;
    let q = query.trim().to_lowercase();

    let filtered: Vec<&ApiEntry> = entries
        .iter()
        .filter(|e| q.is_empty() || e.name.to_lowercase().contains(&q))
        .collect();

    let per_page = 24;
    let total = filtered.len();
    let start = page * per_page;
    let end = (start + per_page).min(total);

    let sounds = if start < total {
        filtered[start..end]
            .iter()
            .map(|entry| {
                let id = format!("ytal-{}", entry.id);
                let name = clean_name(&entry.name);
                Listed {
                    file: format!("{}.mp3", entry.id),
                    id,
                    name,
                    category: "Music".into(),
                    duration: 0.0,
                    bytes: 0,
                    license: "YouTube Audio Library (Royalty Free)".into(),
                    source: "https://www.youtube.com/audiolibrary".into(),
                    author: "YouTube Creator Studio".into(),
                    sha256: String::new(),
                    download_url: download_url(&entry.id),
                }
            })
            .collect()
    } else {
        Vec::new()
    };

    Ok(Catalogue {
        version: 2,
        query: query.to_string(),
        category,
        page,
        more: end < total,
        sounds,
    })
}
