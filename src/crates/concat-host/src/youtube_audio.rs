// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Clovercut contributors

//! Current Studio music and sound effects through the Clovercut Audio API.
//! Only stable API routes enter the cache; expiring Google URLs are resolved
//! immediately before a download. Legacy Drive downloads remain readable.

use crate::dirs::AppDirs;
use crate::sounds::{Catalogue, Credits, Listed};
use serde::{Deserialize, Serialize};
use std::io::Read;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use url::Url;

/// Default endpoint of the local Clovercut Audio API.
pub const DEFAULT_API: &str = "http://127.0.0.1:8917";
/// Official source shared by the Studio catalogue.
pub const SOURCE: &str = "https://www.youtube.com/audiolibrary";
/// Diagnostic code translated by the desktop client.
pub const API_UNAVAILABLE: &str = "youtube-api-unavailable";
/// Diagnostic code for a missing or expired operator Studio session.
pub const STUDIO_UNAVAILABLE: &str = "youtube-studio-unavailable";

/// Normalize a service address; insecure HTTP is accepted only on loopback.
pub fn api_base(value: &str) -> Result<Url, String> {
    let value = if value.trim().is_empty() {
        DEFAULT_API
    } else {
        value.trim()
    };
    let mut url = Url::parse(value).map_err(|_| "invalid YouTube API address")?;
    let local = matches!(url.host_str(), Some("127.0.0.1" | "localhost" | "[::1]"));
    if !(url.scheme() == "https" || url.scheme() == "http" && local)
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return Err("invalid YouTube API address".into());
    }
    let path = format!("{}/", url.path().trim_end_matches('/'));
    url.set_path(&path);
    Ok(url)
}

/// Open the service's dedicated persistent Studio profile, never the user's
/// unrelated browser. A local service is required to own that session.
pub fn open_studio_login(dirs: &AppDirs, api: &str) -> Result<(), String> {
    crate::youtube_service::ensure(dirs, api)?;
    let base = api_base(api)?;
    if !matches!(base.host_str(), Some("127.0.0.1" | "localhost" | "[::1]")) {
        return Err(API_UNAVAILABLE.into());
    }
    let url = base.join("v1/session/login").map_err(|_| API_UNAVAILABLE)?;
    crate::net::agent(Duration::from_secs(5), Duration::from_secs(30))
        .post(url.as_str())
        .set("X-Clovercut-Client", "desktop")
        .call()
        .map_err(|_| STUDIO_UNAVAILABLE.to_string())?;
    Ok(())
}

fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 128
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_'))
}

/// Check the stable, credential-free API route belongs to this exact track.
pub fn is_track_endpoint(value: &str, id: &str) -> bool {
    if !valid_id(id) {
        return false;
    }
    let Ok(url) = Url::parse(value) else {
        return false;
    };
    api_base(value).is_ok() && url.path().ends_with(&format!("/v1/tracks/{id}/audio"))
}

/// Compatibility for originals downloaded from the previous community index.
pub fn is_audio(value: &str) -> bool {
    Url::parse(value).is_ok_and(|url| {
        matches!(
            url.host_str(),
            Some("drive.usercontent.google.com" | "docs.google.com")
        ) && url.scheme() == "https"
            && url.username().is_empty()
            && url.password().is_none()
    })
}

fn valid_stream(value: &str, now: u64) -> bool {
    Url::parse(value).is_ok_and(|url| {
        let host = url.host_str().unwrap_or_default();
        url.scheme() == "https"
            && url.username().is_empty()
            && url.password().is_none()
            && url.port().is_none()
            && (host == "googlevideo.com" || host.ends_with(".googlevideo.com"))
            && url.path() == "/videoplayback"
            && url
                .query_pairs()
                .find(|(key, _)| key == "expire")
                .and_then(|(_, expiry)| expiry.parse::<u64>().ok())
                .is_some_and(|expiry| expiry > now.saturating_add(60))
    })
}

