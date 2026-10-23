// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Clovercut contributors

//! Versioned online GL Transitions catalogue and validated, persistent packages.
//! Network operations run on a worker. Installed packages use the same shader
//! contract for thumbnails, the monitor and export; projects retain stable IDs.

use concat_effects::Package;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, io::Read, path::Path, time::Duration};

/// Pinned distribution: cached packages never silently change beneath a project.
pub const VERSION: &str = "1.76.1";
const BASE: &str = "https://cdn.jsdelivr.net/npm/gl-transitions@1.76.1";
const LIMIT: u64 = 4 * 1024 * 1024;

/// Metadata from the official published collection.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Entry {
    /// Upstream transition name, also used to locate its WGSL source.
    pub name: String,
    /// Original author.
    #[serde(default)]
    pub author: String,
    /// Original license.
    pub license: String,
    /// Searchable explanation.
    #[serde(default)]
    pub description: String,
    /// Original parameter types.
    #[serde(default, rename = "paramsTypes")]
    pub types: BTreeMap<String, String>,
    /// Original defaults, retained including vector components.
    #[serde(default, rename = "defaultParams")]
    pub defaults: BTreeMap<String, serde_json::Value>,
    /// Extra image inputs, currently unsupported by the two-image contract.
    #[serde(default)]
    pub textures: Vec<String>,
    /// Original GLSL, including the author's complete copyright/license header.
    #[serde(default)]
    pub glsl: String,
}

impl Entry {
    /// Stable package ID independent of upstream catalogue ordering.
    pub fn id(&self) -> String {
        format!(
            "gltransitions.{}",
            self.name.to_ascii_lowercase().replace('_', "-")
        )
    }
    /// Whether the entry can be offered by this two-input renderer.
    pub fn supported(&self) -> bool {
        !self.name.is_empty()
            && self.name.len() <= 100
            && self
                .name
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
            && self.textures.is_empty()
            && matches!(
                self.license.as_str(),
                "MIT" | "BSD-2-Clause" | "BSD-3-Clause" | "BSD 2 Clause" | "BSD 3 Clause"
            )
    }
    /// True for the six legacy packages; keep their IDs and avoid duplicate rows.
    pub fn legacy(&self) -> bool {
        matches!(
            self.name.as_str(),
            "BowTieHorizontal" | "CircleCrop" | "crosshatch" | "doorway" | "Dreamy" | "WaterDrop"
        )
    }
}

fn get(path: &str) -> Result<String, String> {
    let response = crate::net::agent(Duration::from_secs(10), Duration::from_secs(30))
        .get(&format!("{BASE}/{path}"))
        .call()
        .map_err(|e| e.to_string())?;
    let mut body = String::new();
    response
        .into_reader()
        .take(LIMIT + 1)
        .read_to_string(&mut body)
        .map_err(|e| e.to_string())?;
    if body.len() as u64 > LIMIT {
        return Err("GL Transitions response is too large".into());
    }
    Ok(body)
}

fn parse(body: &str) -> Result<Vec<Entry>, String> {
    let entries: Vec<Entry> = serde_json::from_str(body).map_err(|e| e.to_string())?;
    if entries.is_empty() || entries.len() > 1000 {
        return Err("Invalid GL Transitions catalogue size".into());
    }
    let mut ids = std::collections::HashSet::new();
    Ok(entries
        .into_iter()
        .filter(|e| e.supported() && ids.insert(e.id()))
        .collect())
}

/// Loads the pinned catalogue from disk, downloading it on first use only.
/// Old cached data stays usable offline.
pub fn catalogue(cache: &Path) -> Result<Vec<Entry>, String> {
    let path = cache.join(format!("gl-transitions-{VERSION}.json"));
    if let Ok(body) = std::fs::read_to_string(&path)
        && let Ok(entries) = parse(&body)
    {
        return Ok(entries);
    }
    let body = get("gl-transitions.json")?;
    let entries = parse(&body)?;
    std::fs::create_dir_all(cache).map_err(|e| e.to_string())?;
    let mut file = tempfile::NamedTempFile::new_in(cache).map_err(|e| e.to_string())?;
    std::io::Write::write_all(&mut file, body.as_bytes()).map_err(|e| e.to_string())?;
    file.persist(&path).map_err(|e| e.to_string())?;
    Ok(entries)
}

fn quoted(s: &str) -> String {
    serde_json::to_string(s).expect("string")
}

