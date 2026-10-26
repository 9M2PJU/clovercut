// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Clovercut contributors
//! On-demand local Studio connector. Its runtime and source are private app data;
//! the dedicated browser owns authentication, without exporting its cookies.

use crate::AppDirs;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Mutex;
use std::time::Duration;

static START: Mutex<()> = Mutex::new(());
const VERSION: &str = "v22.22.0";
const FILES: &[(&str, &str)] = &[
    (
        "bridge.mjs",
        include_str!("../audio-service/src/bridge.mjs"),
    ),
    (
        "browser-bridge.js",
        include_str!("../audio-service/src/browser-bridge.js"),
    ),
    (
        "catalogue.mjs",
        include_str!("../audio-service/src/catalogue.mjs"),
    ),
    ("cdp.mjs", include_str!("../audio-service/src/cdp.mjs")),
    (
        "connect.mjs",
        include_str!("../audio-service/src/connect.mjs"),
    ),
    ("login.mjs", include_str!("../audio-service/src/login.mjs")),
    (
        "server.mjs",
        include_str!("../audio-service/src/server.mjs"),
    ),
    (
        "session.mjs",
        include_str!("../audio-service/src/session.mjs"),
    ),
    (
        "studio.mjs",
        include_str!("../audio-service/src/studio.mjs"),
    ),
    ("sync.mjs", include_str!("../audio-service/src/sync.mjs")),
];

fn hidden(command: &mut Command) -> &mut Command {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
    }
    command
}

fn runtime_asset(os: &str, arch: &str) -> Result<(&'static str, &'static str), String> {
    match (os, arch) {
        ("linux", "x86_64") => Ok((
            "node-v22.22.0-linux-x64.tar.gz",
            "c33c39ed9c80deddde77c960d00119918b9e352426fd604ba41638d6526a4744",
        )),
        ("linux", "aarch64") => Ok((
            "node-v22.22.0-linux-arm64.tar.gz",
            "25ba95dfb96871fa2ef977f11f95ea90818c8fa15c0f2110771db08d4ba423be",
        )),
        ("macos", "x86_64") => Ok((
            "node-v22.22.0-darwin-x64.tar.gz",
            "5ea50c9d6dea3dfa3abb66b2656f7a4e1c8cef23432b558d45fb538c7b5dedce",
        )),
        ("macos", "aarch64") => Ok((
            "node-v22.22.0-darwin-arm64.tar.gz",
            "5ed4db0fcf1eaf84d91ad12462631d73bf4576c1377e192d222e48026a902640",
        )),
        ("windows", "x86_64") => Ok((
            "win-x64/node.exe",
            "bae898add4643fcf890a83ad8ae56e20dce7e781cab161a53991ceba70c99ffb",
        )),
        ("windows", "aarch64") => Ok((
            "win-arm64/node.exe",
            "fd44256121597d6a3707f4c7730b4e3733eacb5a95cc78a099f601d7e7f8290d",
        )),
        _ => Err("unsupported local audio runtime platform".into()),
    }
}

fn runtime(root: &Path) -> Result<PathBuf, String> {
    // Node 22+ provides native fetch and WebSocket, so no npm installation runs.
    if let Ok(output) = hidden(Command::new("node").arg("--version")).output()
        && output.status.success()
        && String::from_utf8_lossy(&output.stdout)
            .trim()
            .trim_start_matches('v')
            .split('.')
            .next()
            .and_then(|v| v.parse::<u32>().ok())
            .is_some_and(|v| v >= 22)
    {
        return Ok(PathBuf::from("node"));
    }
    let folder = root.join(format!("runtime-{VERSION}"));
    let node = folder.join(if cfg!(windows) { "node.exe" } else { "node" });
    if node.is_file() {
        return Ok(node);
    }
    std::fs::create_dir_all(&folder).map_err(|e| e.to_string())?;
    let (asset, sha) = runtime_asset(std::env::consts::OS, std::env::consts::ARCH)?;
    let archive = folder.join("download.partial");
    crate::models::download(
        &format!("https://nodejs.org/dist/{VERSION}/{asset}"),
        &archive,
        0,
        &std::sync::atomic::AtomicBool::new(false),
        "cancelled",
        &mut |_, _| {},
    )?;
    if let Err(error) = crate::models::verify(&archive, sha) {
        let _ = std::fs::remove_file(&archive);
        return Err(error);
    }
    if cfg!(windows) {
        std::fs::rename(&archive, &node).map_err(|e| e.to_string())?;
    } else {
        let member = format!("{}/bin/node", asset.trim_end_matches(".tar.gz"));
        let staging = tempfile::tempdir_in(&folder).map_err(|e| e.to_string())?;
        let status = Command::new("tar")
            .arg("-xzf")
            .arg(&archive)
            .arg("-C")
            .arg(staging.path())
            .arg("--strip-components=2")
            .arg(member)
            .status()
            .map_err(|e| e.to_string())?;
        if !status.success() || !staging.path().join("node").is_file() {
            return Err("could not extract local audio runtime".into());
        }
        std::fs::rename(staging.path().join("node"), &node).map_err(|e| e.to_string())?;
        let _ = std::fs::remove_file(archive);
    }
    Ok(node)
}