/// Obtain and validate a fresh Google audio URL without caching its signature.
pub fn resolve_audio(sound: &Listed) -> Result<String, String> {
    let id = sound
        .id
        .strip_prefix("ytcc-")
        .ok_or("invalid YouTube audio id")?;
    if !is_track_endpoint(&sound.download_url, id) {
        return Err("invalid YouTube audio route".into());
    }
    let agent = crate::net::agent_builder(Duration::from_secs(5), Duration::from_secs(30))
        .redirects(0)
        .build();
    let response = match agent.get(&sound.download_url).call() {
        Ok(r) => r,
        Err(ureq::Error::Status(503, _)) => return Err(STUDIO_UNAVAILABLE.into()),
        Err(_) => return Err(API_UNAVAILABLE.into()),
    };
    if !matches!(response.status(), 302 | 303 | 307 | 308) {
        return Err(API_UNAVAILABLE.into());
    }
    let stream = response.header("Location").ok_or(API_UNAVAILABLE)?;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_secs();
    if !valid_stream(stream, now) {
        return Err("invalid or expired YouTube audio URL".into());
    }
    Ok(stream.to_owned())
}

#[derive(Deserialize, Serialize)]
struct Page {
    total: usize,
    offset: usize,
    items: Vec<Track>,
    #[serde(rename = "nextOffset")]
    next_offset: Option<usize>,
}
#[derive(Deserialize, Serialize)]
struct Track {
    id: String,
    name: String,
    #[serde(rename = "type")]
    kind: String,
    author: Option<String>,
    duration: f64,
    license: License,
}
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct License {
    code: Option<String>,
    requires_attribution: Option<bool>,
    attribution: Option<String>,
    url: Option<String>,
    artist_url: Option<String>,
    track_url: Option<String>,
}

fn track_to_listed(track: Track, base: &Url, category: usize) -> Option<Listed> {
    if !valid_id(&track.id)
        || track.name.trim().is_empty()
        || !track.duration.is_finite()
        || track.duration < 0.0
        || track.kind
            != if category == 6 {
                "sound-effect"
            } else {
                "music"
            }
    {
        return None;
    }
    let code = track.license.code.as_deref().unwrap_or("Unknown");
    let license = match code {
        "CREATOR_MUSIC_LICENSE_TYPE_YOUTUBE" => "YouTube Audio Library",
        "CREATOR_MUSIC_LICENSE_TYPE_CCBY_4" => "CC BY 4.0",
        _ => code,
    }
    .to_owned();
    let required = match code {
        "CREATOR_MUSIC_LICENSE_TYPE_CCBY_4" => Some(true),
        "CREATOR_MUSIC_LICENSE_TYPE_YOUTUBE" => Some(false),
        _ => track.license.requires_attribution,
    };
    let author = track.author.unwrap_or_default();
    let credits = track
        .license
        .attribution
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| {
            let mut lines = vec![track.name.clone()];
            if !author.is_empty() {
                lines.push(author.clone());
            }
            lines.push(license.clone());
            for link in [
                track.license.url,
                track.license.track_url,
                track.license.artist_url,
            ] {
                if let Some(link) = link
                    && Url::parse(&link).is_ok_and(|u| {
                        matches!(u.scheme(), "https" | "http")
                            && u.username().is_empty()
                            && u.password().is_none()
                    })
                {
                    lines.push(link);
                }
            }
            lines.push(SOURCE.into());
            lines.join("\n")
        });
    Some(Listed {
        file: format!("{}.mp3", track.id),
        download_url: base
            .join(&format!("v1/tracks/{}/audio", track.id))
            .ok()?
            .into(),
        id: format!("ytcc-{}", track.id),
        name: track.name,
        category: if category == 6 { "Effects" } else { "Music" }.into(),
        duration: track.duration,
        author,
        license,
        source: SOURCE.into(),
        credits: Credits {
            text: credits,
            required,
        },
        ..Default::default()
    })
}

