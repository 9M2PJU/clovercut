// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Clovercut contributors

//! Access to Freesound.org's audio search API.
//! Sound previews are streamed and downloaded directly from Freesound CDNs.

use std::collections::HashMap;
use std::time::Duration;

use serde::Deserialize;
use url::Url;

use crate::dirs::AppDirs;
use crate::sounds::{Catalogue, Listed};

const API_SEARCH: &str = "https://freesound.org/apiv2/search/text/";

#[derive(Deserialize, Clone, Debug)]
struct FreesoundSound {
    id: u64,
    name: String,
    #[serde(default)]
    username: String,
    #[serde(default)]
    license: String,
    #[serde(default)]
    duration: f64,
    #[serde(default)]
    previews: HashMap<String, String>,
}

#[derive(Deserialize, Clone, Debug)]
#[allow(dead_code)]
struct FreesoundResponse {
    #[serde(default)]
    count: usize,
    #[serde(default)]
    next: Option<String>,
    #[serde(default)]
    previous: Option<String>,
    #[serde(default)]
    results: Vec<FreesoundSound>,
}

/// Whether the given URL points to an audio resource hosted by Freesound.
pub fn is_audio(value: &str) -> bool {
    Url::parse(value).is_ok_and(|url| {
        url.host_str()
            .is_some_and(|h| h == "freesound.org" || h.ends_with(".freesound.org"))
            && url.scheme() == "https"
            && url.username().is_empty()
            && url.password().is_none()
    })
}

/// Search Freesound for sound effects using a user query and API key.
pub fn search(
    _dirs: &AppDirs,
    query: &str,
    category: usize,
    page: usize,
    api_key: &str,
) -> Result<Catalogue, String> {
    let key = api_key.trim();
    if key.is_empty() {
        return Err(
            "Freesound requires an API key. Please add your free key in Settings > Libraries."
                .into(),
        );
    }

    let q = if query.trim().is_empty() {
        "effect"
    } else {
        query.trim()
    };
    let page_1_indexed = page + 1;

    let mut url = Url::parse(API_SEARCH).map_err(|e| e.to_string())?;
    url.query_pairs_mut()
        .append_pair("query", q)
        .append_pair("page", &page_1_indexed.to_string())
        .append_pair("page_size", "24")
        .append_pair("fields", "id,name,previews,duration,username,license");

    let agent = crate::net::agent(Duration::from_secs(15), Duration::from_secs(30));
    let response = agent
        .get(url.as_str())
        .set("Authorization", &format!("Token {key}"))
        .set("User-Agent", "Mozilla/5.0 (compatible; Clovercut/0.1)")
        .call()
        .map_err(|e| match e {
            ureq::Error::Status(401 | 403, _) => {
                "Freesound rejected the API key. Check Settings > Libraries.".to_string()
            }
            ureq::Error::Status(429, _) => {
                "Freesound request limit reached. Please try again later.".to_string()
            }
            _ => format!("Freesound search failed: {e}"),
        })?;

    let parsed: FreesoundResponse = serde_json::from_reader(response.into_reader())
        .map_err(|e| format!("Could not read Freesound response: {e}"))?;

    let sounds = parsed
        .results
        .into_iter()
        .filter(|item| item.previews.values().any(|url| is_audio(url)))
        .map(|item| {
            let id = format!("fs-{}", item.id);
            let download_url = item
                .previews
                .get("preview-hq-mp3")
                .or_else(|| item.previews.get("preview-lq-mp3"))
                .cloned()
                .unwrap_or_default();

            Listed {
                file: format!("{}.mp3", item.id),
                id,
                name: item.name,
                category: "Effects".into(),
                duration: item.duration,
                bytes: 0,
                license: item.license,
                source: format!("https://freesound.org/s/{}/", item.id),
                author: item.username,
                sha256: String::new(),
                download_url,
                credits: Default::default(),
            }
        })
        .collect();

    Ok(Catalogue {
        version: 2,
        query: query.to_string(),
        category,
        page,
        more: parsed.next.is_some(),
        sounds,
    })
}