fn healthy(base: &url::Url) -> bool {
    let Ok(url) = base.join("health") else {
        return false;
    };
    crate::net::agent(Duration::from_secs(1), Duration::from_secs(1))
        .get(url.as_str())
        .call()
        .ok()
        .and_then(|r| serde_json::from_reader::<_, serde_json::Value>(r.into_reader()).ok())
        .is_some_and(|v| v.get("status").and_then(|s| s.as_str()) == Some("ok"))
}

/// Reuse an already running connector. Otherwise start our bundled connector
/// for the default loopback endpoint, only after a deliberate library action.
pub fn ensure(dirs: &AppDirs, api: &str) -> Result<bool, String> {
    let base = crate::youtube_audio::api_base(api)?;
    if base.as_str() != format!("{}/", crate::youtube_audio::DEFAULT_API) {
        return Ok(false);
    }
    let _guard = START
        .lock()
        .map_err(|_| "local audio service lock failed")?;
    if healthy(&base) {
        return Ok(false);
    }
    let root = dirs.data.join("youtube-audio");
    std::fs::create_dir_all(&root).map_err(|e| e.to_string())?;
    let node = runtime(&root)?;
    let mut hash = Sha256::new();
    for (name, content) in FILES {
        hash.update(name);
        hash.update(content);
    }
    let source = root.join(format!("connector-{:x}", hash.finalize()));
    let scripts = source.join("src");
    std::fs::create_dir_all(&scripts).map_err(|e| e.to_string())?;
    for (name, content) in FILES {
        let target = scripts.join(name);
        if !target.exists() {
            std::fs::write(target, content).map_err(|e| e.to_string())?;
        }
    }
    // Session and catalogue survive connector source updates.
    let data = root.join("data");
    std::fs::create_dir_all(&data).map_err(|e| e.to_string())?;
    let log = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(root.join("service.log"))
        .map_err(|e| e.to_string())?;
    hidden(
        Command::new(node)
            .arg(scripts.join("server.mjs"))
            .current_dir(&source)
            .env("PORT", "8917")
            .env("CLOVERCUT_AUDIO_DATA", &data)
            .env("CLOVERCUT_STUDIO_PROFILE", root.join("browser-profile"))
            .stdin(Stdio::null())
            .stdout(log.try_clone().map_err(|e| e.to_string())?)
            .stderr(log),
    )
    .spawn()
    .map_err(|e| e.to_string())?;
    for _ in 0..50 {
        std::thread::sleep(Duration::from_millis(100));
        if healthy(&base) {
            return Ok(true);
        }
    }
    Err(crate::youtube_audio::API_UNAVAILABLE.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn runtime_assets_cover_desktop_architectures_with_pinned_hashes() {
        for os in ["windows", "linux", "macos"] {
            for arch in ["x86_64", "aarch64"] {
                let (asset, hash) = runtime_asset(os, arch).unwrap();
                assert_eq!(hash.len(), 64);
                assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
                assert!(asset.ends_with(".exe") || asset.ends_with(".tar.gz"));
            }
        }
        assert!(runtime_asset("android", "aarch64").is_err());
    }
}