/// Search one current music or sound-effect page through the configured service.
pub fn search(
    dirs: &AppDirs,
    query: &str,
    category: usize,
    page: usize,
    api: &str,
) -> Result<Catalogue, String> {
    let base = api_base(api)?;
    let category = if category == 6 { 6 } else { 5 };
    let offset = page.checked_mul(24).ok_or("invalid audio page")?;
    let mut url = base.join("v1/tracks").map_err(|e| e.to_string())?;
    url.query_pairs_mut()
        .append_pair(
            "type",
            if category == 6 {
                "sound-effect"
            } else {
                "music"
            },
        )
        .append_pair("q", query.trim())
        .append_pair("offset", &offset.to_string())
        .append_pair("limit", "24");
    use sha2::{Digest, Sha256};
    let folder = crate::sounds::cache_dir(dirs);
    let cache = folder.join(format!(
        "youtube-studio-{:x}.json",
        Sha256::digest(url.as_str().as_bytes())
    ));
    let service = crate::youtube_service::ensure(dirs, api).and_then(|started| {
        if started {
            open_studio_login(dirs, api)?;
        }
        Ok(())
    });
    let fetched = match service.and_then(|_| {
        crate::net::agent(Duration::from_secs(5), Duration::from_secs(20))
            .get(url.as_str())
            .call()
            .map_err(|error| match error {
                ureq::Error::Status(503, _) => STUDIO_UNAVAILABLE.to_owned(),
                _ => API_UNAVAILABLE.to_owned(),
            })
    }) {
        Ok(response) => read_page(response.into_reader(), offset),
        Err(error) => Err(error),
    };
    let parsed = match fetched {
        Ok(parsed) => {
            // Persist only the recognized metadata fields, never an arbitrary
            // server response or expiring stream URL. Each writer owns its temp file.
            if std::fs::create_dir_all(&folder).is_ok()
                && let Ok(mut temporary) = tempfile::NamedTempFile::new_in(&folder)
                && serde_json::to_writer(temporary.as_file_mut(), &parsed).is_ok()
            {
                let _ = temporary.persist(&cache);
            }
            parsed
        }
        Err(error) => std::fs::File::open(&cache)
            .ok()
            .and_then(|file| read_page(file, offset).ok())
            .ok_or(error)?,
    };
    Ok(Catalogue {
        version: 2,
        query: query.to_owned(),
        category,
        page,
        more: parsed
            .next_offset
            .is_some_and(|next| next > offset && next < parsed.total),
        sounds: parsed
            .items
            .into_iter()
            .filter_map(|t| track_to_listed(t, &base, category))
            .collect(),
    })
}

