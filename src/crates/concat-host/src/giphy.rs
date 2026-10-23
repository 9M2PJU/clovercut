// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Clovercut contributors

//! GIPHY sticker search. Credentials stay in the local configuration; API
//! errors never include request URLs, which contain the API key. Search results
//! and previews are session-only. Only selected originals enter a project.

use crate::AppDirs;
use serde::Deserialize;
use std::{
    io::{Read, Write},
    path::{Path, PathBuf},
    time::Duration,
};
use url::Url;

/// One animated sticker, in the provider's original result order.
#[derive(Clone, Debug)]
pub struct Sticker {
    /// Provider identifier, safe for local file names.
    pub id: String,
    /// Accessible title.
    pub title: String,
    /// Author's public display name.
    pub author: String,
    /// Original GIPHY page.
    pub source: String,
    /// Small still rendition, including all provider URL parameters.
    pub thumbnail: String,
    /// Original animated GIF, preserving transparency and animation.
    pub original: String,
}

/// One page of results.
#[derive(Clone, Debug, Default)]
pub struct Page {
    /// Unmodified provider ordering.
    pub stickers: Vec<Sticker>,
    /// Another page is available.
    pub more: bool,
}

#[derive(Deserialize)]
struct Response {
    data: Vec<Item>,
    pagination: Pagination,
}
#[derive(Deserialize)]
struct Pagination {
    total_count: usize,
    count: usize,
    offset: usize,
}
#[derive(Deserialize)]
struct Item {
    id: String,
    #[serde(default)]
    title: String,
    #[serde(default)]
    username: String,
    url: String,
    images: Images,
}
#[derive(Deserialize)]
struct Images {
    original: Rendition,
    fixed_width_still: Rendition,
}
#[derive(Deserialize)]
struct Rendition {
    url: String,
}

/// A provider-owned HTTPS URL, without credentials or a custom port.
pub fn is_provider_url(value: &str) -> bool {
    Url::parse(value).is_ok_and(|url| {
        url.scheme() == "https"
            && url.username().is_empty()
            && url.password().is_none()
            && url.port().is_none()
            && url
                .host_str()
                .is_some_and(|host| host == "giphy.com" || host.ends_with(".giphy.com"))
    })
}

fn valid_id(value: &str) -> bool {
    !value.is_empty() && value.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
}

fn parse(bytes: &[u8]) -> Result<Page, String> {
    let response: Response = serde_json::from_slice(bytes)
        .map_err(|_| "GIPHY returned an unreadable response".to_owned())?;
    let more = response
        .pagination
        .offset
        .saturating_add(response.pagination.count)
        < response.pagination.total_count;
    let stickers = response
        .data
        .into_iter()
        .filter(|item| {
            valid_id(&item.id)
                && is_provider_url(&item.url)
                && is_provider_url(&item.images.original.url)
                && is_provider_url(&item.images.fixed_width_still.url)
        })
        .map(|item| Sticker {
            title: if item.title.is_empty() {
                "GIPHY sticker".into()
            } else {
                item.title
            },
            id: item.id,
            author: item.username,
            source: item.url,
            thumbnail: item.images.fixed_width_still.url,
            original: item.images.original.url,
        })
        .collect();
    Ok(Page { stickers, more })
}

fn agent() -> ureq::Agent {
    crate::net::agent_builder(Duration::from_secs(15), Duration::from_secs(30))
        .redirects(0)
        .build()
}

/// Search, or load trending stickers when the query is empty. No background
/// startup requests; the caller submits an explicit user action.
pub fn search(key: &str, query: &str, page: usize) -> Result<Page, String> {
    if key.trim().is_empty() {
        return Err("Configure a GIPHY API key first".into());
    }
    let query: String = query.chars().take(50).collect();
    let endpoint = if query.is_empty() {
        "trending"
    } else {
        "search"
    };
    let mut url =
        Url::parse(&format!("https://api.giphy.com/v1/stickers/{endpoint}")).expect("constant URL");
    url.query_pairs_mut()
        .append_pair("api_key", key)
        .append_pair("limit", "24")
        .append_pair("offset", &page.saturating_mul(24).to_string())
        .append_pair("rating", "g");
    if !query.is_empty() {
        url.query_pairs_mut().append_pair("q", &query);
    }
    let response = agent()
        .get(url.as_str())
        .call()
        .map_err(|error| match error {
            ureq::Error::Status(401 | 403, _) => "GIPHY did not accept this API key".to_owned(),
            ureq::Error::Status(429, _) => {
                "GIPHY's request limit was reached; try again later".to_owned()
            }
            _ => "Could not reach GIPHY; check the connection and try again".to_owned(),
        })?;
    let mut bytes = Vec::new();
    response
        .into_reader()
        .take(4 * 1024 * 1024 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| "GIPHY's response was interrupted")?;
    if bytes.len() > 4 * 1024 * 1024 {
        return Err("GIPHY's response was too large".into());
    }
    let mut result = parse(&bytes)?;
    let maximum = if query.is_empty() { 499 } else { 4999 };
    result.more &= page.saturating_add(1).saturating_mul(24) <= maximum;
    Ok(result)
}

