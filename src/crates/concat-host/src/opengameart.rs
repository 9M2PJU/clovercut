// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Clovercut contributors

//! Direct access to OpenGameArt's public CC0 audio search. This adapter is
//! intentionally separate: OpenGameArt serves HTML rather than a documented API.
//! Search results never authorize a download; the asset page's own licence and
//! original file links are checked before fetching any audio.

use std::collections::HashSet;
use std::io::Read;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use scraper::{Html, Selector};
use sha2::{Digest, Sha256};
use url::Url;

use crate::sounds::{Catalogue, Listed};

const ORIGIN: &str = "https://opengameart.org";
static LAST_REQUEST: Mutex<Option<Instant>> = Mutex::new(None);

fn selector(css: &str) -> Selector {
    Selector::parse(css).expect("constant OpenGameArt selector")
}

fn text(element: scraper::ElementRef<'_>) -> String {
    element
        .text()
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// A first-party HTTPS source page, with no credentials or nonstandard port.
pub fn is_source(value: &str) -> bool {
    Url::parse(value).is_ok_and(|url| {
        trusted(&url) && url.path().starts_with("/content/") && url.path().len() > 9
    })
}

fn trusted(url: &Url) -> bool {
    url.scheme() == "https"
        && url.host_str() == Some("opengameart.org")
        && url.username().is_empty()
        && url.password().is_none()
        && url.port().is_none()
}

/// Only original audio attachments on the source host, never preview montages,
/// archives, external redirects or HTML masquerading as an audio link.
pub fn is_audio(value: &str) -> bool {
    Url::parse(value).is_ok_and(|url| {
        trusted(&url)
            && url.path().starts_with("/sites/default/files/")
            && !url.path().contains("/audio_preview/")
            && crate::sounds::is_sound(std::path::Path::new(url.path()))
    })
}

fn absolute(value: &str) -> Option<String> {
    let url = Url::parse(ORIGIN).ok()?.join(value).ok()?;
    trusted(&url).then(|| url.to_string())
}

fn request(url: &str) -> Result<String, String> {
    // The public site's robots.txt requests ten seconds between page requests.
    // Only this worker waits; browsing and timeline editing remain responsive.
    let mut last = LAST_REQUEST
        .lock()
        .map_err(|_| "audio request lock failed")?;
    if let Some(at) = *last {
        std::thread::sleep(Duration::from_secs(10).saturating_sub(at.elapsed()));
    }
    *last = Some(Instant::now());
    let response = ureq::AgentBuilder::new()
        .user_agent("Clovercut/0.0.1 (+https://github.com/tulcoturo/clovercut)")
        .timeout_connect(Duration::from_secs(15))
        .timeout_read(Duration::from_secs(30))
        .build()
        .get(url)
        .call()
        .map_err(|error| format!("OpenGameArt did not answer: {error}"))?;
    let mut bytes = Vec::new();
    response
        .into_reader()
        .take(4 * 1024 * 1024 + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| format!("OpenGameArt page did not arrive: {error}"))?;
    if bytes.len() > 4 * 1024 * 1024 {
        return Err("OpenGameArt page is too large".into());
    }
    String::from_utf8(bytes).map_err(|_| "OpenGameArt returned an unreadable page".into())
}

/// Search the site's public Music/Sound Effect pages with its CC0 filter.
/// Categories 1..4 add the effect's keyword, 5 is music, 6 is all effects.
pub fn search(query: &str, category: usize, page: usize) -> Result<Catalogue, String> {
    let mut url = Url::parse(&format!("{ORIGIN}/art-search-advanced")).expect("constant URL");
    let extra = match category {
        1 => "whoosh",
        2 => "impact",
        3 => "riser",
        4 => "ambience",
        _ => "",
    };
    let query: String = query.chars().take(160).collect();
    let keywords = format!("{extra} {query}").trim().to_owned();
    {
        let mut params = url.query_pairs_mut();
        if category == 0 || category == 5 {
            params.append_pair("field_art_type_tid[0]", "12");
        }
        if category != 5 {
            params.append_pair("field_art_type_tid[1]", "13");
        }
        params
            .append_pair("field_art_licenses_tid[0]", "4")
            .append_pair("keys", &keywords)
            .append_pair("items_per_page", "24")
            .append_pair("sort_by", "count")
            .append_pair("sort_order", "DESC")
            .append_pair("page", &page.to_string());
    }
    let html = request(url.as_str())?;
    let mut catalogue = parse_search(&html, category)?;
    catalogue.query = query;
    catalogue.category = category;
    catalogue.page = page;
    Ok(catalogue)
}

fn parse_search(html: &str, category: usize) -> Result<Catalogue, String> {
    let document = Html::parse_document(html);
    if document.select(&selector(".view-id-art")).next().is_none()
        || document
            .select(&selector(".messages.error"))
            .next()
            .is_some()
    {
        return Err("OpenGameArt's search page changed or rejected the search; try again or open the source website".into());
    }
    let mut sounds = Vec::new();
    let mut seen = HashSet::new();
    for row in document.select(&selector(".view-id-art .art-previews-inline")) {
        // The type filter is not enough: a page without an audio preview is not
        // an audio asset, and collections themselves do not confer a licence.
        if row.select(&selector(".audio-preview")).next().is_none() {
            continue;
        }
        let Some(link) = row.select(&selector(".field-name-title a")).next() else {
            continue;
        };
        let Some(source) = link
            .value()
            .attr("href")
            .and_then(absolute)
            .filter(|url| is_source(url))
        else {
            continue;
        };
        if !seen.insert(source.clone()) {
            continue;
        }
        let id = format!("oga-{:x}", Sha256::digest(source.as_bytes()));
        let name = text(link);
        if name.is_empty() {
            continue;
        }
        sounds.push(Listed {
            file: format!("{id}.pending"),
            id,
            name,
            source,
            category: match category {
                1 => "Whoosh",
                2 => "Impact",
                3 => "Riser",
                4 => "Ambience",
                5 => "Music",
                6 => "Effects",
                _ => "Audio",
            }
            .into(),
            license: "CC0-1.0".into(),
            ..Default::default()
        });
    }
    Ok(Catalogue {
        version: 2,
        sounds,
        more: document
            .select(&selector(".view-id-art .pager-next a"))
            .next()
            .is_some(),
        ..Default::default()
    })
}

/// Resolve one result to an original downloadable track after confirming CC0
/// on that asset's own page. Archive-only packs are not used as preview montages.
pub fn resolve(sound: &Listed) -> Result<Listed, String> {
    if !is_source(&sound.source) {
        return Err("invalid OpenGameArt source page".into());
    }
    parse_asset(&request(&sound.source)?, sound)
}

fn parse_asset(html: &str, sound: &Listed) -> Result<Listed, String> {
    let document = Html::parse_document(html);
    let cc0 = document
        .select(&selector(".field-name-field-art-licenses a"))
        .filter_map(|a| a.value().attr("href"))
        .any(|href| {
            Url::parse(href).is_ok_and(|url| {
                matches!(url.scheme(), "http" | "https")
                    && url.host_str() == Some("creativecommons.org")
                    && url.path().trim_end_matches('/') == "/publicdomain/zero/1.0"
            })
        });
    if !cc0 {
        return Err(
            "This entry's own page does not confirm CC0; its audio was not downloaded".into(),
        );
    }
    let audio: Vec<_> = document
        .select(&selector(".field-name-field-art-files a"))
        .filter_map(|link| {
            link.value()
                .attr("href")
                .and_then(absolute)
                .filter(|url| is_audio(url))
                .map(|url| (link, url))
        })
        .collect();
    let Some((link, url)) = audio.first() else {
        return Err("This entry has no individual audio attachment. Use its source page for archive-only packs".into());
    };
    let mut resolved = sound.clone();
    resolved.download_url = url.clone();
    let parsed = Url::parse(url).expect("validated audio URL");
    let extension = std::path::Path::new(parsed.path())
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("ogg");
    resolved.file = format!("{}.{}", sound.id, extension);
    resolved.author = document
        .select(&selector(".field-name-author-submitter .username"))
        .next()
        .map(text)
        .unwrap_or_default();
    resolved.bytes = link
        .value()
        .attr("type")
        .and_then(|value| value.split("length=").nth(1))
        .and_then(|value| value.trim().parse().ok())
        .unwrap_or(0);
    if audio.len() > 1 {
        resolved.name = format!("{} — {}", sound.name, text(*link));
    }
    Ok(resolved)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_uses_only_real_audio_results_and_decodes_entities() {
        let html = r#"<div class='view-id-art'><div class='art-previews-inline'><div class='field-name-title'><a href='/content/test-song'>Wind &amp; rain</a></div><div class='audio-preview'></div></div><div class='art-previews-inline'><div class='field-name-title'><a href='/content/not-a-sound'>Image</a></div></div><li class='pager-next'><a href='?page=1'>Next</a></li></div>"#;
        let result = parse_search(html, 5).unwrap();
        assert_eq!(result.sounds.len(), 1);
        assert_eq!(result.sounds[0].name, "Wind & rain");
        assert!(result.more);
        assert!(result.sounds[0].is_offered());
        assert!(parse_search("<h1>rate limited</h1>", 0).is_err());
    }

    #[test]
    fn asset_checks_its_own_license_and_uses_original_audio_only() {
        let sound = Listed {
            id: "oga-test".into(),
            source: "https://opengameart.org/content/test".into(),
            ..Default::default()
        };
        let html = r#"<div class='field-name-field-art-licenses'><a href='https://creativecommons.org/publicdomain/zero/1.0/'>CC0</a></div><div class='field-name-author-submitter'><span class='username'>An &amp; Author</span></div><div class='audio-preview'><a href='https://opengameart.org/sites/default/files/preview.mp3'>Montage</a></div><div class='field-name-field-art-files'><a href='https://opengameart.org/sites/default/files/track.mp3' type='audio/mpeg; length=123'>Track</a></div>"#;
        let resolved = parse_asset(html, &sound).unwrap();
        assert!(resolved.download_url.ends_with("/track.mp3"));
        assert_eq!(resolved.bytes, 123);
        assert_eq!(resolved.author, "An & Author");
        assert!(
            parse_asset(
                &html.replace("publicdomain/zero/1.0", "licenses/by/4.0"),
                &sound
            )
            .is_err()
        );
        assert!(parse_asset(&html.replace("track.mp3", "pack.zip"), &sound).is_err());
        for url in [
            "https://evil.org/sites/default/files/a.mp3",
            "https://opengameart.org.evil.org/sites/default/files/a.wav",
            "http://opengameart.org/sites/default/files/a.wav",
            "https://opengameart.org/sites/default/files/audio_preview/a.mp3",
            "https://user@opengameart.org/sites/default/files/a.mp3",
        ] {
            assert!(!is_audio(url));
        }
    }

    /// Explicit opt-in: exercises the real provider, download, media probe and cache.
    #[test]
    #[ignore = "requires network access to OpenGameArt"]
    fn live_search_download_and_offline_cache() {
        let root = std::env::temp_dir().join(format!("clovercut-oga-live-{}", std::process::id()));
        let dirs = crate::AppDirs::under(&root);
        let catalogue = crate::sounds::search_catalogue(&dirs, "", 5, 0)
            .unwrap()
            .unwrap();
        assert!(!catalogue.sounds.is_empty());
        assert!(catalogue.sounds.iter().all(Listed::is_offered));
        let source = "https://opengameart.org/content/with-me-0";
        let id = format!("oga-{:x}", Sha256::digest(source.as_bytes()));
        let sound = Listed {
            file: format!("{id}.pending"),
            id,
            source: source.into(),
            name: "With me".into(),
            license: "CC0-1.0".into(),
            ..Default::default()
        };
        let cancel = std::sync::atomic::AtomicBool::new(false);
        let path = crate::sounds::fetch_sound(&dirs, &sound, &cancel, &mut |_, _| {}).unwrap();
        let resolved = crate::sounds::resolved_sound(&dirs, &sound).unwrap();
        assert!(resolved.duration > 0.0);
        assert_eq!(resolved.sha256.len(), 64);
        assert!(is_audio(&resolved.download_url));
        assert!(path.is_file());
        assert_eq!(crate::sounds::cached_file(&dirs, &sound), path);
        assert_eq!(
            crate::sounds::fetch_sound(&dirs, &sound, &cancel, &mut |_, _| {}).unwrap(),
            path
        );
        assert!(crate::sounds::cached_catalogue(&dirs).is_some());
        std::fs::remove_dir_all(root).unwrap();
    }
}