fn read_page(reader: impl Read, offset: usize) -> Result<Page, String> {
    const MAX_BYTES: u64 = 2 * 1024 * 1024;
    let mut body = Vec::new();
    reader
        .take(MAX_BYTES + 1)
        .read_to_end(&mut body)
        .map_err(|_| API_UNAVAILABLE)?;
    if body.len() as u64 > MAX_BYTES {
        return Err("YouTube API response too large".into());
    }
    let parsed: Page = serde_json::from_slice(&body).map_err(|_| "invalid YouTube API response")?;
    if parsed.offset != offset || parsed.items.len() > 24 {
        return Err("invalid YouTube API page".into());
    }
    Ok(parsed)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn endpoints_and_signed_streams_do_not_accept_lookalikes_or_expired_links() {
        assert!(api_base("").is_ok());
        assert!(api_base("https://audio.example/api").is_ok());
        for value in [
            "http://audio.example",
            "https://user:secret@example.com",
            "http://127.0.0.1.evil.test:8917",
            "file:///tmp/catalogue",
            "http://127.0.0.1:8917?q=x",
        ] {
            assert!(api_base(value).is_err(), "{value}");
        }
        assert!(is_track_endpoint(
            "http://127.0.0.1:8917/v1/tracks/a_B-1/audio",
            "a_B-1"
        ));
        assert!(!is_track_endpoint(
            "http://127.0.0.1:8917/v1/tracks/other/audio",
            "a_B-1"
        ));
        assert!(valid_stream(
            "https://redirector.googlevideo.com/videoplayback?expire=1000",
            1
        ));
        for value in [
            "https://googlevideo.com.evil.test/videoplayback?expire=1000",
            "http://googlevideo.com/videoplayback?expire=1000",
            "https://googlevideo.com/videoplayback?expire=1",
            "https://googlevideo.com/other?expire=1000",
        ] {
            assert!(!valid_stream(value, 1));
        }
    }
    #[test]
    fn current_tracks_keep_type_duration_author_and_required_credit() {
        let t: Track = serde_json::from_value(serde_json::json!({"id":"a_B-1","name":"Rain","type":"sound-effect","author":"Artist","duration":3.25,"license":{"code":"CREATOR_MUSIC_LICENSE_TYPE_CCBY_4","requiresAttribution":false,"attribution":null,"url":"https://creativecommons.org/licenses/by/4.0/","artistUrl":null,"trackUrl":null}})).unwrap();
        let sound = track_to_listed(t, &api_base("").unwrap(), 6).unwrap();
        assert_eq!(sound.category, "Effects");
        assert_eq!(sound.duration, 3.25);
        assert_eq!(sound.author, "Artist");
        assert_eq!(sound.credits.required, Some(true));
        assert!(sound.credits.text.contains("CC BY 4.0"));
        assert!(sound.is_offered());
        assert!(!sound.credits.text.contains("requiresAttribution"));
    }
    fn serve(
        status: u16,
        body: String,
        location: Option<String>,
    ) -> (String, std::thread::JoinHandle<String>) {
        use std::io::{Read, Write};
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let thread = std::thread::spawn(move || {
            let (mut connection, _) = listener.accept().unwrap();
            connection
                .set_read_timeout(Some(Duration::from_secs(5)))
                .unwrap();
            let mut bytes = Vec::new();
            let mut buffer = [0; 1024];
            while !bytes.windows(4).any(|w| w == b"\r\n\r\n") {
                let read = connection.read(&mut buffer).unwrap();
                if read == 0 {
                    break;
                }
                bytes.extend_from_slice(&buffer[..read]);
            }
            let location = location
                .map(|l| format!("Location: {l}\r\n"))
                .unwrap_or_default();
            write!(connection, "HTTP/1.1 {status} Test\r\nContent-Type: application/json\r\n{location}Content-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len()).unwrap();
            String::from_utf8(bytes).unwrap()
        });
        (base, thread)
    }
    fn effect() -> serde_json::Value {
        serde_json::json!({"id":"fixture_1","name":"Rain","type":"sound-effect","author":"Artist","duration":3.25,"license":{"code":"CREATOR_MUSIC_LICENSE_TYPE_CCBY_4","requiresAttribution":true,"attribution":"Original credit","url":null,"artistUrl":null,"trackUrl":null}})
    }
    #[test]
    fn search_encodes_text_and_keeps_provider_pagination() {
        let body = serde_json::json!({"total":100,"offset":24,"nextOffset":48,"items":[effect()]})
            .to_string();
        let (base, task) = serve(200, body, None);
        let root = tempfile::tempdir().unwrap();
        let dirs = AppDirs {
            config: root.path().into(),
            data: root.path().into(),
        };
        let result = search(&dirs, "rain & wind", 6, 1, &base).unwrap();
        let request = task.join().unwrap();
        assert!(request.contains("type=sound-effect"));
        assert!(request.contains("q=rain+%26+wind"));
        assert!(request.contains("offset=24"));
        assert_eq!(result.page, 1);
        assert!(result.more);
        assert_eq!(result.sounds[0].credits.text, "Original credit");
        let offline = search(&dirs, "rain & wind", 6, 1, &base).unwrap();
        assert_eq!(offline, result);
        assert!(result.sounds[0].is_offered());
        assert_eq!(
            result.sounds[0].download_url,
            format!("{base}/v1/tracks/fixture_1/audio")
        );
    }
    #[test]
    fn resolver_requires_fresh_google_audio_and_reports_unavailable_studio() {
        let expiry = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + 3600;
        for (status, location, expected) in [
            (
                307,
                Some(format!(
                    "https://redirector.googlevideo.com/videoplayback?expire={expiry}"
                )),
                "ok",
            ),
            (
                307,
                Some("https://googlevideo.com.evil.test/videoplayback?expire=9999999999".into()),
                "invalid",
            ),
            (503, None, STUDIO_UNAVAILABLE),
        ] {
            let (base, task) = serve(status, "{}".into(), location);
            let track = serde_json::from_value(effect()).unwrap();
            let sound = track_to_listed(track, &api_base(&base).unwrap(), 6).unwrap();
            let result = resolve_audio(&sound);
            task.join().unwrap();
            if expected == "ok" {
                assert!(result.is_ok());
            } else if expected == "invalid" {
                assert!(result.unwrap_err().contains("invalid"));
            } else {
                assert_eq!(result.unwrap_err(), expected);
            }
        }
    }
    #[test]
    fn cached_audio_and_credits_survive_api_address_changes_without_studio() {
        use sha2::{Digest, Sha256};
        let root = tempfile::tempdir().unwrap();
        let dirs = AppDirs {
            config: root.path().into(),
            data: root.path().into(),
        };
        let mut old = track_to_listed(
            serde_json::from_value(effect()).unwrap(),
            &api_base("http://127.0.0.1:1").unwrap(),
            6,
        )
        .unwrap();
        let payload = b"previously validated cached audio";
        old.sha256 = format!("{:x}", Sha256::digest(payload));
        let folder = crate::sounds::cache_dir(&dirs);
        std::fs::create_dir_all(&folder).unwrap();
        let file = crate::sounds::cached_file(&dirs, &old);
        std::fs::write(&file, payload).unwrap();
        std::fs::write(
            folder.join(format!("{}.json", old.id)),
            serde_json::to_vec(&old).unwrap(),
        )
        .unwrap();
        let mut current = old.clone();
        current.sha256.clear();
        current.download_url = "https://audio.example/v1/tracks/fixture_1/audio".into();
        current.credits.text = "Updated credit".into();
        let restored = crate::sounds::resolved_sound(&dirs, &current).unwrap();
        assert_eq!(restored.download_url, current.download_url);
        assert_eq!(restored.credits.text, "Updated credit");
        assert_eq!(
            crate::sounds::fetch_sound(
                &dirs,
                &current,
                &std::sync::atomic::AtomicBool::new(false),
                &mut |_, _| panic!("cached audio should not download")
            )
            .unwrap(),
            file
        );
    }
    #[test]
    #[ignore = "requires the local Audio API and an authenticated Studio bridge"]
    fn live_api_search_preview_and_cache_both_types() {
        let root = tempfile::tempdir().unwrap();
        let dirs = AppDirs {
            config: root.path().into(),
            data: root.path().into(),
        };
        for (category, query) in [(5, "Mess Call"), (5, "Country Music"), (6, "Straw Squeak")] {
            let page = search(&dirs, query, category, 0, DEFAULT_API).unwrap();
            let sound = page.sounds.first().expect("live track exists");
            assert!(sound.duration > 0.0);
            assert!(!sound.credits.text.is_empty());
            let file = crate::sounds::fetch_sound(
                &dirs,
                sound,
                &std::sync::atomic::AtomicBool::new(false),
                &mut |_, _| {},
            )
            .unwrap();
            let cached = crate::sounds::resolved_sound(&dirs, sound).unwrap();
            assert_eq!(cached.sha256.len(), 64);
            assert!(file.is_file());
            assert_eq!(
                crate::sounds::fetch_sound(
                    &dirs,
                    sound,
                    &std::sync::atomic::AtomicBool::new(false),
                    &mut |_, _| panic!("cached second request should not download")
                )
                .unwrap(),
                file
            );
        }
    }
}