/// Read a key from the environment or the user's private configuration file.
pub fn load_key(dirs: &AppDirs) -> String {
    std::env::var("CLOVERCUT_GIPHY_API_KEY")
        .ok()
        .filter(|key| !key.trim().is_empty())
        .or_else(|| std::fs::read_to_string(dirs.config.join("giphy-api-key")).ok())
        .unwrap_or_default()
        .trim()
        .to_owned()
}

/// Save a key without placing it in project files or application logs.
pub fn save_key(dirs: &AppDirs, key: &str) -> Result<(), String> {
    let key = key.trim();
    if key.len() > 256 || key.chars().any(|c| c.is_control()) {
        return Err("Invalid GIPHY API key".into());
    }
    std::fs::create_dir_all(&dirs.config).map_err(|_| "Could not save GIPHY settings")?;
    let path = dirs.config.join("giphy-api-key");
    let mut file = tempfile::NamedTempFile::new_in(&dirs.config)
        .map_err(|_| "Could not save GIPHY settings")?;
    file.write_all(key.as_bytes())
        .map_err(|_| "Could not save GIPHY settings")?;
    file.persist(path)
        .map(|_| ())
        .map_err(|_| "Could not save GIPHY settings".into())
}

/// A session-owned scratch folder, removed when its last user leaves.
#[derive(Debug)]
pub struct Scratch(tempfile::TempDir);
impl Scratch {
    /// Make a private session folder; previews are never a persistent catalogue.
    pub fn new() -> Result<Self, String> {
        tempfile::Builder::new()
            .prefix("clovercut-giphy-")
            .tempdir()
            .map(Self)
            .map_err(|_| "Could not prepare sticker previews".into())
    }
    /// Original file path inside this private session.
    fn original(&self, id: &str) -> PathBuf {
        self.0.path().join(format!("{id}.gif"))
    }
}

fn download(url: &str, path: &Path, maximum: u64) -> Result<(), String> {
    if !is_provider_url(url) {
        return Err("Invalid GIPHY media URL".into());
    }
    let response = agent()
        .get(url)
        .call()
        .map_err(|_| "Could not download this GIPHY sticker")?;
    let mut bytes = Vec::new();
    response
        .into_reader()
        .take(maximum + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| "The sticker download was interrupted")?;
    if bytes.is_empty() || bytes.len() as u64 > maximum {
        return Err("The sticker is empty or too large".into());
    }
    std::fs::write(path, bytes).map_err(|_| "Could not write the sticker".into())
}

/// Decode a small still thumbnail on a worker, keeping alpha intact.
pub fn thumbnail(sticker: &Sticker) -> Result<concat_core::frame::Frame, String> {
    let file = tempfile::Builder::new()
        .suffix(".gif")
        .tempfile()
        .map_err(|_| "Could not prepare a thumbnail")?;
    download(&sticker.thumbnail, file.path(), 2 * 1024 * 1024)?;
    crate::media::still_at(&file.path().to_string_lossy(), 0.0, 160)
}

/// Download a selected animated original into session-owned storage and verify
/// that the editor can decode it before offering it to the timeline.
pub fn original(sticker: &Sticker, scratch: &Scratch) -> Result<PathBuf, String> {
    if !valid_id(&sticker.id) {
        return Err("Invalid sticker identifier".into());
    }
    let path = scratch.original(&sticker.id);
    if !path.is_file() {
        download(&sticker.original, &path, 64 * 1024 * 1024)?;
    }
    let summary = crate::media::probe(&path.to_string_lossy())?;
    if summary.video.is_none() {
        return Err("This sticker does not contain a picture".into());
    }
    Ok(path)
}