/// Adapts the official WGSL's bindings and parameters to Clovercut's contract.
/// Source functions are namespaced; the result is parsed and validated by naga
/// before it is ever installed. Scalar components expose all vector defaults.
pub fn adapt(entry: &Entry, source: &str) -> Result<(String, String), String> {
    if !entry.supported() {
        return Err("Unsupported GL transition".into());
    }
    let start = source
        .find("struct Uniforms {")
        .ok_or("Missing GL uniforms")?;
    let end = start + source[start..].find('}').ok_or("Unclosed GL uniforms")?;
    let mut params = String::new();
    let mut fields = String::new();
    let mut assignments = String::new();
    for line in source[start + "struct Uniforms {".len()..end].lines() {
        let Some((raw, ty)) = line.trim().trim_end_matches(',').split_once(':') else {
            continue;
        };
        let key = raw.trim();
        let ty = ty.trim();
        if key == "progress" || key == "ratio" {
            continue;
        }
        if !key.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_') {
            return Err("Invalid uniform name".into());
        }
        let original = if entry.defaults.contains_key(key) {
            key
        } else {
            key.trim_end_matches('_')
        };
        let default = entry
            .defaults
            .get(original)
            .ok_or_else(|| format!("Missing default for {key}"))?;
        let components: Vec<f64> = if let Some(array) = default.as_array() {
            array
                .iter()
                .map(|v| v.as_f64().ok_or("Invalid vector default"))
                .collect::<Result<_, _>>()?
        } else {
            vec![
                default
                    .as_f64()
                    .or_else(|| default.as_bool().map(|b| if b { 1.0 } else { 0.0 }))
                    .ok_or("Invalid default")?,
            ]
        };
        let count = match ty {
            "f32" | "i32" | "u32" => 1,
            "vec2<f32>" | "vec2<i32>" => 2,
            "vec3<f32>" => 3,
            "vec4<f32>" => 4,
            _ => return Err(format!("Unsupported uniform type {ty}")),
        };
        if count != components.len() {
            return Err(format!("Wrong default size for {key}"));
        }
        let mut args = Vec::new();
        for (i, value) in components.iter().enumerate() {
            if !value.is_finite() {
                return Err("Nonfinite parameter".into());
            }
            let component = if count == 1 {
                format!("p_{}", key.to_ascii_lowercase())
            } else {
                format!("p_{}_{i}", key.to_ascii_lowercase())
            };
            let kind = match entry.types.get(original).map(String::as_str) {
                Some("bool") => "bool",
                Some("int" | "ivec2") => "int",
                _ => "float",
            };
            let (min, max) = if kind == "bool" {
                (0.0, 1.0)
            } else {
                (value.min(0.0) - value.abs(), (value.abs() * 4.0).max(1.0))
            };
            params.push_str(&format!("\n[[param]]\nkey = {}\nlabel = {}\ntype = {}\nmin = {min:?}\nmax = {max:?}\ndefault = {value:?}\n", quoted(&component), quoted(&if count == 1 { original.to_owned() } else { format!("{} {}", original, ["X", "Y", "Z", "W"][i]) }), quoted(kind)));
            fields.push_str(&format!("{component}: f32,\n"));
            args.push(if ty.contains("i32") {
                format!("i32(params.{component})")
            } else if ty == "u32" {
                format!("u32(params.{component})")
            } else {
                format!("params.{component}")
            });
        }
        let value = if count == 1 {
            args[0].clone()
        } else {
            format!("{ty}({})", args.join(", "))
        };
        assignments.push_str(&format!("gl_uniforms.{key} = {value};\n"));
    }
    // The official generated module has one fragment entry at its end.
    let mut body = source[..source
        .find("@fragment")
        .ok_or("Missing GL fragment entry")?]
        .to_owned();
    while let Some(at) = body.find("@group(") {
        let tail = at + body[at..].find(';').ok_or("Invalid GL binding")? + 1;
        body.replace_range(at..tail, "");
    }
    for name in ["getFromColor", "getToColor"] {
        let at = body
            .find(&format!("fn {name}("))
            .ok_or("Missing GL sampler")?;
        let open = at + body[at..].find('{').ok_or("Missing function body")?;
        let mut depth = 1;
        let mut close = open + 1;
        for (offset, c) in body[open + 1..].char_indices() {
            if c == '{' {
                depth += 1;
            } else if c == '}' {
                depth -= 1;
            }
            if depth == 0 {
                close = open + 2 + offset;
                break;
            }
        }
        if depth != 0 {
            return Err("Unclosed GL sampler".into());
        }
        body.replace_range(at..close, "");
    }
    // Rename all functions plus module globals to avoid host helper collisions.
    let mut names = std::collections::HashSet::from([
        "Uniforms".to_owned(),
        "uniforms".to_owned(),
        "_fragColor".to_owned(),
        "_uv_1".to_owned(),
        "getFromColor".to_owned(),
        "getToColor".to_owned(),
    ]);
    for rest in body.split("fn ").skip(1) {
        if let Some(name) = rest.split('(').next() {
            names.insert(name.trim().to_owned());
        }
    }
    let mut mapped = String::new();
    let mut token = String::new();
    for c in body.chars().chain(std::iter::once('\n')) {
        if c.is_ascii_alphanumeric() || c == '_' {
            token.push(c);
        } else {
            if names.contains(&token) {
                mapped.push_str("gl_");
            }
            mapped.push_str(&token);
            token.clear();
            mapped.push(c);
        }
    }
    if fields.is_empty() {
        fields.push_str("unused: f32,\n");
    }
    mapped.push_str(&format!(r#"
var<private> gl_uniforms: gl_Uniforms;
struct Params {{ {fields} }}
fn gl_getFromColor(uv: ptr<function, vec2<f32>>) -> vec4<f32> {{ return from_at(vec2<f32>((*uv).x, 1.0 - (*uv).y)); }}
fn gl_getToColor(uv: ptr<function, vec2<f32>>) -> vec4<f32> {{ return to_at(vec2<f32>((*uv).x, 1.0 - (*uv).y)); }}
fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {{
    if progress <= 0.0 {{ return from_at(uv); }}
    if progress >= 1.0 {{ return to_at(uv); }}
    gl_uniforms.progress = progress;
    gl_uniforms.ratio = frame.size.x / frame.size.y;
    {assignments}
    var point = vec2<f32>(uv.x, 1.0 - uv.y);
    return gl_transition(&point);
}}
"#));
    let manifest = format!(
        "format = 2\n[effect]\nid = {}\nname = {}\nkind = \"transition\"\ncategory = \"GL Transitions\"\ndescription = {}\n[transition]\nentry = \"effect.wgsl\"\nfallback = \"cross-fade\"\n{params}",
        quoted(&entry.id()),
        quoted(&entry.name),
        quoted(&format!(
            "{} — {} · {}",
            entry.description, entry.author, entry.license
        ))
    );
    Package::from_sources(&manifest, None, Some(&mapped)).map_err(|e| e.to_string())?;
    Ok((manifest, mapped))
}

/// Downloads and atomically installs one validated shader, or reuses its package.
/// Staging lives outside the watched effects directory, so partial downloads never
/// enter the catalogue. A corrupt cache is refused rather than used for export.
pub fn install(entry: &Entry, effects: &Path) -> Result<(), String> {
    if !entry.supported() {
        return Err("Unsupported GL transition".into());
    }
    let destination = effects.join(entry.id());
    if destination.exists() {
        Package::from_folder(&destination).map_err(|e| e.to_string())?;
        return Ok(());
    }
    let source = get(&format!("wgsl/{}.wgsl", entry.name))?;
    let (manifest, shader) = adapt(entry, &source)?;
    std::fs::create_dir_all(effects).map_err(|e| e.to_string())?;
    let staging = tempfile::tempdir_in(effects.parent().ok_or("Missing effects parent")?)
        .map_err(|e| e.to_string())?;
    for (name, body) in [
        ("effect.toml", manifest),
        ("effect.wgsl", shader),
        ("upstream.wgsl", source),
        ("upstream.glsl", entry.glsl.clone()),
        ("LICENSE", get("LICENSE")?),
        (
            "source.json",
            serde_json::to_string_pretty(entry).map_err(|e| e.to_string())?,
        ),
    ] {
        std::fs::write(staging.path().join(name), body).map_err(|e| e.to_string())?;
    }
    std::fs::rename(staging.path(), &destination).map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn catalogue_rejects_paths_and_additional_texture_inputs() {
        let mut e: Entry = serde_json::from_str(r#"{"name":"Dreamy","license":"MIT"}"#).unwrap();
        assert!(e.supported());
        e.name = "../escape".into();
        assert!(!e.supported());
        e.name = "luma".into();
        e.textures.push("luma".into());
        assert!(!e.supported());
    }
    #[test]
    #[ignore = "requires official shader fixtures; set CLOVERCUT_GL_FIXTURES and CLOVERCUT_GL_INDEX"]
    fn official_catalogue_adapts_and_validates() {
        let index = std::env::var("CLOVERCUT_GL_INDEX").expect("index");
        let fixtures = std::env::var("CLOVERCUT_GL_FIXTURES").expect("fixtures");
        let entries = parse(&std::fs::read_to_string(index).unwrap()).unwrap();
        let mut failures = Vec::new();
        for entry in &entries {
            let source =
                std::fs::read_to_string(Path::new(&fixtures).join(format!("{}.wgsl", entry.name)))
                    .unwrap();
            if let Err(error) = adapt(entry, &source) {
                failures.push(format!("{}: {error}", entry.name));
            }
        }
        assert!(
            failures.is_empty(),
            "{} failed of {}:\n{}",
            failures.len(),
            entries.len(),
            failures.join("\n")
        );
        assert!(entries.len() > 100);
    }
}