/// Copy one chosen original into the project, probe it off the UI thread and
/// preserve its source attribution beside it. Existing project assets stay intact.
pub fn adopt(
    sticker: &Sticker,
    source: &Path,
    project: &Path,
) -> Result<crate::media::MediaSummary, String> {
    use sha2::{Digest, Sha256};
    if !valid_id(&sticker.id) {
        return Err("Invalid sticker identifier".into());
    }
    let bytes = std::fs::read(source).map_err(|_| "Could not read the sticker")?;
    let digest = format!("{:x}", Sha256::digest(&bytes));
    let folder = project.join("stickers");
    std::fs::create_dir_all(&folder)
        .map_err(|_| "Could not create the project's stickers folder")?;
    let target = folder.join(format!("giphy-{}-{}.gif", sticker.id, &digest[..12]));
    if !target.exists() {
        let partial = target.with_extension("partial");
        std::fs::write(&partial, bytes)
            .and_then(|()| std::fs::rename(partial, &target))
            .map_err(|_| "Could not copy the sticker into the project")?;
    }
    let metadata = serde_json::json!({"provider":"GIPHY","id":sticker.id,"source":sticker.source,"author":sticker.author});
    std::fs::write(
        target.with_extension("source.json"),
        serde_json::to_vec_pretty(&metadata)
            .map_err(|_| "Could not prepare sticker attribution")?,
    )
    .map_err(|_| "Could not save sticker attribution")?;
    crate::media::probe(&target.to_string_lossy())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn result_order_transparency_rendition_and_url_parameters_are_preserved() {
        let entry = |id: &str| serde_json::json!({"id":id,"title":"A sticker","username":"artist","url":format!("https://giphy.com/stickers/{id}"),"images":{"original":{"url":format!("https://media.giphy.com/media/{id}/giphy.gif?cid=keep&rid=giphy.gif")},"fixed_width_still":{"url":"https://media.giphy.com/media/a/200w_s.gif"}}});
        let response = serde_json::json!({"data":[entry("second"),entry("first")],"pagination":{"offset":0,"count":2,"total_count":3}});
        let page = parse(&serde_json::to_vec(&response).unwrap()).unwrap();
        assert_eq!(
            page.stickers
                .iter()
                .map(|s| s.id.as_str())
                .collect::<Vec<_>>(),
            ["second", "first"]
        );
        assert!(
            page.stickers[0]
                .original
                .ends_with("?cid=keep&rid=giphy.gif")
        );
        assert!(page.more);
        assert!(parse(br#"{"meta":{"status":403}}"#).is_err());
        for bad in [
            "https://giphy.com.evil.org/a.gif",
            "http://media.giphy.com/a.gif",
            "https://me@media.giphy.com/a.gif",
        ] {
            assert!(!is_provider_url(bad));
        }
    }
    #[test]
    fn animated_gif_keeps_alpha_and_a_project_owns_its_copy() {
        let root = tempfile::tempdir().unwrap();
        let source = root.path().join("animation.gif");
        let mut bytes = b"GIF89a\x01\x00\x01\x00\x80\x00\x00\x00\x00\x00\xff\x00\x00".to_vec();
        for pixel in [0x44, 0x4c] {
            bytes.extend_from_slice(&[
                0x21, 0xf9, 4, 1, 10, 0, 0, 0, 0x2c, 0, 0, 0, 0, 1, 0, 1, 0, 0, 2, 2, pixel, 1, 0,
            ]);
        }
        bytes.push(0x3b);
        std::fs::write(&source, bytes).unwrap();
        let sticker = Sticker {
            id: "testgif".into(),
            title: "Test animation".into(),
            author: "Test artist".into(),
            source: "https://giphy.com/stickers/testgif".into(),
            thumbnail: String::new(),
            original: String::new(),
        };
        let project = root.path().join("project");
        let summary = adopt(&sticker, &source, &project).unwrap();
        assert_eq!(summary.kind, concat_project::model::MediaKind::Video);
        assert!(summary.duration.unwrap() > 0.1);
        let first = crate::media::still_at(&summary.path, 0.0, 1).unwrap();
        assert_eq!(
            first.pixels()[3],
            0,
            "transparent background survives decoding"
        );
        std::fs::remove_file(source).unwrap();
        assert!(std::path::Path::new(&summary.path).is_file());
        let attribution =
            std::fs::read_to_string(Path::new(&summary.path).with_extension("source.json"))
                .unwrap();
        assert!(attribution.contains("Test artist"));
        assert!(crate::media::probe(&summary.path).is_ok());
    }

    #[test]
    fn credentials_are_local_and_private() {
        let root = tempfile::tempdir().unwrap();
        let dirs = AppDirs::under(root.path());
        save_key(&dirs, "  test-key  ").unwrap();
        assert_eq!(
            std::fs::read_to_string(dirs.config.join("giphy-api-key")).unwrap(),
            "test-key"
        );
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                std::fs::metadata(dirs.config.join("giphy-api-key"))
                    .unwrap()
                    .permissions()
                    .mode()
                    & 0o777,
                0o600
            );
        }
        assert!(save_key(&dirs, "bad\nkey").is_err());
    }
}
